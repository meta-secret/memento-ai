use pulldown_cmark::{html, Parser};
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use nervo_sdk::api::spec::{LlmChat, LlmMessage, LlmMessageContent, LlmMessageMetaInfo, LlmMessageRole, SendMessageRequest, UserLlmMessage};
use reqwest::Client;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsError, JsValue};

use tracing_web::{performance_layer, MakeWebConsoleWriter};
use tracing_subscriber::prelude::*;
use tracing::{info};
use tracing_subscriber::fmt::format::Pretty;
use nervo_sdk::errors::NervoWebResult;
use crate::browser::nervo_wasm_store::NervoWasmStore;
use crate::run_mode::{ClientRunMode, ClientRunModeUtil};

mod utils;
pub mod browser;
mod db;

pub mod run_mode {
    use wasm_bindgen::prelude::wasm_bindgen;
    use nervo_sdk::errors::{NervoWebError, NervoWebResult};

    pub const LOCAL: &str = "localDev";
    pub const DEV: &str = "dev";
    pub const PROD: &str = "prod";

    #[wasm_bindgen]
    #[derive(Copy, Clone, Debug)]
    pub enum ClientRunMode {
        Local,
        Dev,
        Prod,
    }

    #[wasm_bindgen]
    pub struct ClientRunModeUtil {}

    #[wasm_bindgen]
    impl ClientRunModeUtil {
        pub fn parse(mode: &str) -> NervoWebResult<ClientRunMode> {
            match mode {
                LOCAL => Ok(ClientRunMode::Local),
                DEV => Ok(ClientRunMode::Dev),
                PROD => Ok(ClientRunMode::Prod),
                _ => {
                    let mode_str = String::from(mode);
                    let error = NervoWebError::UnknownRunModeError(mode_str);
                    Err(error)
                }
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct ApiUrl {
    url: &'static str,
    port: u32,
    run_mode: ClientRunMode,
}

#[wasm_bindgen]
impl ApiUrl {
    pub fn get(port: u32, run_mode: ClientRunMode) -> Self {
        match run_mode {
            ClientRunMode::Local => ApiUrl::local(port),
            ClientRunMode::Dev => ApiUrl::dev(port),
            ClientRunMode::Prod => ApiUrl::prod(),
        }
    }

    pub fn local(port: u32) -> Self {
        ApiUrl {
            url: "http://localhost",
            port,
            run_mode: ClientRunMode::Local,
        }
    }
    pub fn dev(port: u32) -> Self {
        ApiUrl {
            url: "http://nervoset.metaelon.space",
            port,
            run_mode: ClientRunMode::Dev,
        }
    }

    pub fn prod() -> Self {
        ApiUrl {
            url: "https://prod.metaelon.space",
            port: 443,
            run_mode: ClientRunMode::Prod,
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
    pub agent_type: AgentType,
    client: Client,
    nervo_store: NervoWasmStore
}

#[wasm_bindgen]
impl NervoClient {
    
    pub async fn init(server_port: u32, run_mode: &str, agent_type: &str) -> NervoWebResult<NervoClient> {
        let run_mode = ClientRunModeUtil::parse(run_mode)?;
        let agent_type = NervoAgentType::try_from(agent_type);
        let api_url = ApiUrl::get(server_port, run_mode);

        info!("Agent type: {:?}, port: {:?}, run mode: {:?}", agent_type, server_port, run_mode);

        Ok(NervoClient {
            api_url,
            agent_type,
            client: Client::new(),
            nervo_store: NervoWasmStore::init().await
        })
    }

    pub fn configure_tracing() {
        utils::set_panic_hook();
        
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false) // Only partially supported across browsers
            .without_time()   // std::time is not available in browsers, see note below
            .with_writer(MakeWebConsoleWriter::new()); // write events to the console
        let perf_layer = performance_layer()
            .with_details_from_fields(Pretty::default());

        let _ = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(perf_layer)
            .try_init(); // Install these as subscribers to tracing events
    }

    #[wasm_bindgen]
    pub async fn get_chat(&self) -> Result<LlmChat, JsValue> {
        info!("get_chat");

        let chat_id = self.nervo_store.get_or_generate_chat_id().await;

        let url = format!("{}/chat/{}", self.api_url.get_url(), chat_id);
        info!("LIB: url {:?}", url);

        let response = match self.client.get(url).send().await {
            Ok(response) => {
                info!("LIB: FETCH GET response {:?}", response);
                response
            }
            Err(error) => return Err(JsValue::from_str(&format!("Request failed: {}", error))),
        };

        let chat: LlmChat = response.json().await.map_err(JsError::from)?;

        let transformed_messages: Vec<LlmMessage> = chat.messages.iter().map(|x| {
            match x.meta_info.role {
                LlmMessageRole::User => x.clone(),
                _ => {
                    let markdown_text = x.content.text();
                    let html_text = markdown_to_html(&markdown_text);
                    let content = LlmMessageContent::from(html_text.as_ref());

                    LlmMessage {
                        meta_info: LlmMessageMetaInfo {
                            sender_id: x.meta_info.sender_id,
                            role: x.meta_info.role,
                            persistence: x.meta_info.persistence,
                        },
                        content,
                    }
                }
            }
        }).collect();

        Ok(LlmChat {
            chat_id: chat.chat_id,
            messages: transformed_messages,
        })
    }

    pub async fn send_message(&self, content: String) -> Result<LlmMessage, JsValue> {
        let chat_id = self.nervo_store.get_or_generate_chat_id().await;
        let user_id = self.nervo_store.get_or_generate_user_id().await;

        let json = SendMessageRequest {
            chat_id,
            agent_type: self.agent_type,
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

        let llm_message_response: LlmMessage = response.json().await.map_err(JsError::from)?;
        info!("LIB: Response LlmMessage: {:?}", llm_message_response);
        let markdown_text = llm_message_response.content.text();
        let html_text = markdown_to_html(&markdown_text);
        info!("LIB: html_text: {:?}", html_text);
        let content = LlmMessageContent::from(html_text.as_ref());

        Ok(LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: llm_message_response.meta_info.sender_id,
                role: llm_message_response.meta_info.role,
                persistence: llm_message_response.meta_info.persistence,
            },
            content,
        })
    }
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}