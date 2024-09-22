use std::sync::Arc;

use crate::config::jarvis::JarvisAppState;
use crate::models::qdrant_search_layers::{
    QdrantSearchInfo, QdrantSearchLayer, QdrantUserRoleTextType,
};
use anyhow::bail;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use nervo_sdk::api::spec::{
    LlmChat, LlmMessage, LlmMessageContent, LlmMessageMetaInfo, LlmMessagePersistence,
    LlmMessageRole, SendMessageRequest,
};
use qdrant_client::qdrant::value::Kind;
use qdrant_client::qdrant::ScoredPoint;
use tiktoken_rs::cl100k_base;
use tokio::fs;
use tracing::info;

pub const RESOURCES_DIR: &str = "../resources";

//Common entry point for WEB and TG
pub async fn llm_conversation(
    app_state: Arc<JarvisAppState>,
    msg_request: SendMessageRequest,
    agent_type: AgentType,
) -> anyhow::Result<LlmMessage> {
    info!("start LLM layers handling");
    let message_sender_id_string = msg_request.llm_message.sender_id.to_string();
    let table_name = message_sender_id_string.as_str();
    let msg = msg_request.llm_message;
    let initial_user_content = msg.content.0.as_str();
    let user_id = msg.sender_id;
    let chat_id = msg_request.chat_id;
    let layers_info = get_all_search_layers(agent_type).await?;
    let all_messages: Vec<LlmMessage> = app_state.local_db.read_from_local_db(&table_name).await?;

    let initial_user_request = detecting_crap_request(
        app_state.clone(),
        &table_name,
        &initial_user_content,
        chat_id,
        layers_info.clone().crap_detecting_layer,
        all_messages.clone(),
    )
    .await?;

    if initial_user_request == "SKIP" {
        save_chat_history(
            app_state.clone(),
            user_id,
            &initial_user_content,
            &table_name,
            LlmMessagePersistence::Temporal,
            LlmMessageRole::User,
        )
        .await?;
        let llm_request_content =
            build_crap_layer_llm_request(agent_type, &initial_user_content).await?;
        let llm_message_text = get_string_from_llm_response(
            app_state.clone(),
            llm_request_content,
            user_id,
            chat_id,
            LlmMessagePersistence::Temporal,
            LlmMessageRole::User,
        )
        .await?;
        let llm_response = save_chat_history(
            app_state,
            user_id,
            llm_message_text.as_str(),
            &table_name,
            LlmMessagePersistence::Temporal,
            LlmMessageRole::Assistant,
        )
        .await?;

        info!(
            "Final response from LLM w/o RAG: {}",
            llm_response.content.text()
        );
        Ok(llm_response)
    } else {
        save_chat_history(
            app_state.clone(),
            user_id,
            &initial_user_content,
            &table_name,
            LlmMessagePersistence::Persistent,
            LlmMessageRole::User,
        )
        .await?;

        let messages_count = all_messages.len();
        let table_name_start_index = format!("{}_start_index", &table_name);
        app_state
            .local_db
            .save_to_local_db(messages_count, &table_name_start_index, Some(10_i64))
            .await?;

        let llm_response_text = rag_system_processing(
            app_state.clone(),
            layers_info,
            &initial_user_content,
            &table_name,
            chat_id,
            agent_type,
            all_messages,
        )
        .await?;

        let llm_response = save_chat_history(
            app_state.clone(),
            user_id,
            &llm_response_text,
            &table_name,
            LlmMessagePersistence::Persistent,
            LlmMessageRole::Assistant,
        )
        .await?;

        app_state
            .local_db
            .save_to_local_db(messages_count + 1, &table_name_start_index, Some(10_i64))
            .await?;

        info!(
            "Final response from LLM with RAG: {}",
            llm_response.content.text()
        );
        Ok(llm_response)
    }
}

async fn detecting_crap_request(
    app_state: Arc<JarvisAppState>,
    table_name: &str,
    initial_user_prompt: &str,
    chat_id: u64,
    layer: QdrantSearchLayer,
    all_saved_messages: Vec<LlmMessage>,
) -> anyhow::Result<String> {
    info!("CRAP DETECTION Started");

    let layer_content = create_layer_content(
        app_state,
        &table_name,
        layer,
        initial_user_prompt,
        "",
        "",
        chat_id,
        all_saved_messages,
    )
    .await?;
    info!("Is crap detected: {}", layer_content == "SKIP");
    Ok(layer_content)
}

