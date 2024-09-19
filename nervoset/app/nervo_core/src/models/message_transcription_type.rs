use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MessageTranscriptionType {
    Tts, // Text To Speach
    Stt, // Speach To Text
}

impl MessageTranscriptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageTranscriptionType::Tts => "Text To Speach",
            MessageTranscriptionType::Stt => "Speach To Text",
        }
    }
}
