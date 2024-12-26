use crate::utils::cryptography::{U64Generator, UuidGenerator};
use wasm_bindgen::prelude::wasm_bindgen;

pub mod api;
pub mod common;
pub mod errors;
pub mod utils;

#[wasm_bindgen]
pub struct WasmIdGenerator {}

#[wasm_bindgen]
impl WasmIdGenerator {
    pub fn generate_uuid() -> String {
        UuidGenerator::rand_uuid_b64_url_enc().text()
    }

    pub fn generate_u64() -> u64 {
        U64Generator::generate_u64()
    }
}

pub mod app_type {
    use crate::errors::{NervoSdkError, NervoWebResult};
    use serde_derive::{Deserialize, Serialize};
    use wasm_bindgen::prelude::wasm_bindgen;

    pub const GROOT: &str = "groot";
    pub const JARVIS: &str = "jarvis";

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum AppType {
        Groot,
        Jarvis,
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
                _ => Err(NervoSdkError::UnknownAppTypeError(name.to_string())),
            }
        }

        pub fn get_name(app_type: AppType) -> String {
            match app_type {
                AppType::Groot => String::from(GROOT),
                AppType::Jarvis => String::from(JARVIS),
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
    pub const GROOT: &str = "groot";
    pub const NERVOZNYAK: &str = "nervoznyak";
    pub const KEVIN: &str = "kevin";

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Sequence)]
    #[serde(rename_all = "camelCase")]
    pub enum AgentType {
        Probiot,
        W3a,
        Groot,
        Nervoznyak,
        None,
        Kevin,
    }

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Sequence)]
    #[serde(rename_all = "camelCase")]
    pub enum AgentPersonality {
        Saylor,
        None,
    }
    
    impl AgentPersonality {
        pub fn get_name(&self) -> String {
            match self {
                AgentPersonality::Saylor => "saylor".to_string(),
                AgentPersonality::None => "none".to_string(),
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    #[wasm_bindgen]
    pub struct NervoAgentType {
        pub agent_type: AgentType,
        pub agent_personality: AgentPersonality,
    }

    #[wasm_bindgen]
    impl NervoAgentType {
        pub fn try_from(name: &str) -> NervoAgentType {
            match name {
                PROBIOT => {
                    NervoAgentType {
                        agent_type: AgentType::Probiot,
                        agent_personality: AgentPersonality::None,
                    }
                },
                
                W3A => {
                    NervoAgentType {
                        agent_type: AgentType::W3a,
                        agent_personality: AgentPersonality::None,
                    }
                },
                GROOT => {
                    NervoAgentType {
                        agent_type: AgentType::Groot,
                        agent_personality: AgentPersonality::None,
                    }
                },
                NERVOZNYAK => {
                    NervoAgentType {
                        agent_type: AgentType::Nervoznyak,
                        agent_personality: AgentPersonality::None,
                    }
                },
                KEVIN => {
                    NervoAgentType {
                        agent_type: AgentType::Kevin,
                        agent_personality: AgentPersonality::None,
                    }
                },
                _ => {
                    NervoAgentType {
                        agent_type: AgentType::None,
                        agent_personality: AgentPersonality::None,
                    }
                },
            }
        }

        pub fn get_name(agent_type: AgentType) -> String {
            match agent_type {
                AgentType::Probiot => String::from(PROBIOT),
                AgentType::W3a => String::from(W3A),
                AgentType::Groot => String::from(GROOT),
                AgentType::Nervoznyak => String::from(NERVOZNYAK),
                AgentType::Kevin => String::from(KEVIN),
                AgentType::None => String::from(""),
            }
        }
    }
}
