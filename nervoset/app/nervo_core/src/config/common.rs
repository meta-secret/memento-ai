use std::fs;
use crate::config::groot::GrootConfig;
use crate::config::jarvis::JarvisConfig;
use anyhow::bail;
use config::Config as AppConfig;
use grammers_client::{Client, Config};
use grammers_session::Session;
use nervo_sdk::agent_type::AgentType;
use serde::Deserialize;
use tracing::info;
use crate::utils::ai_utils::RESOURCES_DIR;

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
    pub user_agent: TelegramUserAgent,
}

impl TelegramConfig {
    pub fn agent_params(self, agent_type: AgentType) -> anyhow::Result<TelegramBotParams> {
        match agent_type {
            AgentType::Probiot => Ok(self.agent.probiot),
            AgentType::W3a => Ok(self.agent.w3a),
            AgentType::Leo => Ok(self.agent.leo),
            AgentType::Groot => Ok(self.agent.groot),
            AgentType::Nervoznyak => Ok(self.agent.nervoznyak),
            _ => bail!("Unknown agent type"),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramAgent {
    pub probiot: TelegramBotParams,
    pub w3a: TelegramBotParams,
    pub leo: TelegramBotParams,
    pub groot: TelegramBotParams,
    pub nervoznyak: TelegramBotParams,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramBotParams {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramUserAgent {
    pub groot: TelegramUserAgentParams,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramUserAgentParams {
    pub api_id: i32,
    pub api_hash: String,
    pub session_file_path: String,
}

pub struct TelegramUserAgentClient {
    pub g_client: Client,
}

impl TelegramUserAgentClient {
    pub async fn from(parameters: TelegramUserAgentParams, agent_name: String) -> anyhow::Result<Self> {
        let session_file_path = format!("{}{}/{}", RESOURCES_DIR, agent_name, parameters.session_file_path);
        
        info!("Create G config from: {}, {}, {}",
            session_file_path,
            parameters.api_id,
            parameters.api_hash.to_string()
        );
        let g_config = Config {
            session: Session::load_file_or_create(session_file_path)?,
            api_id: parameters.api_id,
            api_hash: parameters.api_hash.to_string(),
            params: Default::default(),
        };
        
        let g_client = Client::connect(g_config).await?;
        info!("Connect during creation: {:?}", g_client);

        if !g_client.is_authorized().await? {
            bail!("Achtung! G_client is not authorized!");
        } else {
            info!("All's good");
            Ok(Self { g_client })
        }
    }
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
