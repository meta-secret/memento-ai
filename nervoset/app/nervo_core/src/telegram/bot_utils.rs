use crate::common::AppState;
use crate::models::nervo_message_model::TelegramMessage;
use crate::models::user_model::TelegramUser;
use anyhow::bail;
use async_openai::types::{ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs};
use chrono::Utc;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{
    AudioOutputFormat, AudioSpeechParameters, AudioSpeechResponseFormat,
    AudioTranscriptionParameters, AudioVoice,
};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::prelude::ChatId;
use teloxide::prelude::*;
use teloxide::types::{
    ChatKind, File, FileMeta, InputFile, MediaKind, MessageKind, ParseMode, User,
};
use teloxide::Bot;
use tokio::fs;
use tracing::info;

pub async fn chat(bot: Bot, msg: Message, app_state: Arc<AppState>) -> anyhow::Result<()> {
    info!("Start chat...");
    let mut parser = MessageParser {
        bot: &bot,
        msg: &msg,
        app_state: &app_state,
        is_voice: false,
    };

    let bot_info = &bot.get_me().await?;
    let bot_name = bot_info.clone().user.username.unwrap();
    let user = parser.parse_user().await?;

    info!("Ready to save user");
    save_user_id(app_state.clone(), user.id.to_string()).await?;

    let mut is_reply: bool = false;
    if let Some(message) = msg.clone().reply_to_message() {
        if let Some(user) = message.from() {
            if let Some(username) = user.username.clone() {
                is_reply = username == bot_name.clone()
            }
        }
    }

    let MessageKind::Common(msg_common) = &msg.kind else {
        bail!("Unsupported message content type.");
    };
    let is_text = match &msg_common.media_kind {
        MediaKind::Text(_media_text) => true,
        _ => false,
    };
    if matches!(&msg.chat.kind, ChatKind::Private(_))
        || is_reply
        || (matches!(&msg.chat.kind, ChatKind::Public(_)) && is_text)
    {
        let text = parser.parse_message().await?;
        if (text.contains(&bot_name)) || is_reply || matches!(&msg.chat.kind, ChatKind::Private(_))
        {
            let _ = &bot
                .send_chat_action(msg.chat.id.clone(), teloxide::types::ChatAction::Typing)
                .await?;

            if text.is_empty() {
                bot.send_message(msg.chat.id, "Please provide a message to send.")
                    .reply_to_message_id(msg.id.clone())
                    .await?;

                return Ok(());
            }

            let mut username: &str = "";
            let message_text = text;
            let user_msg = ChatCompletionRequestUserMessageArgs::default()
                .content(message_text.clone())
                .build()?;

            let is_moderation_passed = app_state.nervo_llm.moderate(&message_text).await?;
            info!("Is moderation passed {:?}", is_moderation_passed);
            if is_moderation_passed {
                let tg_message = TelegramMessage {
                    id: msg.chat.id.0 as u64,
                    message: message_text.clone(),
                    timestamp: Utc::now().naive_utc(),
                };

                if let Some(name) = &user.username {
                    username = name;
                    app_state
                        .local_db
                        .save_message(tg_message, username, true)
                        .await?;
                }

                chat_gpt_conversation(
                    &bot,
                    &msg,
                    username,
                    msg.chat.id,
                    &app_state,
                    user_msg,
                    parser.is_voice,
                )
                .await?
            } else {
                if let Some(name) = &user.username {
                    username = name;
                    let question = format!("I have a message from the user, I know the message is unacceptable, can you please read the message and reply that the message is not acceptable. Reply using the same language the massage uses. Here is the message: {:?}", &message_text);
                    let question_msg = ChatCompletionRequestUserMessageArgs::default()
                        .content(question)
                        .build()?;
                    chat_gpt_conversation(
                        &bot,
                        &msg,
                        &username,
                        msg.chat.id,
                        &app_state,
                        question_msg,
                        parser.is_voice,
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

pub async fn chat_gpt_conversation(
    bot: &Bot,
    message: &Message,
    username: &str,
    chat_id: ChatId,
    app_state: &Arc<AppState>,
    msg: ChatCompletionRequestUserMessage,
    is_voice: bool,
) -> anyhow::Result<()> {
    info!("Start GPt conversation: username {:?} chat_id {:?}", &username, chat_id );
    let reply = app_state
        .nervo_llm
        .chat(username, msg, &app_state.local_db)
        .await?
        .unwrap_or(String::from("I'm sorry, internal error."));

    if is_voice {
        info!("Send voice answer");
        create_speech(&bot, reply.clone(), chat_id, &app_state).await;
    } else {
        info!("Send to chat_id {:?} text answer {:?}", chat_id, &reply);
        bot.send_message(chat_id, reply)
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
            info!("ERROR: {:?}", err);
            let _ = bot.send_message(chat_id.clone(), err.to_string()).await;
        }
    }
}

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
    pub async fn parse_user(&mut self) -> anyhow::Result<User> {
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("Unsupported message content type.");
        };

        let Some(user) = &msg_common.from else {
            bail!("User not found. We can handle only direct messages.");
        };

        Ok(user.clone())
    }

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
                    info!("ERROR {:?}", err.to_string());
                    Err(anyhow::Error::msg(err.to_string()))
                }
            }
        } else {
            info!("File '{}' doesn't exist.", file_path);
            Err(anyhow::Error::msg(format!(
                "File '{}' doesn't exist.",
                file_path
            )))
        }
    }
}

