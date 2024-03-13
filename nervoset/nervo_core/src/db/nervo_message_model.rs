use chrono::NaiveDateTime;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramMessage {
    pub id: u64,
    pub message: String,
    pub timestamp: NaiveDateTime,
}
