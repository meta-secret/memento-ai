use std::sync::Arc;
use anyhow::{anyhow, bail};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{AudioOutputFormat, AudioTranscriptionFile, AudioTranscriptionParameters};
use teloxide::Bot;
use teloxide::net::Download;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::{File, FileMeta, MediaKind, MessageId, MessageKind, ReplyParameters, User};
use tokio::fs;
use tracing::info;
use crate::config::jarvis::JarvisAppState;

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
            bail!("COMMON: User not found. We can handle only direct messages.");
        };
        Ok(user.clone())
    }

    // Get text from TG message
    pub async fn parse_text(&mut self, is_editing: bool) -> anyhow::Result<String> {
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("COMMON: Unsupported message content type.");
        };

        let media_kind = &msg_common.media_kind;

        let result_text = match media_kind {
            MediaKind::Text(media_text) => media_text.text.clone(),
            MediaKind::Voice(media_voice) => {
                info!("COMMON: MediaKind - Voice");
                let text = self.parse_voice_to_text(&media_voice.voice.file, is_editing).await?;
                text.clone()
            }
            MediaKind::Audio(media_voice) => {
                info!("COMMON: MediaKind - Audio");
                let text = self.parse_voice_to_text(&media_voice.audio.file, is_editing).await?;
                text.clone()
            }
            _ => {
                bail!("COMMON: Unsupported case. We can handle only direct messages.");
            }
        };

        info!("COMMON: Text from the message: {}", result_text);
        Ok(result_text)
    }

    // Get voice from TG message
     pub async fn parse_voice_to_text(
        &mut self,
        media_voice: &FileMeta,
        is_editing: bool
    ) -> anyhow::Result<String> {
        info!("COMMON: Parse voice to text");
        let reply_parameters = ReplyParameters {
            message_id: self.msg.id,
            chat_id: None,
            allow_sending_without_reply: None,
            quote: None,
            quote_parse_mode: None,
            quote_entities: None,
            quote_position: None,
        };
        
        if !is_editing {
            self.bot.send_message(
                    self.msg.chat.id.clone(),
                    "Один момент, сейчас отвечу!".to_string(),
                )
                .reply_parameters(reply_parameters)
                .await?;
        } else {
            self.bot.send_chat_action(self.msg.chat.id.clone(), teloxide::types::ChatAction::Typing).await?;
        }

        info!("COMMON: Generate audio message");
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

            let client = Client::new(self.app_state.nervo_llm.api_key().to_string());
            let response = client.audio().create_transcription(parameters).await;
            
            fs::remove_file(&file_path).await?;
            drop(dst);

            match response {
                Ok(text) => {
                    self.set_is_voice(true);
                    Ok(text.clone())
                }
                Err(err) => {
                    Err(anyhow!(err).context("Can't transcribe audio file to text"))
                }
            }
        } else {
            let error = anyhow!(format!(
                "COMMON: File '{}' doesn't exist.",
                file_path
            ));
            Err(error)
        }
    }
}