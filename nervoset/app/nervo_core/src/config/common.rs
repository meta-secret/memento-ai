use crate::config::groot::GrootConfig;
use crate::config::jarvis::JarvisConfig;
use config::Config as AppConfig;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct NervoConfig {
    pub apps: AppsConfig,
    pub telegram: TelegramConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppsConfig {
    pub jarvis: JarvisConfig,
    pub groot: GrootConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramConfig {
    pub agent: TelegramAgent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramAgent {
    pub probiot: TelegramBotParams,
    pub w3a: TelegramBotParams,
    pub leo: TelegramBotParams,
    pub groot: TelegramBotParams,
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
