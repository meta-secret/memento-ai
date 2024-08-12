use serde_derive::Deserialize;
use crate::ai::nervo_llm::NervoLlmConfig;
use crate::config::common::{DatabaseParams, QdrantParams, TelegramBotParams};

#[derive(Debug, Clone, Deserialize)]
pub struct ProbiotConfig {
    pub telegram: TelegramBotParams,
    pub llm: NervoLlmConfig,
    pub qdrant: QdrantParams,
    pub database: DatabaseParams,
}
