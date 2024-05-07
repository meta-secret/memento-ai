use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemMessages {
    pub start: String,
    pub manual: String,
}
