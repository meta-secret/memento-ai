use std::sync::Arc;
use anyhow::bail;
use chrono::Utc;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent};
use qdrant_client::qdrant::value::Kind;
use serde_derive::{Deserialize, Serialize};
use tiktoken_rs::p50k_base;
use tokio::fs;
use tracing::{error, info};
use crate::ai::nervo_llm::{LlmMessage, LlmMessageContent, UserLlmMessage};
use crate::common::AppState;
use crate::models::nervo_message_model::TelegramMessage;
use crate::models::qdrant_search_layers::{QDrantSearchInfo, QDrantSearchLayer, QDrantUserRoleTextType};

#[derive(Serialize, Deserialize, Clone)]
pub struct SendMessageRequest {
    pub chat_id: u64,
    pub llm_message: UserLlmMessage,
}

pub async fn  llm_conversation(app_state: &Arc<AppState>, msg_request: SendMessageRequest, table_name: String) -> anyhow::Result<LlmMessage> {
    let content = msg_request.llm_message.content.0.as_str();
    let user_id = msg_request.llm_message.sender_id.to_string();
    let initial_user_request = detecting_crap_request(&app_state, content, user_id.as_str()).await?;

    if initial_user_request == "SKIP" {
        let crap_system_role =
            std::fs::read_to_string("resources/crap_request_system_role.txt")
                .expect("Failed to read system message from file");

        let user_request = format!(
            "{:?}\nТекущий запрос пользователя: {:?}",
            crap_system_role,
            &content
        );
        let content = LlmMessageContent::from(user_request.as_str()); 

        let request_to_llm = LlmMessage::User(UserLlmMessage { sender_id: msg_request.llm_message.sender_id, content });

        let llm_response = app_state
            .nervo_llm
            .send_msg(request_to_llm, msg_request.chat_id)
            .await?;
        info!("SKIPPED LLM_RESPONSE {:?}", llm_response);
        Ok(llm_response)
    } else {
        let result = openai_chat_preparations(&app_state, &initial_user_request, user_id.as_str(), table_name)
                .await?;
        
        let llm_response = LlmMessage::Assistant(LlmMessageContent::from(result.as_str()));
        info!("LLM_RESPONSE {:?}", llm_response);
        Ok(llm_response)
    }
}

async fn detecting_crap_request(
    app_state: &Arc<AppState>,
    prompt: &str,
    user_id: &str,
) -> anyhow::Result<String> {
    let layers_info = get_all_search_layers().await?;
    let layer_content = create_layer_content(
        &app_state,
        prompt,
        user_id,
        layers_info.crap_detecting_layer,
        String::new(),
        String::new(),
    )
        .await?;
    Ok(layer_content)
}

pub async fn get_all_search_layers() -> anyhow::Result<QDrantSearchInfo> {
    let json_string = fs::read_to_string("resources/vectorisation_roles.json").await?;
    let layers_info: QDrantSearchInfo =
        serde_json::from_str(&json_string).expect("Failed to parse JSON");
    Ok(layers_info)
}

