use crate::config::jarvis::JarvisAppState;
use crate::models::nervo_message_model::TelegramMessage;
use crate::models::system_messages::SystemMessages;
use crate::models::user_model::TelegramUser;
use crate::utils::ai_utils::{llm_conversation, RESOURCES_DIR};
use anyhow::bail;
use chrono::Utc;
use nervo_api::agent_type::{AgentType, NervoAgentType};
use nervo_api::{LlmMessageContent, SendMessageRequest, UserLlmMessage};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{
    AudioOutputFormat, AudioSpeechParameters, AudioSpeechResponseFormat, AudioTranscriptionFile,
    AudioTranscriptionParameters, AudioVoice,
};
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::prelude::ChatId;
use teloxide::prelude::*;
use teloxide::types::{
    ChatKind, File, FileMeta, InputFile, MediaKind, MessageKind, ParseMode, ReplyParameters, User,
};
use teloxide::Bot;
use tokio::fs;
use tracing::{error, info};

pub async fn chat(
    bot: Bot,
    msg: Message,
    app_state: Arc<JarvisAppState>,
    agent_type: AgentType,
) -> anyhow::Result<()> {
    info!("Start chat...");
    let mut parser = MessageParser {
        bot: &bot,
        msg: &msg,
        app_state: &app_state,
        is_voice: false,
    };

    // Get info about bot and user
    let bot_info = &bot.get_me().await?;
    let bot_name = bot_info.clone().user.username.unwrap();
    let user = parser.parse_user().await?;
    let UserId(user_id) = user.id;

    // We need it for future. Just to send spam etc.
    save_user_id(app_state.clone(), user_id.to_string()).await?;

    // Need to detect it in group chats. To understand whether to answer or not.
    let is_reply = msg
        .clone()
        .reply_to_message()
        .and_then(|message| message.from.as_ref())
        .and_then(|user| user.username.clone())
        .map_or(false, |username| username == bot_name.clone());

    let MessageKind::Common(msg_common) = &msg.kind else {
        bail!("Unsupported message content type: {:?}.", msg.kind);
    };

    let is_text = matches!(&msg_common.media_kind, MediaKind::Text(_media_text));
    let is_private = matches!(&msg.chat.kind, ChatKind::Private(_));
    let is_public = matches!(&msg.chat.kind, ChatKind::Public(_));
    let is_public_and_text = is_public && is_text;

    // Parse message to raw text
    let text = parser.parse_message().await?;
    let contains_bot_name = text.contains(&bot_name);

    // Answer formation
    if is_private || is_reply || is_public_and_text {
        // Parse message to raw text
        let text = parser.parse_message().await?;
        if contains_bot_name || is_reply || is_private {
            // Show typing indicator
            let _ = &bot
                .send_chat_action(msg.chat.id.clone(), teloxide::types::ChatAction::Typing)
                .await?;

            let reply_parameters = ReplyParameters {
                message_id: msg.id,
                chat_id: None,
                allow_sending_without_reply: None,
                quote: None,
                quote_parse_mode: None,
                quote_entities: None,
                quote_position: None,
            };
            if text.is_empty() {
                bot.send_message(msg.chat.id, "Please provide a message to send.")
                    .reply_parameters(reply_parameters)
                    .await?;

                return Ok(());
            }

            let message_text = text;

            // Moderation checking
            let is_moderation_passed = app_state.nervo_llm.moderate(&message_text).await?;
            if is_moderation_passed {
                let tg_message = TelegramMessage {
                    id: user_id,
                    message: message_text.clone(),
                    timestamp: Utc::now().naive_utc(),
                };

                // Create question for LLM
                let question_msg = SendMessageRequest {
                    chat_id: msg.chat.id.0 as u64,
                    agent_type,
                    llm_message: UserLlmMessage {
                        sender_id: user_id,
                        content: LlmMessageContent::from(tg_message.message.as_str()),
                    },
                };

                chat_gpt_conversation(
                    &bot,
                    &msg,
                    app_state.clone(),
                    question_msg,
                    parser.is_voice,
                    false,
                    agent_type,
                )
                .await?
            } else {
                // Moderation is not passed
                if let Some(_) = &user.username {
                    let question = format!("I have a message from the user, I know the message is unacceptable, can you please read the message and reply that the message is not acceptable. Reply using the same language the massage uses. Here is the message: {:?}", &message_text);
                    let question_msg = SendMessageRequest {
                        chat_id: msg.chat.id.0 as u64,
                        agent_type,
                        llm_message: UserLlmMessage {
                            sender_id: user_id,
                            content: LlmMessageContent::from(question.as_str()),
                        },
                    };

                    chat_gpt_conversation(
                        &bot,
                        &msg,
                        app_state.clone(),
                        question_msg,
                        parser.is_voice,
                        true,
                        agent_type,
                    )
                    .await?
                } else {
                    return Ok(());
                }
            }
            return Ok(());
        }
    }
    Ok(())
}

