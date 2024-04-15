use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelegramUser {
    pub id: u64,
}