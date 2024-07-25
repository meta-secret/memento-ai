use crate::ai::ai_db::NervoAiDb;
use crate::ai::nervo_llm::{NervoLlm, NervoLlmConfig};
use crate::db::local_db::LocalDb;
use config::Config as AppConfig;
use serde::Deserialize;

/// Application state
pub struct AppState {
    pub nervo_llm: NervoLlm,
    pub nervo_ai_db: NervoAiDb,
    pub local_db: LocalDb,
    pub nervo_config: NervoConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NervoConfig {
    pub telegram: TelegramBotParams,
    pub llm: NervoLlmConfig,
    pub qdrant: QdrantParams,
    pub database: DatabaseParams,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramBotParams {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QdrantParams {
    pub server_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseParams {
    pub url: String,
}

impl NervoConfig {
    pub fn load() -> anyhow::Result<NervoConfig> {
        let config_file = config::File::with_name("config").format(config::FileFormat::Yaml);

        let app_config = AppConfig::builder().add_source(config_file).build()?;

        let cfg = app_config.try_deserialize()?;

        Ok(cfg)
    }
}

impl TryFrom<NervoConfig> for AppState {
    type Error = anyhow::Error;

    fn try_from(nervo_config: NervoConfig) -> Result<Self, Self::Error> {
        let nervo_llm = NervoLlm::from(nervo_config.llm.clone());
        let nervo_ai_db = NervoAiDb::try_from(&nervo_config.qdrant)?;
        let local_db = LocalDb::try_init(nervo_config.database.clone())?;

        Ok(Self {
            nervo_llm,
            nervo_ai_db,
            local_db,
            nervo_config,
        })
    }
}