// PARSING USER & TEXT & VOICE
pub struct MessageParser<'a> {
    pub bot: &'a Bot,
    pub(crate) msg: &'a Message,
    pub app_state: &'a Arc<JarvisAppState>,
    pub is_voice: bool,
}

impl<'a> MessageParser<'a> {
    pub fn set_is_voice(&mut self, is_voice: bool) {
        self.is_voice = is_voice;
    }
}

impl<'a> MessageParser<'a> {
    // Get user from TG message
    pub async fn parse_user(&mut self) -> anyhow::Result<User> {
        let Some(user) = &self.msg.from else {
            bail!("User not found. We can handle only direct messages.");
        };

        Ok(user.clone())
    }

    // Get text from TG message
    pub async fn parse_message(&mut self) -> anyhow::Result<String> {
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("Unsupported message content type.");
        };

        let Some(user) = &self.msg.from else {
            bail!("User not found. We can handle only direct messages.");
        };

        let media_kind = &msg_common.media_kind;

        let result_text = match media_kind {
            MediaKind::Text(media_text) => media_text.text.clone(),
            MediaKind::Voice(media_voice) => {
                let (_, text) = self.user_and_voice(&media_voice.voice.file, &user).await?;
                text.clone()
            }
            MediaKind::Audio(media_voice) => {
                let (_, text) = self.user_and_voice(&media_voice.audio.file, &user).await?;
                text.clone()
            }
            _ => {
                bail!("Unsupported case. We can handle only direct messages.");
            }
        };

        Ok(result_text)
    }

    // Get voice from TG message
    pub async fn user_and_voice(
        &mut self,
        media_voice: &FileMeta,
        user: &User,
    ) -> anyhow::Result<(User, String)> {
        let reply_parameters = ReplyParameters {
            message_id: self.msg.id,
            chat_id: None,
            allow_sending_without_reply: None,
            quote: None,
            quote_parse_mode: None,
            quote_entities: None,
            quote_position: None,
        };

        info!("COMMON: Generate audio message");
        self.bot
            .send_message(
                self.msg.chat.id.clone(),
                "Один момент, сейчас отвечу!".to_string(),
            )
            .reply_parameters(reply_parameters)
            .await?;

        let file: File = self.bot.get_file(&media_voice.id).await?;

        let file_extension = "oga";
        let file_name: &str = &file.id;
        let file_path = format!("/tmp/{}.{}", &file_name, &file_extension);

        let mut dst = fs::File::create(&file_path).await?;

        if fs::metadata(&file_path).await.is_ok() {
            self.bot.download_file(&file.path, &mut dst).await?;

            let parameters = AudioTranscriptionParameters {
                file: AudioTranscriptionFile::File(file_path.to_string()),
                model: "whisper-1".to_string(),
                language: None,
                prompt: None,
                response_format: Some(AudioOutputFormat::Text),
                temperature: None,
                timestamp_granularities: None,
            };

            // let parameters = AudioTranscriptionParameters {
            //     file: file_path.to_string(),
            //     model: "whisper-1".to_string(),
            //     language: None,
            //     prompt: None,
            //     response_format: Some(AudioOutputFormat::Text),
            //     temperature: None,
            //     timestamp_granularities: None,
            // };

            let client = Client::new(self.app_state.nervo_llm.api_key().to_string());
            let response = client.audio().create_transcription(parameters).await;

            fs::remove_file(&file_path).await?;
            drop(dst);

            match response {
                Ok(text) => {
                    self.set_is_voice(true);
                    Ok((user.clone(), text.clone()))
                }
                Err(err) => {
                    error!("ERROR {:?}", err.to_string());
                    Err(anyhow::Error::msg(err.to_string()))
                }
            }
        } else {
            Err(anyhow::Error::msg(format!(
                "File '{}' doesn't exist.",
                file_path
            )))
        }
    }
}

