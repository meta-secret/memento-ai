use std::path::PathBuf;
use serde_derive::{Deserialize, Serialize};
use nervo_api::{AppType, NervoAppType};
use crate::models::migration_model::MigrationModel;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationPlan {
    pub app_type: AppType,
    pub data_models: Vec<MigrationMetaData>,
}

impl MigrationPlan {
    pub fn app_name(&self) -> String {
        NervoAppType::get_name(self.app_type)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationMetaData {
    pub json_path: PathBuf,
    pub migration_model: MigrationModel,
}
