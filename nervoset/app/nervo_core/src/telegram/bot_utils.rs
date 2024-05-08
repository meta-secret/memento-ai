use crate::common::AppState;
use crate::models::nervo_message_model::TelegramMessage;
use crate::models::qdrant_search_layers::{
    QDrantSearchInfo, QDrantSearchLayer, QDrantUserRoleTextType,
};
use crate::models::system_messages::SystemMessages;
use crate::models::user_model::TelegramUser;
use anyhow::bail;
use async_openai::types::{ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs, ChatCompletionRequestUserMessageContent};
use chrono::Utc;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{
    AudioOutputFormat, AudioSpeechParameters, AudioSpeechResponseFormat,
    AudioTranscriptionParameters, AudioVoice,
};
use openai_dive::v1::resources::chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent};
use qdrant_client::qdrant::value::Kind;
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::prelude::ChatId;
use teloxide::prelude::*;
use teloxide::types::{
    ChatKind, File, FileMeta, InputFile, MediaKind, MessageKind, ParseMode, User,
};
use teloxide::Bot;
use tiktoken_rs::p50k_base;
use tokio::fs;
use tracing::{error, info};

pub async fn chat(bot: Bot, msg: Message, app_state: Arc<AppState>) -> anyhow::Result<()> {
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

    // We need it for future. Just to send spam and etc.
    save_user_id(app_state.clone(), user.id.to_string()).await?;

    // Need to detect it in group chats. To understand whether to answer or not.
    let is_reply = msg
        .clone()
        .reply_to_message()
        .and_then(|message| message.from())
        .and_then(|user| user.username.clone())
        .map_or(false, |username| username == bot_name.clone());

    let MessageKind::Common(msg_common) = &msg.kind else {
        bail!("Unsupported message content type.");
    };
    let is_text = match &msg_common.media_kind {
        MediaKind::Text(_media_text) => true,
        _ => false,
    };
    // Answer formation
    if matches!(&msg.chat.kind, ChatKind::Private(_))
        || is_reply
        || (matches!(&msg.chat.kind, ChatKind::Public(_)) && is_text)
    {
        // Parse message to raw text
        let text = parser.parse_message().await?;
        if (text.contains(&bot_name)) || is_reply || matches!(&msg.chat.kind, ChatKind::Private(_))
        {
            // Show typing indicator
            let _ = &bot
                .send_chat_action(msg.chat.id.clone(), teloxide::types::ChatAction::Typing)
                .await?;

            if text.is_empty() {
                bot.send_message(msg.chat.id, "Please provide a message to send.")
                    .reply_to_message_id(msg.id.clone())
                    .await?;

                return Ok(());
            }

            let message_text = text;

            // Moderation checking
            let is_moderation_passed = app_state.nervo_llm.moderate(&message_text).await?;
            if is_moderation_passed {
                let tg_message = TelegramMessage {
                    id: msg.chat.id.0 as u64,
                    message: message_text.clone(),
                    timestamp: Utc::now().naive_utc(),
                };

                // Local caching
                if let Some(name) = &user.username {
                    app_state
                        .local_db
                        .save_to_local_db(tg_message.clone(), &name, true)
                        .await?;
                }
                let question_msg = ChatCompletionRequestUserMessageArgs::default()
                    .content(tg_message.message.clone())
                    .build()?;
                
                chat_gpt_conversation(
                    &bot,
                    &msg,
                    msg.chat.id,
                    &app_state,
                    question_msg,
                    parser.is_voice,
                    &user,
                    false,
                )
                .await?
            } else {
                // Moderation is not passed
                if let Some(_) = &user.username {
                    let question = format!("I have a message from the user, I know the message is unacceptable, can you please read the message and reply that the message is not acceptable. Reply using the same language the massage uses. Here is the message: {:?}", &message_text);
                    let question_msg = ChatCompletionRequestUserMessageArgs::default()
                        .content(question)
                        .build()?;

                    chat_gpt_conversation(
                        &bot,
                        &msg,
                        msg.chat.id,
                        &app_state,
                        question_msg,
                        parser.is_voice,
                        &user,
                        true,
                    )
                    .await?
                } else {
                    return Ok(());
                }
            }
            return Ok(());
        }
    }
    return Ok(());
}

