use crate::ai::ai_db::NervoAiDb;
use crate::ai::nervo_llm::NervoLlm;
use serde::Deserialize;

pub struct AppState {
    pub nervo_llm: NervoLlm,
    pub nervo_ai_db: NervoAiDb,
}

#[derive(Debug, Deserialize)]
pub struct NervoConfig {
    pub openai_api_key: String,
    pub qdrant_server_url: String,
    pub qdrant_api_key: String,
    pub telegram_bot_token: String
}