use crate::ai::nervo_llm::{NervoLlm, NervoLlmConfig};
use crate::config::common::{DatabaseParams, QdrantParams};
use serde_derive::Deserialize;
use crate::ai::ai_db::NervoAiDb;
use crate::db::local_db::LocalDb;

#[derive(Debug, Clone, Deserialize)]
pub struct JarvisConfig {
    pub llm: NervoLlmConfig,
    pub qdrant: QdrantParams,
    pub database: DatabaseParams,
}

/// Application state
pub struct JarvisAppState {
    pub nervo_llm: NervoLlm,
    pub nervo_ai_db: NervoAiDb,
    pub local_db: LocalDb,
    pub nervo_config: JarvisConfig,
}

impl TryFrom<JarvisConfig> for JarvisAppState {
    type Error = anyhow::Error;

    fn try_from(nervo_config: JarvisConfig) -> Result<Self, Self::Error> {
        let nervo_llm = NervoLlm::from(nervo_config.llm.clone());
        let nervo_ai_db = NervoAiDb::build(&nervo_config.qdrant, nervo_llm.clone())?;
        let local_db = LocalDb::try_init(nervo_config.database.clone())?;

        Ok(Self {
            nervo_llm,
            nervo_ai_db,
            local_db,
            nervo_config,
        })
    }
}
