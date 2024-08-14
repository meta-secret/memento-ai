use config::Config as AppConfig;
use serde::Deserialize;

use crate::config::groot::GrootConfig;
use crate::config::leo::LeoConfig;
use crate::config::nervo_server::NervoServerConfig;
use crate::config::probiot::ProbiotConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct NervoConfig {
    pub nervo_server: NervoServerConfig,
    pub probiot: ProbiotConfig,
    pub groot: GrootConfig,
    pub leo: LeoConfig,
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
        let config_file = config::File::with_name("config")
            .format(config::FileFormat::Yaml);

        let app_config = AppConfig::builder()
            .add_source(config_file)
            .build()?;

        let cfg = app_config.try_deserialize()?;

        Ok(cfg)
    }
}