async fn create_user_message_of(
    persistence_type: LlmMessagePersistence,
    sender_id: u64,
    content: &str,
    role: LlmMessageRole,
) -> anyhow::Result<LlmMessage> {
    let llm_user_message = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: Some(sender_id),
            role,
            persistence: persistence_type,
        },
        content: LlmMessageContent(String::from(content)),
    };
    Ok(llm_user_message)
}

async fn build_crap_layer_llm_request(
    agent_type: AgentType,
    content: &str,
) -> anyhow::Result<LlmMessageContent> {
    info!("Start Preparing handled crap Answer");
    // Prepare System Role and User question to ask a regular LLM
    let agent_type_name = NervoAgentType::get_name(agent_type);
    let resource_path = format!(
        "{}/agent/{}/crap_request_system_role.txt",
        RESOURCES_DIR, agent_type_name
    );
    let crap_system_role = std::fs::read_to_string(resource_path)?;

    let user_request_text = format!(
        "{:?}\nТекущий запрос пользователя: {:?}",
        crap_system_role, &content
    );
    let content = LlmMessageContent::from(user_request_text.as_str());
    info!("LLMContent of  crap Answer is ready");
    Ok(content)
}

async fn get_string_from_llm_response(
    app_state: Arc<JarvisAppState>,
    llm_content: LlmMessageContent,
    sender_id: u64,
    chat_id: u64,
    persistence_type: LlmMessagePersistence,
    role: LlmMessageRole,
) -> anyhow::Result<String> {
    info!("Need to get LLM Message from LLM Content");
    let request_to_llm = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: Some(sender_id),
            role,
            persistence: persistence_type,
        },
        content: llm_content,
    };

    // Asking LLM
    let llm_response_text = app_state
        .nervo_llm
        .send_msg(request_to_llm, chat_id)
        .await?;
    info!(
        "LLM Response WITHOUT RAG handling {:?}",
        llm_response_text.clone()
    );
    Ok(llm_response_text)
}

async fn save_chat_history(
    app_state: Arc<JarvisAppState>,
    user_id: u64,
    content: &str,
    table_name: &str,
    persistence_type: LlmMessagePersistence,
    role: LlmMessageRole,
) -> anyhow::Result<LlmMessage> {
    info!(" Save to DB to restore chat history, not for history context");
    let llm_message = create_user_message_of(persistence_type, user_id, content, role).await?;

    app_state
        .local_db
        .save_to_local_db(llm_message.clone(), &table_name, None)
        .await?;
    Ok(llm_message)
}

pub async fn get_all_search_layers(agent_type: AgentType) -> anyhow::Result<QdrantSearchInfo> {
    let agent_type_name = NervoAgentType::get_name(agent_type);
    info!("Getting all layers info for {} bot", agent_type_name);
    let resource_path = format!(
        "{}/agent/{}/vectorisation_roles.json",
        RESOURCES_DIR, agent_type_name
    );
    let json_string = fs::read_to_string(resource_path).await?;
    let all_layers_data: QdrantSearchInfo = serde_json::from_str(&json_string)?;
    info!("There are {} layers", all_layers_data.layers.len() + 1);
    Ok(all_layers_data)
}

async fn create_layer_content(
    app_state: Arc<JarvisAppState>,
    db_table_name: &str,
    layer: QdrantSearchLayer,
    initial_user_prompt: &str,
    llm_rephrased_prompt: &str,
    search_result_content: &str,
    chat_id: u64,
    all_saved_messages: Vec<LlmMessage>,
) -> anyhow::Result<String> {
    info!(
        "Create layer request content accor ding to user initial question {}",
        &initial_user_prompt
    );
    let mut messages: Vec<LlmMessage> = Vec::new();
    let user_role_msg = formation_user_role_llm_message(
        app_state.clone(),
        &db_table_name,
        all_saved_messages,
        layer.clone(),
        initial_user_prompt,
        llm_rephrased_prompt,
        search_result_content.to_string(),
    )
    .await?;

    let system_role_msg = formation_system_role_llm_message(layer).await?;

    messages.push(system_role_msg);
    messages.push(user_role_msg);

    let chat: LlmChat = LlmChat { chat_id: Some(chat_id), messages };
    info!("Making chat with llm and prepared user and system roles");
    app_state.nervo_llm.send_msg_batch(chat).await
}

