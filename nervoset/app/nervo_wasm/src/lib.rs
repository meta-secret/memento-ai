use log::{info, Level};
use reqwest::Client;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use nervo_api::{LlmMessage, LlmMessageContent, SendMessageRequest, UserLlmMessage};

mod utils;

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct ApiUrl {
    url: &'static str,
    port: u32,
}

#[wasm_bindgen]
impl ApiUrl {
    pub fn dev(port: u32) -> Self {
        ApiUrl {
            url: "http://nervoset.metaelon.space",
            port,
        }
    }

    pub fn prod() -> Self {
        ApiUrl {
            url: "https://prod.metaelon.space",
            port: 443,
        }
    }
}

#[wasm_bindgen]
impl ApiUrl {
    pub fn get_url(&self) -> String {
        format!("{}:{}", self.url, self.port)
    }
}

#[wasm_bindgen]
pub struct NervoClient {
    pub api_url: ApiUrl,
    client: Client
}

#[wasm_bindgen]
impl NervoClient {
    pub fn new(api_url: ApiUrl) -> Self {
        NervoClient { api_url, client: Client::new() }
    }

    pub fn configure(&self) {
        utils::set_panic_hook();
    }

    #[wasm_bindgen]
    pub async fn get_chat(&self, chat_id: u64) -> Result<String, JsValue> {
        // console_log::init_with_level(Level::Debug).expect("TODO: panic message");
        info!("LIB: get_chat");

        let url = format!("{}/chat/{}", self.api_url.get_url(), chat_id);
        info!("LIB: url {:?}", url);

        self.fetch_get(&url).await
    }

    pub async fn send_message(&self, chat_id: u64, user_id: u64, content: String) -> Result<LlmMessage, JsValue> {
        let json = SendMessageRequest {
            chat_id,
            llm_message: UserLlmMessage { sender_id: user_id, content: LlmMessageContent(content) },
        };

        let url = format!("{}/send_message", self.api_url.get_url());
        info!("LIB: Send msg url {:?} with json: {:?}", url, json);

        self.fetch_post(&url, json).await
    }

    async fn fetch_get(&self, url: &str) -> Result<String, JsValue> {
        info!("LIB: FETCH GET {:?}", url);
        
        let response = match self.client.get(url)
            .send()
            .await {
            Ok(response) => {
                info!("LIB: FETCH GET response {:?}", response);
                response
            }
            Err(error) => return Err(JsValue::from_str(&format!("Request failed: {}", error))),
        };

        if response.status().is_success() {
            info!("LIB: FETCH GET response SUCCESS");
            match response.text().await {
                Ok(body) => Ok(body),
                Err(error) => Err(JsValue::from_str(&format!("Failed to read response body: {}", error))),
            }
        } else {
            Err(JsValue::from_str(&format!("Request failed with status code: {}", response.status())))
        }
    }

    async fn fetch_post<T, R>(&self, url: &str, json: T) -> Result<R, JsValue>
        where
            T: Serialize {
        let response = match self.client.post(url)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", url)
            .json(&json)
            .send()
            .await {
            Ok(response) => {
                info!("LIB: FETCH POST response {:?}", response);
                response
            }
            Err(error) => return Err(JsValue::from_str(&format!("Request failed: {}", error))),
        };

        if response.status().is_success() {
            info!("LIB: FETCH POST response SUCCESS");
            match response.text().await {
                Ok(body) => {
                    Ok(body)
                },
                Err(error) => Err(JsValue::from_str(&format!("Failed to read response body: {}", error))),
            }
        } else {
            Err(JsValue::from_str(&format!("Request failed, with status code: {}", response.status())))
        }
    }
}
