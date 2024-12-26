use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FeatureToggle {
    pub rag_crap_request_method: bool,
    pub rag_related_points: bool,
    pub permanent_memory: bool,
    pub localization: bool,
}
