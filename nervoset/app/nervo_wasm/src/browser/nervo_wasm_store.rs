use wasm_bindgen::prelude::*;
use nervo_sdk::WasmIdGenerator;
use crate::db::wasm_repo::WasmRepo;

#[wasm_bindgen]
pub struct NervoWasmStore {
    db: WasmRepo
}

#[wasm_bindgen]
impl NervoWasmStore {
    
    pub async fn init() -> Self {
        Self {
            db: WasmRepo::init().await,
        }
    }

    pub async fn get_or_generate_user_id(&self) -> u64 {
        let maybe_user_id = self.db.get("userId").await;

        match maybe_user_id {
            Some(user_id_str) => user_id_str.parse::<u64>().unwrap(),
            None => {
                let user_id = WasmIdGenerator::generate_u64();
                let user_id_str = user_id.to_string();
                self.db.put("userId", user_id_str.as_str()).await;
                user_id
            }
        }
    }

    pub async fn get_or_generate_chat_id(&self) -> u64 {
        let maybe_chat_id = self.db.get("chatId").await;
        if let Some(chat_id) = maybe_chat_id {
            chat_id.parse::<u64>().unwrap()
        } else {
            let chat_id = WasmIdGenerator::generate_u64();
            let chat_id_str = chat_id.to_string();
            self.db.put("chatId", chat_id_str.as_str()).await;
            chat_id
        }
    }
}