// PARSING USER & TEXT & VOICE
pub struct MessageParser<'a> {
    pub bot: &'a Bot,
    pub(crate) msg: &'a Message,
    pub app_state: &'a Arc<AppState>,
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
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("Unsupported message content type.");
        };

        let Some(user) = &msg_common.from else {
            bail!("User not found. We can handle only direct messages.");
        };

        Ok(user.clone())
    }

    // Get text from TG message
    pub async fn parse_message(&mut self) -> anyhow::Result<String> {
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("Unsupported message content type.");
        };

        let Some(user) = &msg_common.from else {
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
        self.bot
            .send_message(
                self.msg.chat.id.clone(),
                "Один момент, сейчас отвечу!".to_string(),
            )
            .reply_to_message_id(self.msg.id.clone())
            .await?;

        let file: File = self.bot.get_file(&media_voice.id).await?;

        let file_extension = "oga";
        let file_name: &str = &file.id;
        let file_path = format!("/tmp/{}.{}", &file_name, &file_extension);

        let mut dst = fs::File::create(&file_path).await?;

        if fs::metadata(&file_path).await.is_ok() {
            self.bot.download_file(&file.path, &mut dst).await?;

            let parameters = AudioTranscriptionParameters {
                file: file_path.to_string(),
                model: "whisper-1".to_string(),
                language: None,
                prompt: None,
                response_format: Some(AudioOutputFormat::Text),
                temperature: None,
                timestamp_granularities: None,
            };

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
    let introduction_msg = message_type.as_str().await;
    bot.send_message(msg.chat.id, introduction_msg)
        .reply_to_message_id(msg.id.clone())
        .await?;

    Ok(())
}

pub enum SystemMessage {
    Start,
    Manual,
}

impl SystemMessage {
    pub async fn as_str(&self) -> String {
        let json_string = fs::read_to_string("resources/system_messages.json").await.unwrap();
        let system_messages_models: SystemMessages =
            serde_json::from_str(&json_string).expect("Failed to parse JSON");

        match self {
            SystemMessage::Start => system_messages_models.start.clone(),
            SystemMessage::Manual => system_messages_models.manual.clone(),
        }
    }
}

// Work with User Ids
async fn save_user_id(app_state: Arc<AppState>, user_id: String) -> anyhow::Result<()> {
    let user_ids = load_user_ids(&app_state).await?;

    let contains_id = user_ids.iter().any(|user| user.id == user_id);
    if !contains_id {
        let user = TelegramUser {
            id: user_id.parse().unwrap(),
        };
        app_state
            .local_db
            .save_to_local_db(user, "all_users_list", false)
            .await?;
    }
    Ok(())
}

async fn load_user_ids(app_state: &AppState) -> anyhow::Result<Vec<TelegramUser>> {
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
    chat_id: ChatId,
    app_state: &Arc<AppState>,
    msg: ChatCompletionRequestUserMessage,
    is_voice: bool,
    user: &User,
    direct_message: bool,
) -> anyhow::Result<()> {
    let user_final_question: String;
    if !direct_message {
        let msg_text = match msg.content {
            ChatCompletionRequestUserMessageContent::Text(text) => {text}
            ChatCompletionRequestUserMessageContent::Array(_) => { String::new() }
        };
        
        let initial_user_request =
            detecting_crap_request(&app_state, &msg_text, &user).await?;
        
        if initial_user_request == "SKIP" {
            let crap_system_role =
                std::fs::read_to_string("resources/crap_request_system_role.txt")
                    .expect("Failed to read system message from file");
            let user_request = format!(
                "{:?}\nТекущий запрос пользователя: {:?}",
                crap_system_role,
                &message.text().unwrap()
            );
            let request_to_llm = ChatCompletionRequestUserMessageArgs::default()
                .content(user_request)
                .build()?;
            user_final_question = app_state
                .nervo_llm
                .chat(request_to_llm)
                .await?
                .unwrap_or(String::from("I'm sorry, internal error."));
        } else {
            let result =
                openai_chat_preparations(&app_state, &initial_user_request, user.clone(), chat_id)
                    .await?;
            user_final_question = result
        };
    } else {
        user_final_question = match msg.content {
            ChatCompletionRequestUserMessageContent::Text(text) => {text}
            ChatCompletionRequestUserMessageContent::Array(_) => {String::new()}
        }
    }

    if is_voice {
        create_speech(&bot, user_final_question.clone(), chat_id, &app_state).await;
    } else {
        bot.send_message(chat_id, user_final_question)
            .parse_mode(ParseMode::Markdown)
            .reply_to_message_id(message.id.clone())
            .await?;
    }

    Ok(())
}

async fn create_speech(bot: &Bot, text: String, chat_id: ChatId, app_state: &AppState) {
    let client = Client::new(app_state.nervo_llm.api_key().to_string());

    let parameters = AudioSpeechParameters {
        model: "tts-1".to_string(),
        input: text,
        voice: AudioVoice::Onyx,
        response_format: Some(AudioSpeechResponseFormat::Mp3),
        speed: Some(1.0),
    };

    let response = client.audio().create_speech(parameters).await;
    match response {
        Ok(audio) => {
            // stop_loop();
            let input_file = InputFile::memory(audio.bytes);
            let _ = bot.send_voice(chat_id.clone(), input_file).await;
        }
        Err(err) => {
            error!("ERROR: {:?}", err);
            let _ = bot.send_message(chat_id.clone(), err.to_string()).await;
        }
    }
}

async fn openai_chat_preparations(
    app_state: &AppState,
    prompt: &str,
    user: User,
    chat_id: ChatId,
) -> anyhow::Result<String> {
    let mut rephrased_prompt = String::from(prompt);
    let bpe = p50k_base().unwrap();

    match user.username {
        None => {
            bail!("Oooops! Something went wrong")
        }
        Some(username) => {
            let layers_info = get_all_search_layers().await?;
            let processing_layers = layers_info.layers;
            let mut search_content = String::new();

            for processing_layer in processing_layers{
                if !processing_layer.collection_params.is_empty() {
                    for collection_param in &processing_layer.collection_params {
                        let db_search = &app_state
                            .nervo_ai_db
                            .search(
                                &app_state,
                                collection_param.name.clone(),
                                rephrased_prompt.clone(),
                                collection_param.vectors_limit.clone(),
                            )
                            .await?;

                        for search_result in &db_search.result {
                            if search_result.score.clone() > 0.1 {
                                let Some(Kind::StringValue(result)) =
                                    &search_result.payload["text"].kind
                                else {
                                    bail!("Oooops! Error")
                                };
                                
                                let token_limit =
                                    collection_param.tokens_limit.clone() as usize;
                                let mut tokens = bpe.split_by_token(&result, true)?;
                                if tokens.len() > token_limit {
                                    tokens.truncate(token_limit);
                                    let response = tokens.join("");
                                    search_content.push_str(&response);
                                } else {
                                    search_content.push_str(result);
                                }
                            }
                        }
                    }
                }

                rephrased_prompt = create_layer_content(
                    &app_state,
                    &prompt,
                    &user.id.to_string(),
                    processing_layer.clone(),
                    rephrased_prompt.clone(),
                    search_content.clone(),
                )
                .await?;
            }

            let tg_message = TelegramMessage {
                id: chat_id.0 as u64,
                message: rephrased_prompt.clone(),
                timestamp: Utc::now().naive_utc(),
            };
            app_state
                .local_db
                .save_to_local_db(tg_message, &username, false)
                .await?;

            let cached_messages: Vec<TelegramMessage> =
                app_state.local_db.read_from_local_db(&username).await?;

            if cached_messages.len() % 5 == 0 {
                rephrased_prompt.push_str(&layers_info.info_message);
            };

            Ok(String::from(rephrased_prompt.clone()))
        }
    }
}
async fn get_all_search_layers() -> anyhow::Result<QDrantSearchInfo> {
    let json_string = fs::read_to_string("resources/vectorisation_roles.json").await?;
    let layers_info: QDrantSearchInfo =
        serde_json::from_str(&json_string).expect("Failed to parse JSON");
    Ok(layers_info)
}

async fn create_layer_content(
    app_state: &AppState,
    prompt: &str,
    db_table_name: &str,
    layer: QDrantSearchLayer,
    rephrased_prompt: String,
    search_result_content: String,
) -> anyhow::Result<String> {
    let client = Client::new(app_state.nervo_llm.api_key().to_string());
    let cached_messages: Vec<TelegramMessage> =
        app_state.local_db.read_from_local_db(db_table_name).await?;
    let model_name = app_state.nervo_config.llm.model_name.clone();
    let system_role_content = layer.system_role_text;

    //Create user request role text
    let mut user_role_full_text = String::new();
    for parameter in layer.user_role_params {
        let value = match parameter.param_type {
            QDrantUserRoleTextType::History => cached_messages
                .clone()
                .iter()
                .map(|msg| msg.message.clone())
                .collect::<Vec<String>>()
                .join("\n"),
            QDrantUserRoleTextType::UserPromt => prompt.to_string().clone(),
            QDrantUserRoleTextType::RephrasedPromt => rephrased_prompt.clone(),
            QDrantUserRoleTextType::DBSearch => search_result_content.clone(),
        };
        let part = format!("{:?}{:?}\n", parameter.param_value, value);
        user_role_full_text.push_str(&part)
    }

    let mut messages: Vec<ChatMessage> = Vec::new();
    let system_role_msg = ChatMessage {
        role: openai_dive::v1::resources::chat::Role::System,
        content: ChatMessageContent::Text(String::from(system_role_content)),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };
    let user_role_msg = ChatMessage {
        role: openai_dive::v1::resources::chat::Role::User,
        content: ChatMessageContent::Text(String::from(user_role_full_text)),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };

    messages.push(system_role_msg);
    messages.push(user_role_msg);

    let params = ChatCompletionParameters {
        messages,
        model: model_name,
        frequency_penalty: None,
        logit_bias: None,
        logprobs: None,
        top_logprobs: None,
        max_tokens: Some(layer.max_tokens),
        n: None,
        presence_penalty: None,
        response_format: None,
        seed: None,
        stop: None,
        stream: None,
        temperature: Some(layer.temperature),
        top_p: None,
        tools: None,
        tool_choice: None,
        user: None,
    };
    let layer_processing_content = match client.chat().create(params).await {
        Ok(value) => value.choices[0].message.content.to_owned(),
        Err(err) => {
            error!("Error {:?}", err);
            bail!("Error {:?}", err)
        }
    };
    let layer_content_text = match layer_processing_content {
        ChatMessageContent::Text(text) => text,
        _ => String::new(),
    };
    Ok(layer_content_text)
}

// Need to check if it is crap question from user (e.g "Hi!", "What's up" etc.)
async fn detecting_crap_request(
    app_state: &Arc<AppState>,
    prompt: &str,
    user: &User,
) -> anyhow::Result<String> {
    let layers_info = get_all_search_layers().await?;
    let layer_content = create_layer_content(
        &app_state,
        &prompt,
        &user.id.to_string(),
        layers_info.crap_detecting_layer,
        String::new(),
        String::new(),
    )
    .await?;
    Ok(layer_content)
}