async fn formation_user_role_llm_message(
    app_state: Arc<JarvisAppState>,
    table_name: &str,
    all_saved_messages: Vec<LlmMessage>,
    layer: QdrantSearchLayer,
    prompt: &str,
    llm_rephrased_prompt: &str,
    search_result_content: String,
) -> anyhow::Result<LlmMessage> {
    info!("Let's start User Role Creation formation");

    let mut user_role_full_text = String::new();
    for parameter in layer.user_role_params {
        let value = match parameter.param_type {
            QdrantUserRoleTextType::History => {
                let cached_messages = get_last_messages_from_cache(
                    app_state.clone(),
                    &table_name,
                    all_saved_messages.clone(),
                )
                .await?;
                cached_messages
                    .clone()
                    .iter()
                    .map(|msg| msg.content.text())
                    .collect::<Vec<String>>()
                    .join("\n")
            }
            QdrantUserRoleTextType::UserPrompt => prompt.to_string().clone(),
            QdrantUserRoleTextType::RephrasedPrompt => llm_rephrased_prompt.to_string(),
            QdrantUserRoleTextType::DbSearch => search_result_content.clone(),
        };
        let part = format!("{:?}{:?}\n", parameter.param_value, value);
        user_role_full_text.push_str(&part)
    }

    let user_role_msg = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: None,
            role: LlmMessageRole::User,
            persistence: LlmMessagePersistence::Persistent,
        },
        content: LlmMessageContent::from(user_role_full_text.as_str()),
    };
    info!("User Role full text: {}", user_role_full_text);
    Ok(user_role_msg)
}

pub async fn formation_system_role_llm_message(
    layer: QdrantSearchLayer,
) -> anyhow::Result<LlmMessage> {
    info!("Let's prepare system role!");
    let system_role_full_text = layer.system_role_text;
    let system_role_msg = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: None,
            role: LlmMessageRole::System,
            persistence: LlmMessagePersistence::Persistent,
        },
        content: LlmMessageContent::from(system_role_full_text.as_str()),
    };
    info!("System Role full text: {}", system_role_full_text);
    Ok(system_role_msg)
}

async fn get_last_messages_from_cache(
    app_state: Arc<JarvisAppState>,
    table_name: &str,
    all_saved_messages: Vec<LlmMessage>,
) -> anyhow::Result<Vec<LlmMessage>> {
    info!("Looking for last N messages");
    let start_index_table_name = format!("{}_start_index", table_name);
    let context_messages_indexes: Vec<i64> = app_state
        .local_db
        .read_from_local_db(&start_index_table_name)
        .await?;

    let mut cached_messages: Vec<LlmMessage> = vec![];
    for index in context_messages_indexes {
        if let Some(content_message) = all_saved_messages.get(index as usize) {
            cached_messages.push(content_message.clone());
        }
    }
    info!(
        "We got {} cached messages to build history context",
        cached_messages.len()
    );
    Ok(cached_messages)
}

async fn rag_system_processing(
    app_state: Arc<JarvisAppState>,
    all_layers_info: QdrantSearchInfo,
    initial_user_prompt: &str,
    table_name: &str,
    chat_id: u64,
    agent_type: AgentType,
    all_saved_messages: Vec<LlmMessage>,
) -> anyhow::Result<String> {
    info!("Initial INPUT prompt for LLM: {}", &initial_user_prompt);
    let processing_layers = all_layers_info.layers;
    let mut llm_rephrased_prompt = String::from(initial_user_prompt);
    info!("We need to process thru {} layers", processing_layers.len());
    for processing_layer in processing_layers {
        let mut search_content = String::new();
        if processing_layer.layer_for_search {
            search_content = searching_in_qdrant(
                app_state.clone(),
                agent_type,
                &llm_rephrased_prompt,
                processing_layer.clone(),
            )
            .await?;
        }

        llm_rephrased_prompt = create_layer_content(
            app_state.clone(),
            &table_name,
            processing_layer,
            initial_user_prompt,
            llm_rephrased_prompt.as_str(),
            search_content.as_str(),
            chat_id,
            all_saved_messages.clone(),
        )
        .await?;
    }

    let start_index_table_name = format!("{}_start_index", &table_name);
    let context_messages_indexes: Vec<i64> = app_state
        .local_db
        .read_from_local_db(&start_index_table_name)
        .await?;

    let mut cached_messages: Vec<LlmMessage> = vec![];
    for index in context_messages_indexes {
        if let Some(content_message) = all_saved_messages.get(index as usize) {
            cached_messages.push(content_message.clone());
        }
    }

    info!("cached_messages {:?}", cached_messages.len());
    if cached_messages.len() % 4 == 0 {
        llm_rephrased_prompt.push_str(&all_layers_info.info_message_1);
    };

    if cached_messages.len() % 6 == 0 {
        llm_rephrased_prompt.push_str(&all_layers_info.info_message_2);
    };

    Ok(llm_rephrased_prompt)
}

