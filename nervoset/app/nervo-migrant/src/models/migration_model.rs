use async_openai::types::Embedding;
use serde_derive::{Deserialize, Serialize};

/// Migration model represent a data sample that we manage,
/// it used to do data versioning and migrate a record from one version to another.
/// Also, it represents a json file in dataset directory.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MigrationModel {
    pub delete: Vec<DataSample>,
    pub create: DataSample,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataSample {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<VectorData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VectorData {
    pub embedding_model_name: Option<String>,
    pub embedding: Embedding,
}
