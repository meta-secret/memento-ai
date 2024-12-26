use serde_derive::Deserialize;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use crate::ai::nervo_llm::{NervoLlm, NervoLlmConfig};
use crate::config::common::{DatabaseParams, NervoConfig, TelegramUserAgentClient};
use crate::db::local_db::LocalDb;

#[derive(Debug, Clone, Deserialize)]
pub struct GrootConfig {
    pub llm: NervoLlmConfig,
    pub database: DatabaseParams,
}

/// Application state
pub struct GrootAppState {
    pub nervo_llm: NervoLlm,
    pub local_db: LocalDb,
    pub nervo_config: GrootConfig,
    pub telegram_user_agent_client: Option<TelegramUserAgentClient>,
}

impl TryFrom<NervoConfig> for GrootAppState {
    type Error = anyhow::Error;

    fn try_from(nervo_config: NervoConfig) -> Result<Self, Self::Error> {
        let groot_config = nervo_config.clone().apps.groot;
        let nervo_llm = NervoLlm::from(nervo_config.apps.groot.llm.clone());
        let local_db = LocalDb::try_init(nervo_config.apps.groot.database.clone())?;
    
        Ok(Self {
            nervo_llm,
            local_db,
            nervo_config: groot_config,
            telegram_user_agent_client: None,
        })
    }
}

impl GrootAppState {
    pub async fn generate_user_agent_client(&mut self, nervo_config: NervoConfig) -> anyhow::Result<()> {
        let agent_name = NervoAgentType::get_name(AgentType::Groot);
        
        let telegram_client = TelegramUserAgentClient::from(
            nervo_config.telegram.user_agent.groot,
            agent_name
        ).await?;
        self.telegram_user_agent_client.replace(telegram_client);
        Ok(())
    }
}