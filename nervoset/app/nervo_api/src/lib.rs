use crate::agent_type::AgentType;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod errors;

pub mod app_type {
    use serde_derive::{Deserialize, Serialize};
    use wasm_bindgen::prelude::wasm_bindgen;
    use crate::errors::{NervoWebError, NervoWebResult};

    pub const GROOT: &str = "groot";
    pub const JARVIS: &str = "jarvis";

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum AppType {
        Groot,
        Jarvis
    }

    #[derive(Copy, Clone, Debug)]
    #[wasm_bindgen]
    pub struct NervoAppType {}

    #[wasm_bindgen]
    impl NervoAppType {
        pub fn try_from(name: &str) -> NervoWebResult<AppType> {
            match name {
                GROOT => Ok(AppType::Groot),
                JARVIS => Ok(AppType::Jarvis),
                _ => Err(NervoWebError::UnknownAppTypeError(name.to_string()))
            }
        }

        pub fn get_name(app_type: AppType) -> String {
            match app_type {
                AppType::Groot => String::from(GROOT),
                AppType::Jarvis => String::from(JARVIS)
            }
        }
    }
}

pub mod agent_type {
    use enum_iterator::Sequence;
    use serde_derive::{Deserialize, Serialize};
    use wasm_bindgen::prelude::wasm_bindgen;

    pub const PROBIOT: &str = "probiot";
    pub const W3A: &str = "w3a";
    pub const LEO: &str = "leo";
    pub const GROOT: &str = "groot";
    pub const NERVOZNYAK: &str = "nervoznyak";

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Sequence)]
    #[serde(rename_all = "camelCase")]
    pub enum AgentType {
        Probiot,
        W3a,
        Leo,
        Groot,
        Nervoznyak,
        None,
    }

    #[derive(Copy, Clone, Debug)]
    #[wasm_bindgen]
    pub struct NervoAgentType {}

    #[wasm_bindgen]
    impl NervoAgentType {
        pub fn try_from(name: &str) -> AgentType {
            match name {
                PROBIOT => AgentType::Probiot,
                W3A => AgentType::W3a,
                LEO => AgentType::Leo,
                GROOT => AgentType::Groot,
                NERVOZNYAK => AgentType::Nervoznyak,
                _ => AgentType::None,
            }
        }

        pub fn get_name(agent_type: AgentType) -> String {
            match agent_type {
                AgentType::Probiot => String::from(PROBIOT),
                AgentType::W3a => String::from(W3A),
                AgentType::Leo => String::from(LEO),
                AgentType::Groot => String::from(GROOT),
                AgentType::Nervoznyak => String::from(NERVOZNYAK),
                AgentType::None => String::from(""),
            }
        }
    }
}

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
