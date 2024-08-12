use serde_derive::{Deserialize, Serialize};
use async_openai::types::Embedding;

/// Migration model represent a data sample that we manage, 
/// it used to do data versioning and migrate a record from one version to another.
/// Also, it represents a json file in dataset directory. 
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationModel {
    pub delete: Vec<DataSample>,
    pub create: DataSample
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSample {
    pub text: String,
    pub embedding: Option<Embedding>
}