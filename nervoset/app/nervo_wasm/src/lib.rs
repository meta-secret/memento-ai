mod utils;

use wasm_bindgen::prelude::*;
use log::{info, Level};
use reqwest::{Client, Error, Response, StatusCode};


const API_URL: &str = "http://nervoset.metaelon.space";

#[wasm_bindgen]
pub fn configure() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub async fn get_chat(chat_id: String, user_id: String) -> Result<String, JsValue> {
    console_log::init_with_level(Level::Debug).expect("TODO: panic message");
    info!("get_chat");
    
    let url = format!("{}:3000/chat/{}", API_URL, chat_id);
    info!("#Lib url {:?}", url);
    
    fetch_get(&url).await
}

#[wasm_bindgen]
pub fn send_message(chat_id: String, user_id: String, role: String, content: String) -> String {
    let json = format!(
        "{{\"chat_id\": \"{}\", \"llm_message\": {{\"sender_id\": \"{}\", \"role\": \"{}\", \"content\": \"{}\"}}}}",
        chat_id, user_id, role, content
    );
    info!("json");
    return  json
}

async fn fetch_get(url: &str) -> Result<String, JsValue> {
    info!("#FETCH GET {:?}", url);
    let client = Client::new();
    info!("#FETCH GET Client");
    let response = match client.get(url)
        // .fetch_mode_no_cors()
        // .header("Access-Control-Allow-Origin", API_URL)
        .send()
        .await {
        Ok(response) => {
            info!("#FETCH GET response {:?}", response);
            response
        },
        Err(error) => return Err(JsValue::from_str(&format!("Request failed: {}", error))),
    };

    if response.status().is_success() {
        info!("#FETCH GET response SUCCESS");
        match response.text().await {
            Ok(body) => Ok(body),
            Err(error) => Err(JsValue::from_str(&format!("Failed to read response body: {}", error))),
        }
    } else {
        Err(JsValue::from_str(&format!("Request failed with status code: {}", response.status())))
    }
}

// Модуль для тестов
#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_get_chat() {
    //     let chat_id = String::from("test_chat_id");
    //     let user_id = String::from("test_user_id");
    //     let result = get_chat(chat_id, user_id);
    // 
    //     let expected = String::from("[{\"role\":\"System\",\"content\":\"You are a helpful assistant.\"},{\"role\":\"User\",\"content\":\"Как дела, боров?\"},{\"role\":\"Assistant\",\"content\":\"Всё отлично! Как твои?. Всё отлично! Как твои? Всё отлично! Как твои?\"},{\"role\":\"User\",\"content\":\"Загниваем! Всё ништяк!\"}]");
    // 
    //     assert_eq!(result, expected);
    // }

    #[test]
    fn test_send_message() {
        let chat_id = "example_chat_id".to_string();
        let user_id = "123456789".to_string();
        let role = "User".to_string();
        let content = "This is a sample message content.".to_string();

        let expected_json = r#"{"chat_id": "example_chat_id", "llm_message": {"sender_id": "123456789", "role": "User", "content": "This is a sample message content."}}"#;
        let result = send_message(chat_id, user_id, role, content);

        assert_eq!(result, expected_json);
    }
}