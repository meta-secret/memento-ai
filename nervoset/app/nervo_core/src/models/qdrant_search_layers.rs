use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QDrantSearchInfo {
    pub crap_detecting_layer: QDrantSearchLayer,
    pub layers: Vec<QDrantSearchLayer>,
    pub info_message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QDrantSearchLayer {
    pub index: i64,
    pub user_role_params: Vec<QDrantUserRoleParameters>,
    pub system_role_text: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub collection_names: Vec<String>,
    pub collections_tokens_limit: Vec<i64>,
    pub vectors_limit: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QDrantUserRoleParameters {
    pub param_type: QDrantUserRoleTextType,
    pub param_value: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum QDrantUserRoleTextType {
    History,
    UserPromt,
    RephrasedPromt,
    DBSearch,
}
