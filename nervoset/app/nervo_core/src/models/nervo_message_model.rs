use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TelegramMessage {
    pub id: u64,
    pub message: String,
    pub timestamp: NaiveDateTime,
}