pub async fn system_message(
    bot: &Bot,
    msg: &Message,
    message_type: SystemMessage,
) -> anyhow::Result<()> {
    let introduction_msg = message_type.as_str();
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
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemMessage::Start => "Привет, я чат-бот, заряженный мощью ИИ, спроси у меня что угодно на любом языке... даже голосом!
    Нажми /manual чтобы получить инструкцию по взаимодействию со мной.
    Нажми /cancel чтобы дать мне отдохнуть.
    Возникли вопросы? Мы поможем: @night_intelligence_in_action
    ______________________________________
    Hi, I am a chatbot charged with the power of AI. I will hold a conversation with you. Ask me anything... even by voice!
    Send /manual to get manual.
    Send /cancel to send me to sleep.
    Any questions? Contact us: @night_intelligence_in_action",
            SystemMessage::Manual => "Итак, давай я расскажу, что я умею:
            Ты можешь общаться со мной обычными текстовыми сообщениями… неожиданно, да?)
            Если предпочитаешь голосовые сообщения - я буду только рад, отвечу тем же способом.
            Бывают случаи, когда тебе удобно печатать, но ты хочешь что бы я отвечал голосом - без проблем, просто добавь к сообщению слово “голосом”, например: “Мне нужен классический рецепт ризотто с белыми грибами голосом”.
            Ты можешь добавить меня в свой групповой чат, и его участники смогут общаться со мной публично.
            Для меня не существует языковых границ, я поддержу беседу как на русском, так и на английском, французском и многих других языках.
            Я предпочитаю не общаться на деструктивные темы, я знаю много интересных и полезных вещей - можешь убедиться в этом, задав любой вопрос на любую тематику.
            ☝️В скором времени я научусь генерировать изображения по описанию, хранить с твоего доверия и обрабатывать по твоем запросу предоставленную мне информацию, предоставлять доступ к open-source моделям и многое другое!
            Обо всех обновлениях и ходе разработки ты можешь читать в нашей группе: https://t.me/night_intelligence, welcome!
            Готов помочь тебе с любым вопросом 24/7. 🤝",
        }
    }
}

async fn save_user_id(app_state: Arc<AppState>, user_id: String) -> anyhow::Result<()> {
    let user_ids = load_user_ids(&app_state).await?;

    let contains_id = user_ids.iter().any(|user| user.id == user_id);
    info!("user {:?} exists = {:?}", user_id, contains_id);
    if !contains_id {
        let user = TelegramUser{
            id: user_id.parse().unwrap(),
        };
        app_state
            .local_db
            .save_message(user, "all_users_list", false)
            .await?;
    }
    Ok(())
}

async fn load_user_ids(app_state: &AppState) -> anyhow::Result<Vec<TelegramUser>> {
    match app_state.local_db.read_messages("all_users_list").await {
        Ok(ids) => {
            Ok(ids)
        },
        Err(_err) => {
            Ok(Vec::new())
        },
    }
}
