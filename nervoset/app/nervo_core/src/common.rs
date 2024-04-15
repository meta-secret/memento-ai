use crate::ai::ai_db::NervoAiDb;
use crate::ai::nervo_llm::NervoLlm;
use crate::db::local_db::LocalDb;
use serde::Deserialize;

/// Application state
pub struct AppState {
    pub nervo_llm: NervoLlm,
    pub nervo_ai_db: NervoAiDb,
    pub local_db: LocalDb,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NervoConfig {
    pub openai_api_key: String,
    pub model_name: String,
    pub max_tokens: u16,
    pub temperature: f32,

    pub qdrant_server_url: String,
    pub qdrant_api_key: String,

    pub telegram_bot_token: String,

    pub database_url: String,
}
