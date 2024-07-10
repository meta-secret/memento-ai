use std::sync::Arc;
use anyhow::bail;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent};
use qdrant_client::qdrant::{ScoredPoint, SearchResponse};
use qdrant_client::qdrant::value::Kind;
use serde_derive::{Deserialize, Serialize};
use teloxide::types::MessageKind::Common;
use tiktoken_rs::p50k_base;
use tokio::fs;
use tracing::{error, info};
use crate::ai::nervo_llm::{LlmMessage, LlmMessageContent, UserLlmMessage};
use crate::ai::nervo_llm::LlmOwnerType::{Assistant, User};
use crate::ai::nervo_llm::LlmSaveContext::{False, True};
use crate::common::AppState;
use crate::models::qdrant_search_layers::{QDrantSearchInfo, QDrantSearchLayer, QDrantUserRoleTextType};

#[derive(Serialize, Deserialize, Clone)]
pub struct SendMessageRequest {
    pub chat_id: u64,
    pub llm_message: UserLlmMessage,
}

//Common entry point for WEB and TG
pub async fn llm_conversation(
    app_state: &Arc<AppState>,
    msg_request: SendMessageRequest,
    table_name: String
) -> anyhow::Result<LlmMessage> {
    let content = msg_request.llm_message.content.0.as_str();
    let user_id = msg_request.llm_message.sender_id.to_string();
    info!("COMMON: content: {:?} and user_id: {:?}", &content, user_id);

    // First, detect the request type. Will we use Qdrant or a simple LLM going forward?
    let initial_user_request = detecting_crap_request(&app_state, &content, user_id.as_str()).await?;

    // Prepare user message to save to DB
    let mut llm_user_message = LlmMessage {
        save_to_context: False,
        message_owner: User(UserLlmMessage {
            sender_id: user_id.parse().expect("Can't parse user id"),
            content: LlmMessageContent(String::from(content)),
        }),
    };

    // Prepare llm response to save to DB
    let mut llm_response = LlmMessage {
        save_to_context: False,
        message_owner: Assistant(LlmMessageContent(String::new())),
    };

    if initial_user_request == "SKIP" {
        // Save to DB to restore chat history, not for history context
        app_state.local_db.save_to_local_db(llm_user_message, &table_name, None).await?;

        // Prepare System Role anf User question to ask a simple LLM
        let crap_system_role =
            std::fs::read_to_string("resources/crap_request_system_role.txt")
                .expect("Failed to read system message from file");

        let user_request_text = format!(
            "{:?}\nТекущий запрос пользователя: {:?}",
            crap_system_role,
            &content
        );
        let content = LlmMessageContent::from(user_request_text.as_str());
        let request_to_llm = LlmMessage {
            save_to_context: False,
            message_owner: User(UserLlmMessage {
                sender_id: user_id.parse().expect("Can't parse user id"),
                content,
            }),
        };

        // Asking LLM
        let llm_response_text = app_state
            .nervo_llm
            .send_msg(request_to_llm, msg_request.chat_id)
            .await?;
        info!("SKIPPED LLM_RESPONSE {:?}", llm_response_text.clone());

        // Saving answer from LLM but not for History Context
        llm_response.message_owner = Assistant(LlmMessageContent(llm_response_text));
        app_state.local_db.save_to_local_db(llm_response.clone(), &table_name, None).await?;
        Ok(llm_response)
    } else {
        llm_user_message.save_to_context = True;
        
        // Save to DB to restore chat history, and for history context
        let all_messages: Vec<LlmMessage> = app_state.local_db.read_from_local_db(&table_name).await?;
        let messages_count = all_messages.len();
        let table_name_start_index = format!("{}_start_index", table_name.clone());
        app_state.local_db.save_to_local_db(messages_count, &table_name_start_index, Some(10_i64)).await?;
        app_state.local_db.save_to_local_db(llm_user_message, &table_name, None).await?;

        // Need to ask Qdrant
        let llm_response_text = openai_chat_preparations(&app_state, &initial_user_request, table_name.clone())
            .await?;

        llm_response.save_to_context = True;
        llm_response.message_owner = Assistant(LlmMessageContent(llm_response_text.clone()));
        // Saving answer from LLM but not for History Context
        app_state.local_db.save_to_local_db(messages_count+1, &table_name_start_index, Some(10_i64)).await?;
        app_state.local_db.save_to_local_db(llm_response.clone(), &table_name, None).await?;
        info!("LLM_RESPONSE {:?}", llm_response_text);
        Ok(llm_response)
    }
}

