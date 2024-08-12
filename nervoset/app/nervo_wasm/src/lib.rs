use log::{info};
use nervo_api::{LlmChat, LlmMessage, LlmMessageContent, SendMessageRequest, UserLlmMessage};
use reqwest::Client;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

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
    client: Client,
}

#[wasm_bindgen]
impl NervoClient {
    pub fn new(api_url: ApiUrl) -> Self {
        NervoClient {
            api_url,
            client: Client::new(),
        }
    }

    pub fn configure(&self) {
        utils::set_panic_hook();
    }

    #[wasm_bindgen]
    pub async fn get_chat(&self, chat_id: u64) -> Result<LlmChat, JsValue> {
        // console_log::init_with_level(Level::Debug).expect("TODO: panic message");
        info!("LIB: get_chat");

        let url = format!("{}/chat/{}", self.api_url.get_url(), chat_id);
        info!("LIB: url {:?}", url);

        let response = match self.client.get(url).send().await {
            Ok(response) => {
                info!("LIB: FETCH GET response {:?}", response);
                response
            }
            Err(error) => return Err(JsValue::from_str(&format!("Request failed: {}", error))),
        };

        let json: LlmChat = response.json().await.map_err(JsError::from)?;

        Ok(json)
    }

    pub async fn send_message(
        &self,
        chat_id: u64,
        user_id: u64,
        content: String,
    ) -> Result<LlmMessage, JsValue> {
        let json = SendMessageRequest {
            chat_id,
            llm_message: UserLlmMessage {
                sender_id: user_id,
                content: LlmMessageContent(content),
            },
        };

        let url = format!("{}/send_message", self.api_url.get_url());
        info!("LIB: Send msg url {:?} with json: {:?}", url, json);

        let response = self
            .client
            .post(url.clone())
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", url)
            .json(&json)
            .send()
            .await
            .map_err(JsError::from)?;

        let json: LlmMessage = response.json().await.map_err(JsError::from)?;

        Ok(json)
    }
}
