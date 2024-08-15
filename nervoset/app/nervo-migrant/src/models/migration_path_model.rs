use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use nervo_api::agent_type::AgentType;

use crate::models::migration_model::MigrationModel;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPlan {
    pub agent_type: AgentType,
    pub data_models: Vec<MigrationMetaData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationMetaData {
    pub json_path: PathBuf,
    pub migration_model: MigrationModel,
}
