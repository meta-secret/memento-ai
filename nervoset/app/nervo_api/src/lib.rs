use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub chat_id: u64,
    pub llm_message: UserLlmMessage,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmChat {
    pub chat_id: u64,
    pub messages: Vec<LlmMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmMessage {
    pub save_to_context: LlmSaveContext,
    pub message_owner: LlmOwnerType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LlmSaveContext {
    True,
    False,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LlmOwnerType {
    System(LlmMessageContent),
    User(UserLlmMessage),
    Assistant(LlmMessageContent),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserLlmMessage {
    pub sender_id: u64,
    pub content: LlmMessageContent,
}

impl LlmMessage {
    pub fn role(&self) -> String {
        match self.message_owner {
            LlmOwnerType::System(_) => String::from("system"),
            LlmOwnerType::User(_) => String::from("user"),
            LlmOwnerType::Assistant(_) => String::from("assistant")
        }
    }

    pub fn content_text(&self) -> String {
        match &self.message_owner {
            LlmOwnerType::System(LlmMessageContent(content)) => content.clone(),
            LlmOwnerType::User(UserLlmMessage { content: LlmMessageContent(text), .. }) => text.clone(),
            LlmOwnerType::Assistant(LlmMessageContent(content)) => content.clone(),
        }
    }

    pub fn content(&self) -> LlmMessageContent {
        match &self.message_owner {
            LlmOwnerType::System(content) => content.clone(),
            LlmOwnerType::User(user) => user.content.clone(),
            LlmOwnerType::Assistant(content) => content.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmMessageContent(pub String);

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

