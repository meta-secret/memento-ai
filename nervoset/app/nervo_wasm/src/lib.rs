use error_stack::ResultExt;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use nervo_sdk::api::spec::{
    LlmChat, LlmMessage, LlmMessageContent, LlmMessageMetaInfo, LlmMessageRole, SendMessageRequest,
    UserLlmMessage,
};
use pulldown_cmark::{html, Parser};
use reqwest::Client;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::browser::nervo_wasm_store::NervoWasmStore;
use crate::common::api_url::ApiUrl;
use crate::common::nweb_spans;
use crate::common::nweb_spans::nweb_send_msg_span;
use crate::run_mode::ClientRunModeUtil;
use nervo_sdk::errors::NervoWebResult;
use tracing::{info, Instrument};
use tracing_subscriber::fmt::format::Pretty;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeConsoleWriter};

pub mod browser;
mod common;
mod db;
mod utils;

pub mod run_mode {
    use nervo_sdk::errors::{NervoSdkError, NervoWebResult};
    use wasm_bindgen::prelude::wasm_bindgen;

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
                    let error = NervoSdkError::UnknownRunModeError(mode_str);
                    Err(error)
                }
            }
        }
    }
}

#[wasm_bindgen]
pub struct NervoClient {
    pub api_url: ApiUrl,
    pub agent_type: AgentType,
    client: Client,
    nervo_store: NervoWasmStore,
}

#[wasm_bindgen]
impl NervoClient {
    pub async fn init(
        server_port: u32,
        run_mode: &str,
        agent_type: &str,
    ) -> NervoWebResult<NervoClient> {
        let run_mode = ClientRunModeUtil::parse(run_mode)?;
        let agent_type = NervoAgentType::try_from(agent_type);
        let api_url = ApiUrl::get(server_port, run_mode);

        info!(
            "Agent type: {:?}, port: {:?}, run mode: {:?}",
            agent_type, server_port, run_mode
        );

        Ok(NervoClient {
            api_url,
            agent_type,
            client: Client::new(),
            nervo_store: NervoWasmStore::init().await,
        })
    }

    pub fn configure_tracing() {
        utils::set_panic_hook();

        let fmt_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_ansi(false)
            //.with_max_level(Level::INFO)
            .pretty() // Only partially supported across browsers
            .with_timer(UtcTime::rfc_3339()) // std::time is not available in browsers
            .with_writer(MakeConsoleWriter); // write events to the console

        let perf_layer = performance_layer().with_details_from_fields(Pretty::default());

        let _ = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(perf_layer)
            .try_init(); // Install these as subscribers to tracing events
    }

    #[wasm_bindgen]
    pub async fn get_chat(&self) -> LlmChat {
        info!("get_chat");

        let chat_id = self
            .nervo_store
            .get_or_generate_chat_id()
            .instrument(nweb_spans::nweb_chat_span())
            .await;

        let url = format!("{}/chat/{}", self.api_url.get_url(), chat_id);
        info!("url {:?}", url);

        let response = self
            .client
            .get(url)
            .send()
            .instrument(nweb_spans::nweb_chat_span())
            .await
            .attach_printable_lazy(|| "Failed fetching chat")
            .unwrap();

        let chat: LlmChat = response
            .json()
            .instrument(nweb_spans::nweb_chat_span())
            .await
            .attach_printable_lazy(|| "Failed parsing chat response as json")
            .unwrap();

        let transformed_messages: Vec<LlmMessage> = chat
            .messages
            .iter()
            .map(|msg| match msg.meta_info.role {
                LlmMessageRole::User => msg.clone(),
                _ => {
                    let markdown_text = msg.content.text();
                    let html_text = markdown_to_html(&markdown_text);
                    let content = LlmMessageContent::from(html_text.as_ref());

                    LlmMessage {
                        meta_info: LlmMessageMetaInfo {
                            sender_id: msg.meta_info.sender_id,
                            role: msg.meta_info.role,
                            persistence: msg.meta_info.persistence,
                        },
                        content,
                    }
                }
            })
            .collect();

        LlmChat {
            chat_id: chat.chat_id,
            messages: transformed_messages,
        }
    }

    pub async fn send_message(&self, content: String) -> LlmMessage {
        let chat_id = self
            .nervo_store
            .get_or_generate_chat_id()
            .instrument(nweb_send_msg_span())
            .await;
        let user_id = self
            .nervo_store
            .get_or_generate_user_id()
            .instrument(nweb_send_msg_span())
            .await;

        let json = SendMessageRequest {
            chat_id,
            agent_type: self.agent_type,
            llm_message: UserLlmMessage {
                sender_id: user_id,
                content: LlmMessageContent(content),
            },
        };

        let url = format!("{}/send_message", self.api_url.get_url());
        info!("Send msg url {:?} with json: {:?}", url, json);

        let response = self
            .client
            .post(url.clone())
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", url)
            .json(&json)
            .send()
            .instrument(nweb_send_msg_span())
            .await
            .attach_printable_lazy(|| "Failed sending message")
            .unwrap();

        let llm_message_response: LlmMessage = response
            .json()
            .instrument(nweb_send_msg_span())
            .await
            .attach_printable_lazy(|| "Json parsing error")
            .unwrap();

        info!("Response LlmMessage: {:?}", llm_message_response);
        let markdown_text = llm_message_response.content.text();
        let html_text = markdown_to_html(&markdown_text);
        info!("html_text: {:?}", html_text);
        let content = LlmMessageContent::from(html_text.as_ref());

        LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: llm_message_response.meta_info.sender_id,
                role: llm_message_response.meta_info.role,
                persistence: llm_message_response.meta_info.persistence,
            },
            content,
        }
    }
}

fn markdown_to_html(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
