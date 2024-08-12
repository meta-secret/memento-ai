use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;
use crate::app_type::{GROOT, JAISON, PROBIOT};

pub mod app_type {
    pub const GROOT: &str = "groot";
    pub const PROBIOT: &str = "probiot";
    pub const JAISON: &str = "jaison";
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppType {
    Groot,
    Probiot,
    Jaison,
    None,
}

#[derive(Copy, Clone, Debug)]
#[wasm_bindgen]
pub struct NervoAppType {}

#[wasm_bindgen]
impl NervoAppType {
    pub fn try_from(name: &str) -> AppType {
        match name {
            GROOT => AppType::Groot,
            PROBIOT => AppType::Probiot,
            JAISON => AppType::Jaison,
            _ => AppType::None
        }
    }
    
    pub fn get_name(app_type: AppType) -> String {
        match app_type {
            AppType::Groot => String::from(GROOT),
            AppType::Probiot => String::from(PROBIOT),
            AppType::Jaison => String::from(JAISON),
            AppType::None => String::from(""),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct SendMessageRequest {
    pub chat_id: u64,
    pub llm_message: UserLlmMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct LlmChat {
    pub chat_id: u64,
    pub messages: Vec<LlmMessage>,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub enum LlmMessageRole {
    System,
    User,
    Assistant,
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub struct LlmMessageMetaInfo {
    pub sender_id: Option<u64>,
    pub role: LlmMessageRole,
    pub persistence: LlmMessagePersistence,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct LlmMessage {
    pub meta_info: LlmMessageMetaInfo,
    pub content: LlmMessageContent,
}

impl LlmMessage {
    pub fn role_str(&self) -> String {
        match self.meta_info.role {
            LlmMessageRole::System => String::from("system"),
            LlmMessageRole::User => String::from("user"),
            LlmMessageRole::Assistant => String::from("assistant"),
        }
    }
}

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen]
pub enum LlmMessagePersistence {
    Persistent,
    Temporal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct UserLlmMessage {
    pub sender_id: u64,
    pub content: LlmMessageContent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct LlmMessageContent(pub String);

#[wasm_bindgen]
impl LlmMessageContent {
    pub fn text(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for LlmMessageContent {
    fn from(content: &str) -> Self {
        LlmMessageContent(content.to_string())
    }
}