async fn detecting_crap_request(
    app_state: &Arc<AppState>,
    prompt: &str,
    user_id: &str,
) -> anyhow::Result<String> {
    info!("COMMON: CRAP DETECTION");
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
    info!("COMMON: get_all_search_layers");
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
    info!("COMMON: create_layer_content");
    let client = Client::new(app_state.nervo_llm.api_key().to_string());
    let all_saved_messages: Vec<LlmMessage> =
        app_state.local_db.read_from_local_db(db_table_name).await?;
    let start_index_table_name = format!("{}_start_index", db_table_name);
    info!("COMMON: start_index_table_name {}", start_index_table_name);
    let context_messages_indexes: Vec<i64> = app_state.local_db.read_from_local_db(&start_index_table_name).await?;
    info!("COMMON: context_messages_indexes len {:?}", context_messages_indexes.len());
    let mut cached_messages: Vec<LlmMessage> = vec![];
    for index in context_messages_indexes {
        if let Some(content_message) = all_saved_messages.get(index as usize) {
            info!("COMMON: add message");
            cached_messages.push(content_message.clone());
        }
    }

    let model_name = app_state.nervo_config.llm.model_name.clone();
    let system_role_content = layer.system_role_text;

    //Create user request role text
    let mut user_role_full_text = String::new();
    for parameter in layer.user_role_params {
        let value = match parameter.param_type {
            QDrantUserRoleTextType::History => cached_messages
                .clone()
                .iter()
                .map(|msg| msg.content().0)
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
    table_name: String,
) -> anyhow::Result<String> {
    let mut rephrased_prompt = String::from(prompt);
    let bpe = p50k_base().unwrap();
    let layers_info = get_all_search_layers().await?;
    let processing_layers = layers_info.layers;
    let mut search_content = String::new();

    for processing_layer in processing_layers {
        if !processing_layer.collection_params.is_empty() {
            info!("COMMON: QDrant collections count is {:?}", &processing_layer.collection_params.len());
            
            let mut all_search_results = vec![];
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
                    if search_result.score.clone() >= 0.3 {
                        all_search_results.push(search_result.clone());
                    }
                }
            }

            all_search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            all_search_results.truncate(10);
            let mut concatenated_texts: String = String::new();
            for search_result in all_search_results {
                let Some(Kind::StringValue(text)) =
                    &search_result.payload["text"].kind
                    else {
                        bail!("Oooops! Error")
                    };
                concatenated_texts.push_str(text);
            }
            
            info!("COMMON: concatenated text {}", concatenated_texts.clone());
            let token_limit = processing_layer.common_token_limit as usize;
            let mut tokens = bpe.split_by_token(&concatenated_texts, true)?;

            info!("COMMON: tokens len {:?} and token limit {:?}", tokens.len(), token_limit);
            if tokens.len() > token_limit {
                tokens.truncate(token_limit);
                let response = tokens.join("");
                search_content.push_str(&response);
            } else {
                search_content.push_str(&concatenated_texts);
            }
            info!("COMMON: search content result: {}", search_content);
        }

        rephrased_prompt = create_layer_content(
            &app_state,
            &prompt,
            &table_name,
            processing_layer.clone(),
            rephrased_prompt.clone(),
            search_content.clone()).await?;
    }

    let all_saved_messages: Vec<LlmMessage> =
        app_state.local_db.read_from_local_db(&table_name).await?;
    let start_index_table_name = format!("{}_start_index", table_name.clone());
    let context_messages_indexes: Vec<i64> = app_state.local_db.read_from_local_db(&start_index_table_name).await?;

    let mut cached_messages: Vec<LlmMessage> = vec![];
    for index in context_messages_indexes {
        if let Some(content_message) = all_saved_messages.get(index as usize) {
            cached_messages.push(content_message.clone());
        }
    }
    if cached_messages.len() % 5 == 0 {
        rephrased_prompt.push_str(&layers_info.info_message);
    };

    Ok(String::from(rephrased_prompt.clone()))
}