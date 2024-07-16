use serde_derive::{Deserialize, Serialize};
use nervo_bot_core::ai::nervo_llm::LlmMessage;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub chat_id: String,
    pub llm_message: LlmMessage,
}