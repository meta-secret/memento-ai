use crate::agent_type::AgentType;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct SendMessageRequest {
    pub chat_id: u64,
    pub agent_type: AgentType,
    pub llm_message: UserLlmMessage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct LlmChat {
    pub chat_id: Option<u64>,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct UserAction {
    pub user_id: i64,
    pub action: UserActionType,
    pub username: String,
}

#[wasm_bindgen]
impl UserAction {
    #[wasm_bindgen(constructor)]
    pub fn new(user_id: i64, action: UserActionType, username: String) -> UserAction {
        UserAction {
            user_id,
            action,
            username,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserActionType {
    MainMenu,
    MiniAppInitialized,
    Start,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[wasm_bindgen(getter_with_clone)]
pub struct ServerResponse {
    pub message: String,
    pub buttons: Vec<String>,
    pub action_buttons: Vec<String>,
    pub can_input: bool,
}
