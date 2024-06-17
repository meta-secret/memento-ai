mod utils;

use wasm_bindgen::prelude::*;
use log::{info, Level};
use reqwest::{Client};


#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct ApiUrl {
    url: &'static str,
    port: u32 
}

#[wasm_bindgen]
impl ApiUrl {
    pub fn dev(port: u32) -> Self {
        ApiUrl {
            url: "http://nervoset.metaelon.space",
            port
        }
    }
    
    pub fn prod() -> Self {
        ApiUrl {
            url: "https://prod.metaelon.space",
            port: 443
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
    pub api_url: ApiUrl
}

#[wasm_bindgen]
impl NervoClient {
    pub fn new(api_url: ApiUrl) -> Self {
        NervoClient {
            api_url
        }
    }
    
    pub fn configure(&self) {
        utils::set_panic_hook();
    }

    #[wasm_bindgen]
    pub async fn get_chat(&self, chat_id: String) -> Result<String, JsValue> {
        console_log::init_with_level(Level::Debug).expect("TODO: panic message");
        info!("LIB: get_chat");

        let url = format!("{}/chat/{}", self.api_url.get_url(), chat_id);
        info!("LIB: url {:?}", url);

        self.fetch_get(&url).await
    }
    
    pub async fn send_message(&self, chat_id: String, user_id: u32, role: String, content: String) -> Result<String, JsValue> {
        let json = format!(
            "{{\"chat_id\": \"{}\", \"llm_message\": {{\"sender_id\": {}, \"role\": \"{}\", \"content\": \"{}\"}}}}",
            chat_id, user_id, role, content
        );
        info!("LIB: json");

        let url = format!("{}/send_message", self.api_url.get_url());
        info!("LIB: Send msg url {:?} with json: {}", url, json);

        self.fetch_post(&url, json).await
    }

    async fn fetch_get(&self, url: &str) -> Result<String, JsValue> {
        info!("LIB: FETCH GET {:?}", url);
        let client = Client::new();
        info!("LIB: FETCH GET Client");
        let response = match client.get(url)
            .send()
            .await {
            Ok(response) => {
                info!("LIB: FETCH GET response {:?}", response);
                response
            },
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

    async fn fetch_post(&self, url: &str, json: String) -> Result<String, JsValue> {
        let client = Client::new();

        let response = match client.post(url)
            .header("Content-Type", "application/json")
            .body(json)
            .send()
            .await {
            Ok(response) => {
                info!("LIB: FETCH POST response {:?}", response);
                response
            },
            Err(error) => return Err(JsValue::from_str(&format!("Request failed: {}", error))),
        };

        if response.status().is_success() {
            info!("LIB: FETCH POST response SUCCESS");
            match response.text().await {
                Ok(body) => Ok(body),
                Err(error) => Err(JsValue::from_str(&format!("Failed to read response body: {}", error))),
            }
        } else {
            Err(JsValue::from_str(&format!("Request failed, with status code: {}", response.status())))
        }
    }
}

// Модуль для тестов
#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    // #[wasm_bindgen_test]
    // async fn test_get_chat() {
    //     let chat_id = String::from("test_chat_id");
    //     let user_id = String::from("test_user_id");
    //     let result = get_chat(chat_id, user_id).await;
    // 
    //     let expected = String::from("[{\"role\":\"System\",\"content\":\"You are a helpful assistant.\"},{\"role\":\"User\",\"content\":\"Как дела, боров?\"},{\"role\":\"Assistant\",\"content\":\"Всё отлично! Как твои?. Всё отлично! Как твои? Всё отлично! Как твои?\"},{\"role\":\"User\",\"content\":\"Загниваем! Всё ништяк!\"}]");
    // 
    //     assert_eq!(result.unwrap(), expected);
    // }

    // #[wasm_bindgen_test]
    // async fn test_send_message() {
    //     let chat_id = "example_chat_id".to_string();
    //     let user_id = "123456789".to_string();
    //     let role = "User".to_string();
    //     let content = "This is a sample message content.".to_string();
    // 
    //     let expected_json = r#"{"chat_id": "example_chat_id", "llm_message": {"sender_id": "123456789", "role": "User", "content": "This is a sample message content."}}"#;
    //     let result = send_message(chat_id, user_id, role, content).await;
    // 
    //     assert_eq!(result.unwrap(), expected_json);
    // }
}