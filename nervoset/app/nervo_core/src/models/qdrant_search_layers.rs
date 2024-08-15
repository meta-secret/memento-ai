use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QdrantSearchInfo {
    pub crap_detecting_layer: QdrantSearchLayer,
    pub layers: Vec<QdrantSearchLayer>,
    pub info_message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QdrantSearchLayer {
    pub index: i64,
    pub user_role_params: Vec<QdrantUserRoleParameters>,
    pub system_role_text: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub common_token_limit: u32,
    pub vectors_limit: u64,
    pub layer_for_search: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QdrantUserRoleParameters {
    pub param_type: QdrantUserRoleTextType,
    pub param_value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum QdrantUserRoleTextType {
    History,
    UserPrompt,
    RephrasedPrompt,
    DbSearch,
}