async fn create_layer_content(
    app_state: &AppState,
    prompt: &str,
    db_table_name: &str,
    layer: QDrantSearchLayer,
    rephrased_prompt: String,
    search_result_content: String,
) -> anyhow::Result<String> {
    let client = Client::new(app_state.nervo_llm.api_key().to_string());
    let cached_messages: Vec<TelegramMessage> =
        app_state.local_db.read_from_local_db(db_table_name).await?;
    let model_name = app_state.nervo_config.llm.model_name.clone();
    let system_role_content = layer.system_role_text;

    //Create user request role text
    let mut user_role_full_text = String::new();
    for parameter in layer.user_role_params {
        let value = match parameter.param_type {
            QDrantUserRoleTextType::History => cached_messages
                .clone()
                .iter()
                .map(|msg| msg.message.clone())
                .collect::<Vec<String>>()
                .join("\n"),
            QDrantUserRoleTextType::UserPromt => prompt.to_string().clone(),
            QDrantUserRoleTextType::RephrasedPromt => rephrased_prompt.clone(),
            QDrantUserRoleTextType::DBSearch => search_result_content.clone(),
        };
        let part = format!("{:?}{:?}\n", parameter.param_value, value);
        user_role_full_text.push_str(&part)
    }

    let mut messages: Vec<ChatMessage> = Vec::new();
    let system_role_msg = ChatMessage {
        role: openai_dive::v1::resources::chat::Role::System,
        content: ChatMessageContent::Text(String::from(system_role_content)),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };
    let user_role_msg = ChatMessage {
        role: openai_dive::v1::resources::chat::Role::User,
        content: ChatMessageContent::Text(String::from(user_role_full_text)),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };

    messages.push(system_role_msg);
    messages.push(user_role_msg);

    let params = ChatCompletionParameters {
        messages,
        model: model_name,
        frequency_penalty: None,
        logit_bias: None,
        logprobs: None,
        top_logprobs: None,
        max_tokens: Some(layer.max_tokens),
        n: None,
        presence_penalty: None,
        response_format: None,
        seed: None,
        stop: None,
        stream: None,
        temperature: Some(layer.temperature),
        top_p: None,
        tools: None,
        tool_choice: None,
        user: None,
    };
    let layer_processing_content = match client.chat().create(params).await {
        Ok(value) => value.choices[0].message.content.to_owned(),
        Err(err) => {
            error!("Error {:?}", err);
            bail!("Error {:?}", err)
        }
    };
    let layer_content_text = match layer_processing_content {
        ChatMessageContent::Text(text) => text,
        _ => String::new(),
    };
    Ok(layer_content_text)
}

async fn openai_chat_preparations(
    app_state: &AppState,
    prompt: &str,
    user_id: &str,
    table_name: String
) -> anyhow::Result<String> {
    let mut rephrased_prompt = String::from(prompt);
    let bpe = p50k_base().unwrap();
    let layers_info = get_all_search_layers().await?;
    let processing_layers = layers_info.layers;
    let mut search_content = String::new();

    for processing_layer in processing_layers {
        if !processing_layer.collection_params.is_empty() {
            for collection_param in &processing_layer.collection_params {

                let db_search = &app_state
                    .nervo_ai_db
                    .search(
                        &app_state,
                        collection_param.name.clone(),
                        rephrased_prompt.clone(),
                        collection_param.vectors_limit.clone(),
                    )
                    .await?;

                for search_result in &db_search.result {
                    if search_result.score.clone() > 0.1 {
                        let Some(Kind::StringValue(result)) =
                            &search_result.payload["text"].kind
                            else {
                                bail!("Oooops! Error")
                            };

                        let token_limit = collection_param.tokens_limit.clone() as usize;
                        let vectors_limit = collection_param.vectors_limit.clone() as usize;
                        let mut tokens = bpe.split_by_token(&result, true)?;

                        if tokens.len() > (token_limit / vectors_limit) {
                            tokens.truncate(token_limit / vectors_limit);
                            let response = tokens.join("");
                            search_content.push_str(&response);
                        } else {
                            search_content.push_str(result);
                        }
                    }
                }
            }
        }

        rephrased_prompt = create_layer_content(
            &app_state,
            &prompt,
            &table_name,
            processing_layer.clone(),
            rephrased_prompt.clone(),
            search_content.clone()).await?;
    }

    let tg_message = TelegramMessage {
        id: user_id.parse().expect("Failed to parse string"),
        message: rephrased_prompt.clone(),
        timestamp: Utc::now().naive_utc(),
    };
    app_state
        .local_db
        .save_to_local_db(tg_message, &table_name, false)
        .await?;

    let cached_messages: Vec<TelegramMessage> =
        app_state.local_db.read_from_local_db(&table_name).await?;
    if cached_messages.len() % 5 == 0 {
        rephrased_prompt.push_str(&layers_info.info_message);
    };

    Ok(String::from(rephrased_prompt.clone()))
        
    
}