use serde_derive::Deserialize;

use crate::ai::nervo_llm::{NervoLlm, NervoLlmConfig};
use crate::config::common::{DatabaseParams, TelegramBotParams};
use crate::db::local_db::LocalDb;

#[derive(Debug, Clone, Deserialize)]
pub struct GrootConfig {
    pub telegram: TelegramBotParams,
    pub llm: NervoLlmConfig,
    pub database: DatabaseParams,
}

/// Application state
pub struct GrootAppState {
    pub nervo_llm: NervoLlm,
    pub local_db: LocalDb,
    pub nervo_config: GrootConfig,
}

impl TryFrom<GrootConfig> for GrootAppState {
    type Error = anyhow::Error;

    fn try_from(nervo_config: GrootConfig) -> Result<Self, Self::Error> {
        let nervo_llm = NervoLlm::from(nervo_config.llm.clone());
        let local_db = LocalDb::try_init(nervo_config.database.clone())?;

        Ok(Self {
            nervo_llm,
            local_db,
            nervo_config,
        })
    }
}