async fn searching_in_qdrant(
    app_state: Arc<JarvisAppState>,
    agent_type: AgentType,
    llm_rephrased_prompt: &str,
    processing_layer: QdrantSearchLayer,
) -> anyhow::Result<String> {
    info!("Need to ask QDrant DB to get some info");
    let mut search_content = String::new();
    let mut all_search_results = vec![];

    let db_search_response = &app_state
        .nervo_ai_db
        .text_search(
            agent_type,
            llm_rephrased_prompt.to_string(),
            processing_layer.vectors_limit,
        )
        .await?;

    for search_result in &db_search_response.result {
        if search_result.score >= 0.3 {
            info!(
                "ADD to SEARCH_Result: {:?} with \n SCORE {}",
                &search_result.payload["text"].kind, search_result.score
            );
            all_search_results.push(search_result.clone());
        }
    }

    all_search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    all_search_results.truncate(10);

    let scores_string = all_search_results
        .iter()
        .map(|element| element.score.to_string())
        .collect::<Vec<String>>()
        .join(", ");

    info!("All_search_result scores {}", scores_string);

    let concatenated_texts = concatenate_results(all_search_results)?;
    let token_limit = processing_layer.common_token_limit as usize;
    let updated_content = update_search_content(token_limit, concatenated_texts)?;
    search_content.push_str(updated_content.as_str());
    Ok(search_content)
}

fn update_search_content(token_limit: usize, concatenated_texts: String) -> anyhow::Result<String> {
    info!("Update search content");

    let bpe = cl100k_base()?;
    let mut tokens = bpe.split_by_token(&concatenated_texts, true)?;

    info!(
        "tokens len {:?} and token limit {:?}",
        tokens.len(),
        token_limit
    );
    if tokens.len() > token_limit {
        tokens.truncate(token_limit);
        let truncated = tokens.join("");
        info!("Truncated search_result: {}", truncated);
        Ok(truncated)
    } else {
        info!("Concatenated_texts (non-truncated): {}", concatenated_texts);
        Ok(concatenated_texts)
    }
}

fn concatenate_results(all_search_results: Vec<ScoredPoint>) -> anyhow::Result<String> {
    info!("Need to concatenate vector");
    let mut concatenated_texts: String = String::new();

    for search_result in all_search_results {
        let Some(Kind::StringValue(text)) = &search_result.payload["text"].kind else {
            bail!("Oooops! Error")
        };
        concatenated_texts.push_str(text.as_str());
    }
    info!("Concatenating are done");
    Ok(concatenated_texts)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::utils::ai_utils::{concatenate_results, update_search_content};
    use qdrant_client::qdrant::ScoredPoint;

    #[test]
    fn test_update_search_content() -> anyhow::Result<()> {
        let content = update_search_content(1, String::from("hi hi"))?;
        assert_eq!(content, String::from("hi"));
        Ok(())
    }

    #[test]
    fn test_concatenate_results() -> anyhow::Result<()> {
        let mut data = HashMap::new();
        data.insert(
            "text".to_string(),
            qdrant_client::qdrant::Value::from("lala-ley".to_string()),
        );
        let vector = vec![ScoredPoint {
            id: None,
            payload: data,
            score: 0.5,
            version: 0,
            vectors: None,
            shard_key: None,
            order_value: None,
        }];
        let test_result = concatenate_results(vector)?;
        assert_eq!(test_result, String::from("lala-ley"));
        Ok(())
    }
}
