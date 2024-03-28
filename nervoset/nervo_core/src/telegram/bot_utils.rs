
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use anyhow::bail;
use async_openai::types::{ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs};
use chrono::Utc;
use teloxide::prelude::ChatId;
use teloxide::{prelude::*, repl};
use teloxide::types::{ChatKind, File, FileMeta, InputFile, MediaKind, MessageKind, ParseMode, User};
use teloxide::Bot;
use teloxide::net::Download;
use tokio::fs;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{AudioOutputFormat, AudioSpeechParameters, AudioSpeechResponseFormat, AudioTranscriptionParameters, AudioVoice};
use tokio::sync::Mutex;

use crate::common::AppState;
use crate::db::nervo_message_model::TelegramMessage;

pub async fn chat(bot: Bot, msg: Message, app_state: Arc<AppState>) -> anyhow::Result<()> {
    let mut parser = MessageParser {
        bot: &bot,
        msg: &msg,
        app_state: &app_state,
        is_voice: false,
    };

    let bot_info = &bot.get_me().await?;
    let bot_name = bot_info.clone().user.username.unwrap();
    let (user, text) = parser.user_and_text().await?;

    let mut is_reply: bool = false;
    if let Some(message) = msg.clone().reply_to_message() {
        if let Some(user) = message.from() {
            if let Some(username) = user.username.clone() {
                is_reply = username == bot_name.clone()
            }
        }
    }

    if text.contains(&bot_name) || matches!(&msg.chat.kind, ChatKind::Private(_)) || is_reply  {
        let _ = &bot.send_chat_action(msg.chat.id.clone(), teloxide::types::ChatAction::Typing).await?;

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
                    .save_message(tg_message, username)
                    .await?;
            }

            chat_gpt_conversation(&bot, &msg, username, msg.chat.id, &app_state, user_msg, parser.is_voice).await
        } else {
            if let Some(name) = &user.username {
                username = name;
                let question = format!("I have a message from the user, I know the message is unacceptable, can you please read the message and reply that the message is not acceptable. Reply using the same language the massage uses. Here is the message: {:?}", &message_text);
                let question_msg = ChatCompletionRequestUserMessageArgs::default()
                    .content(question)
                    .build()?;
                chat_gpt_conversation(&bot, &msg, &username, msg.chat.id, &app_state, question_msg, parser.is_voice).await
            } else {
                Ok(())
            }
        }
    } else {
        return Ok(())
    }
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
    let reply = app_state
        .nervo_llm
        .chat(username, msg, &app_state.local_db)
        .await?
        .unwrap_or(String::from("I'm sorry, internal error."));

    if is_voice {
        create_speech(&bot, reply.clone(), chat_id, &app_state).await;
    } else {
        bot.send_message(chat_id, reply)
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
        voice: AudioVoice::Alloy,
        response_format: Some(AudioSpeechResponseFormat::Mp3),
        speed: Some(1.0),
    };

    let response = client.audio().create_speech(parameters).await;
    match response {
        Ok(audio) => {
            // stop_loop();
            let input_file = InputFile::memory(audio.bytes);
            let _ = &bot.send_voice(chat_id.clone(), input_file).await;
        },
        Err(err) => {
            println!("ERROR: {:?}", err);
            &bot.send_message(chat_id.clone(), err.to_string()).await;
        },
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

impl <'a> MessageParser<'a> {
    pub async fn user_and_text(&mut self) -> anyhow::Result<(User, String)> {
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("Unsupported message content type.");
        };

        let Some(user) = &msg_common.from else {
            bail!("User not found. We can handle only direct messages.");
        };

        let mut result_text = String::new();
        let media_kind = &msg_common.media_kind;

        match media_kind {
            MediaKind::Text(media_text) => {
                result_text = media_text.text.clone()
            }
            MediaKind::Voice(media_voice) => {
                let (_, text) = self.user_and_voice(&media_voice.voice.file, &user).await?;
                result_text = text.clone();
            }
            MediaKind::Audio(media_voice) => {
                let (_, text) = self.user_and_voice(&media_voice.audio.file, &user).await?;
                result_text = text.clone();
            }
            _ => {
                bail!("Unsupported case. We can handle only direct messages.");
            }
        }

        Ok((user.clone(), result_text.clone()))
    }

    pub async fn user_and_voice(&mut self, media_voice: &FileMeta, user: &User) -> anyhow::Result<(User, String)> {
        self.bot.send_message(self.msg.chat.id.clone(), "Один момент, сейчас отвечу!".to_string())
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

            // send_typing_indicator(&self.bot, self.msg.chat.id).await;
            let client = Client::new(self.app_state.nervo_llm.api_key().to_string());
            let response = client
                .audio()
                .create_transcription(parameters)
                .await;

            fs::remove_file(&file_path).await?;
            drop(dst);

            match response {
                Ok(text) => {
                    self.set_is_voice(true);
                    Ok((user.clone(), text.clone()))
                },
                Err(err) => {
                    println!("ERROR {:?}", err.to_string());
                    Err(anyhow::Error::msg(err.to_string()))
                },
            }

        } else {
            println!("Файл '{}' не существует.", file_path);
            Err(anyhow::Error::msg(format!("Файл '{}' не существует.", file_path)))
        }
    }
}