// Sending some system messages
pub async fn system_message(
    bot: &Bot,
    msg: &Message,
    message_type: SystemMessage,
) -> anyhow::Result<()> {
    let introduction_msg = message_type.as_str().await?;
    let reply_parameters = ReplyParameters {
        message_id: msg.id,
        chat_id: None,
        allow_sending_without_reply: None,
        quote: None,
        quote_parse_mode: None,
        quote_entities: None,
        quote_position: None,
    };

    info!("COMMON: Send system message");
    bot.send_message(msg.chat.id, introduction_msg)
        .reply_parameters(reply_parameters)
        .await?;

    Ok(())
}

pub enum SystemMessage {
    Start(AgentType),
    Manual(AgentType),
}

impl SystemMessage {
    fn agent_type(&self) -> AgentType {
        match self {
            SystemMessage::Start(agent_type) => agent_type.clone(),
            SystemMessage::Manual(agent_type) => agent_type.clone(),
        }
    }

    pub async fn as_str(&self) -> anyhow::Result<String> {
        let agent = NervoAgentType::get_name(self.agent_type());
        let system_msg_file = format!("{}/agent/{}/system_messages.json", RESOURCES_DIR, agent);

        let json_string = fs::read_to_string(system_msg_file).await?;
        let system_messages_models: SystemMessages = serde_json::from_str(&json_string)?;

        match self {
            SystemMessage::Start(_) => Ok(system_messages_models.start.clone()),
            SystemMessage::Manual(_) => Ok(system_messages_models.manual.clone()),
        }
    }
}

// Work with User Ids
async fn save_user_id(app_state: Arc<JarvisAppState>, user_id: String) -> anyhow::Result<()> {
    let user_ids = load_user_ids(app_state.clone()).await?;

    let contains_id = user_ids.iter().any(|user| user.id == user_id);
    if !contains_id {
        let user = TelegramUser {
            id: user_id.parse()?,
        };
        app_state
            .local_db
            .save_to_local_db(user, "all_users_list", None)
            .await?;
    }
    Ok(())
}

async fn load_user_ids(app_state: Arc<JarvisAppState>) -> anyhow::Result<Vec<TelegramUser>> {
    match app_state
        .local_db
        .read_from_local_db("all_users_list")
        .await
    {
        Ok(ids) => Ok(ids),
        Err(_) => Ok(Vec::new()),
    }
}

pub async fn chat_gpt_conversation(
    bot: &Bot,
    message: &Message,
    app_state: Arc<JarvisAppState>,
    msg: SendMessageRequest,
    is_voice: bool,
    direct_message: bool,
    agent_type: AgentType,
) -> anyhow::Result<()> {
    let chat_id = msg.chat_id;
    let table_name = msg.llm_message.sender_id.to_string();

    let user_final_question = if direct_message {
        msg.llm_message.content.text()
    } else {
        llm_conversation(app_state.clone(), msg, table_name, agent_type)
            .await?
            .content
            .text()
    };

    if is_voice {
        create_speech(bot, user_final_question.as_str(), chat_id, app_state).await;
    } else {
        let reply_parameters = ReplyParameters {
            message_id: message.id,
            chat_id: None,
            allow_sending_without_reply: None,
            quote: None,
            quote_parse_mode: None,
            quote_entities: None,
            quote_position: None,
        };

        info!("COMMON: Send ANSWER");
        bot.send_message(ChatId(chat_id as i64), user_final_question)
            .parse_mode(ParseMode::Markdown)
            .reply_parameters(reply_parameters)
            .await?;
    }

    Ok(())
}

async fn create_speech(bot: &Bot, text: &str, chat_id: u64, app_state: Arc<JarvisAppState>) {
    let client = Client::new(app_state.nervo_llm.api_key().to_string());

    let parameters = AudioSpeechParameters {
        model: "tts-1".to_string(),
        input: text.to_string(),
        voice: AudioVoice::Onyx,
        response_format: Some(AudioSpeechResponseFormat::Mp3),
        speed: Some(1.0),
    };

    let response = client.audio().create_speech(parameters).await;
    match response {
        Ok(audio) => {
            // stop_loop();
            let input_file = InputFile::memory(audio.bytes);
            let _ = bot.send_voice(ChatId(chat_id as i64), input_file).await;
        }
        Err(err) => {
            error!("ERROR: {:?}", err);
            info!("CMOMON: Send ERROR: {:?}", err);
            let _ = bot
                .send_message(ChatId(chat_id as i64), err.to_string())
                .await;
        }
    }
}
