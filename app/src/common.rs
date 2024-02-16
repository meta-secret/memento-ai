use crate::ai::ai_db::NervoAiDb;
use crate::ai::nervo_llm::NervoLlm;

pub struct AppState {
    pub nervo_llm: NervoLlm,
    pub nervo_ai_db: NervoAiDb
}
