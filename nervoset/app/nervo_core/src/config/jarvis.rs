use crate::ai::ai_db::NervoAiDb;
use crate::ai::nervo_llm::{NervoLlm, NervoLlmConfig};
use crate::config::common::{DatabaseParams, QdrantParams};
use crate::db::local_db::LocalDb;
use crate::utils::localisation_parser::LocalisationManager;
use serde_derive::Deserialize;
use tokio::fs;
use tokio::sync::{RwLock};
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use crate::context::main_handler::UserContextMainHandler;
use crate::models::feature_toggle::FeatureToggle;
use crate::utils::ai_utils::RESOURCES_DIR;

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
    pub localisation_manager: RwLock<LocalisationManager>,
    pub user_context: UserContextMainHandler,
    pub feature_toggle: Option<FeatureToggle>,
}

impl TryFrom<JarvisConfig> for JarvisAppState {
    type Error = anyhow::Error;

    fn try_from(nervo_config: JarvisConfig) -> Result<Self, Self::Error> {
        let nervo_llm = NervoLlm::from(nervo_config.llm.clone());
        let nervo_ai_db = NervoAiDb::build(&nervo_config.qdrant, nervo_llm.clone())?;
        let local_db = LocalDb::try_init(nervo_config.database.clone())?;
        let localisation_manager = LocalisationManager::build(nervo_llm.clone())?;

        Ok(Self {
            nervo_llm,
            nervo_ai_db,
            local_db,
            nervo_config,
            localisation_manager: RwLock::new(localisation_manager),
            user_context: UserContextMainHandler::new(),
            feature_toggle: None,
        })
    }
}

impl JarvisAppState {
    pub async fn create_from(initial_params: InitialParams) -> anyhow::Result<Self> {
        let nervo_config = initial_params.config;
        let nervo_llm = NervoLlm::from(nervo_config.llm.clone());
        let nervo_ai_db = NervoAiDb::build(&nervo_config.qdrant, nervo_llm.clone())?;
        let local_db = LocalDb::try_init(nervo_config.database.clone())?;
        let localisation_manager = LocalisationManager::build(nervo_llm.clone())?;

        let agent_type = initial_params.agent_type;
        let agent_name = NervoAgentType::get_name(agent_type);
        let system_msg_file = format!("{}{}/feature_toggle.json", RESOURCES_DIR, agent_name);
        let json_string = fs::read_to_string(system_msg_file).await?;
        let feature_toggle: FeatureToggle = serde_json::from_str(&json_string)?;

        Ok(Self {
            nervo_llm,
            nervo_ai_db,
            local_db,
            nervo_config,
            localisation_manager: RwLock::new(localisation_manager),
            user_context: UserContextMainHandler::new(),
            feature_toggle: Some(feature_toggle),
        })
    }
}

pub struct InitialParams {
    pub config: JarvisConfig,
    pub agent_type: AgentType,
}