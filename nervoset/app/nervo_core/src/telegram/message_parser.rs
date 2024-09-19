use crate::config::jarvis::JarvisAppState;
use anyhow::{anyhow, bail};
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::audio::{
    AudioOutputFormat, AudioTranscriptionFile, AudioTranscriptionParameters,
};
use std::sync::Arc;
use teloxide::net::Download;
use teloxide::prelude::{Message, Requester};
use teloxide::types::{File, FileMeta, MediaKind, MessageKind, User};
use teloxide::Bot;
use tokio::fs;
use tracing::info;

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

    // Detect if TG message is text
    pub async fn is_tg_message_text(&self) -> anyhow::Result<bool> {
        let media_kind = self.get_message_media_kind().await?;
        let is_text = matches!(media_kind, MediaKind::Text(_));

        info!("Сurrent message is text: {}", is_text);
        Ok(is_text)
    }
    pub async fn parse_tg_message_content(&mut self) -> anyhow::Result<String> {
        let media_kind = self.get_message_media_kind().await?;

        let result_text = match media_kind {
            MediaKind::Text(media_text) => {
                info!("Your message is Text");
                media_text.text.clone()
            }

            MediaKind::Voice(media_voice) => {
                info!("Your message is Voice");
                let text = self.parse_voice_to_text(&media_voice.voice.file).await?;
                text.clone()
            }
            MediaKind::Audio(media_voice) => {
                info!("Your message is Audio");
                let text = self.parse_voice_to_text(&media_voice.audio.file).await?;
                text.clone()
            }
            _ => {
                bail!("Unsupported case. We can handle only direct messages.");
            }
        };

        info!("Text from your message: {}", result_text);
        Ok(result_text)
    }

    // Get voice from TG message
    async fn parse_voice_to_text(&mut self, media_voice: &FileMeta) -> anyhow::Result<String> {
        info!("Start parsing voice to text");
        let file: File = self.bot.get_file(&media_voice.id).await?;
        let file_path = self.get_file_path_from(&file).await?;
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
                    info!("Parsing voice to text are success");
                    Ok(text.clone())
                }
                Err(err) => Err(anyhow!(err).context("Can't transcribe audio file to text")),
            }
        } else {
            let error = anyhow!(format!("File '{}' doesn't exist.", file_path));
            Err(error)
        }
    }

    async fn get_file_path_from(&self, file: &File) -> anyhow::Result<String> {
        let file_extension = "oga";
        let file_name: &str = &file.id;
        Ok(format!("/tmp/{}.{}", &file_name, &file_extension))
    }

    async fn get_message_media_kind(&self) -> anyhow::Result<MediaKind> {
        let MessageKind::Common(msg_common) = &self.msg.kind else {
            bail!("Unsupported message content type.");
        };
        Ok(msg_common.media_kind.clone())
    }
}
