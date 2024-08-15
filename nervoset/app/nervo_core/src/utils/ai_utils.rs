use std::sync::Arc;

use anyhow::bail;
use qdrant_client::qdrant::value::Kind;
use qdrant_client::qdrant::ScoredPoint;
use tiktoken_rs::cl100k_base;
use tokio::fs;
use tracing::info;

use crate::ai::nervo_llm::NervoLlm;
use crate::config::jarvis::JarvisAppState;
use crate::db::local_db::LocalDb;
use crate::models::qdrant_search_layers::{
    QdrantSearchInfo, QdrantSearchLayer, QdrantUserRoleTextType,
};
use nervo_api::agent_type::{AgentType, NervoAgentType};
use nervo_api::{
    LlmChat, LlmMessage, LlmMessageContent, LlmMessageMetaInfo, LlmMessagePersistence,
    LlmMessageRole, SendMessageRequest,
};

//Common entry point for WEB and TG
pub async fn llm_conversation(
    app_state: Arc<JarvisAppState>,
    msg_request: SendMessageRequest,
    table_name: String,
    agent_type: AgentType,
) -> anyhow::Result<LlmMessage> {
    let content = msg_request.llm_message.content.0.as_str();
    let user_id = msg_request.llm_message.sender_id.to_string();
    info!("COMMON: content: {:?} and user_id: {:?}", &content, user_id);

    // First, detect the request type. Will we use Qdrant or a simple LLM going forward?
    let initial_user_request = detecting_crap_request(
        app_state.clone(),
        content,
        user_id.as_str(),
        msg_request.chat_id,
        agent_type,
    )
    .await?;

    if initial_user_request == "SKIP" {
        let llm_user_message = LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: Some(msg_request.llm_message.sender_id),
                role: LlmMessageRole::User,
                persistence: LlmMessagePersistence::Temporal,
            },
            content: LlmMessageContent(String::from(content)),
        };
        // Save to DB to restore chat history, not for history context
        app_state
            .local_db
            .save_to_local_db(llm_user_message, &table_name, None)
            .await?;

        // Prepare System Role anf User question to ask a simple LLM
        let agent_type_name = NervoAgentType::get_name(agent_type);
        let resource_path = format!(
            "resources/agent/{}/crap_request_system_role.txt",
            agent_type_name
        );
        let crap_system_role = std::fs::read_to_string(resource_path)?;

        let user_request_text = format!(
            "{:?}\nТекущий запрос пользователя: {:?}",
            crap_system_role, &content
        );
        let content = LlmMessageContent::from(user_request_text.as_str());

        let request_to_llm = LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: Some(msg_request.llm_message.sender_id),
                role: LlmMessageRole::User,
                persistence: LlmMessagePersistence::Temporal,
            },
            content,
        };

        // Asking LLM
        let llm_response_text = app_state
            .nervo_llm
            .send_msg(request_to_llm, msg_request.chat_id)
            .await?;
        info!("SKIPPED LLM_RESPONSE {:?}", llm_response_text.clone());

        // Saving answer from LLM but not for History Context
        let llm_response = LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: None,
                role: LlmMessageRole::Assistant,
                persistence: LlmMessagePersistence::Temporal,
            },
            content: LlmMessageContent(llm_response_text),
        };

        app_state
            .local_db
            .save_to_local_db(llm_response.clone(), &table_name, None)
            .await?;
        Ok(llm_response)
    } else {
        let llm_user_message = LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: Some(msg_request.llm_message.sender_id),
                role: LlmMessageRole::User,
                persistence: LlmMessagePersistence::Persistent,
            },
            content: LlmMessageContent(String::from(content)),
        };

        // Save to DB to restore chat history, and for history context
        let all_messages: Vec<LlmMessage> =
            app_state.local_db.read_from_local_db(&table_name).await?;

        let messages_count = all_messages.len();
        let table_name_start_index = format!("{}_start_index", table_name.clone());
        app_state
            .local_db
            .save_to_local_db(messages_count, &table_name_start_index, Some(10_i64))
            .await?;
        app_state
            .local_db
            .save_to_local_db(llm_user_message, &table_name, None)
            .await?;

        let llm_response_text = openai_chat_preparations(
            app_state.clone(),
            &initial_user_request,
            table_name.clone(),
            msg_request.chat_id,
            agent_type,
        )
        .await?;

        let llm_response = LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: None,
                role: LlmMessageRole::Assistant,
                persistence: LlmMessagePersistence::Persistent,
            },
            content: LlmMessageContent(llm_response_text.clone()),
        };

        // Saving answer from LLM but not for History Context
        app_state
            .local_db
            .save_to_local_db(messages_count + 1, &table_name_start_index, Some(10_i64))
            .await?;
        app_state
            .local_db
            .save_to_local_db(llm_response.clone(), &table_name, None)
            .await?;

        info!("LLM_RESPONSE {:?}", llm_response_text);
        Ok(llm_response)
    }
}

async fn detecting_crap_request(
    app_state: Arc<JarvisAppState>,
    prompt: &str,
    user_id: &str,
    chat_id: u64,
    agent_type: AgentType,
) -> anyhow::Result<String> {
    info!("COMMON: CRAP DETECTION");
    let layers_info = get_all_search_layers(agent_type).await?;
    let layer_content = create_layer_content(
        &app_state.nervo_llm,
        &app_state.local_db,
        prompt,
        user_id,
        layers_info.crap_detecting_layer,
        String::new(),
        String::new(),
        chat_id,
    )
    .await?;
    Ok(layer_content)
}

pub async fn get_all_search_layers(agent_type: AgentType) -> anyhow::Result<QdrantSearchInfo> {
    info!("COMMON: get_all_search_layers");
    let agent_type_name = NervoAgentType::get_name(agent_type);
    let resource_path = format!(
        "resources/agent/{}/vectorisation_roles.txt",
        agent_type_name
    );
    let json_string = fs::read_to_string(resource_path).await?;
    let all_layers_data: QdrantSearchInfo = serde_json::from_str(&json_string)?;
    Ok(all_layers_data)
}

async fn create_layer_content(
    nervo_llm: &NervoLlm,
    local_db: &LocalDb,
    prompt: &str,
    db_table_name: &str,
    layer: QdrantSearchLayer,
    rephrased_prompt: String,
    search_result_content: String,
    chat_id: u64,
) -> anyhow::Result<String> {
    info!("COMMON: create_layer_content");
    let all_saved_messages: Vec<LlmMessage> = local_db.read_from_local_db(db_table_name).await?;

    let start_index_table_name = format!("{}_start_index", db_table_name);
    info!("COMMON: start_index_table_name {}", start_index_table_name);
    let context_messages_indexes: Vec<i64> =
        local_db.read_from_local_db(&start_index_table_name).await?;

    info!(
        "COMMON: context_messages_indexes len {:?}",
        context_messages_indexes.len()
    );

    let mut cached_messages: Vec<LlmMessage> = vec![];
    for index in context_messages_indexes {
        if let Some(content_message) = all_saved_messages.get(index as usize) {
            info!("COMMON: add message");
            cached_messages.push(content_message.clone());
        }
    }

    let system_role_content = layer.system_role_text;

    //Create user request role text
    let mut user_role_full_text = String::new();
    for parameter in layer.user_role_params {
        let value = match parameter.param_type {
            QdrantUserRoleTextType::History => cached_messages
                .clone()
                .iter()
                .map(|msg| msg.content.text())
                .collect::<Vec<String>>()
                .join("\n"),
            QdrantUserRoleTextType::UserPrompt => prompt.to_string().clone(),
            QdrantUserRoleTextType::RephrasedPrompt => rephrased_prompt.clone(),
            QdrantUserRoleTextType::DbSearch => search_result_content.clone(),
        };
        let part = format!("{:?}{:?}\n", parameter.param_value, value);
        user_role_full_text.push_str(&part)
    }

    let mut messages: Vec<LlmMessage> = Vec::new();

    let system_role_msg = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: None,
            role: LlmMessageRole::System,
            persistence: LlmMessagePersistence::Persistent,
        },
        content: LlmMessageContent::from(system_role_content.as_str()),
    };

    let user_role_msg = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: None,
            role: LlmMessageRole::User,
            persistence: LlmMessagePersistence::Persistent,
        },
        content: LlmMessageContent::from(user_role_full_text.as_str()),
    };

    messages.push(system_role_msg);
    messages.push(user_role_msg);

    let chat: LlmChat = LlmChat { chat_id, messages };

    nervo_llm.send_msg_batch(chat).await
}

async fn openai_chat_preparations(
    app_state: Arc<JarvisAppState>,
    prompt: &str,
    table_name: String,
    chat_id: u64,
    agent_type: AgentType,
) -> anyhow::Result<String> {
    let mut rephrased_prompt = String::from(prompt);
    let all_layers_data = get_all_search_layers(agent_type).await?;
    let processing_layers = all_layers_data.layers;
    let mut search_content = String::new();

    for processing_layer in processing_layers {
        if processing_layer.layer_for_search {
            let mut all_search_results = vec![];
            let db_search_response = &app_state
                .nervo_ai_db
                .search(
                    agent_type,
                    rephrased_prompt.clone(),
                    processing_layer.vectors_limit,
                )
                .await?;

            for search_result in &db_search_response.result {
                if search_result.score >= 0.3 {
                    all_search_results.push(search_result.clone());
                }
            }

            all_search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            all_search_results.truncate(10);

            let concatenated_texts = concatenate_results(all_search_results)?;

            let token_limit = processing_layer.common_token_limit as usize;
            let updated_content = update_search_content(token_limit, concatenated_texts)?;
            search_content.push_str(updated_content.as_str());
        }

        rephrased_prompt = create_layer_content(
            &app_state.nervo_llm,
            &app_state.local_db,
            prompt,
            &table_name,
            processing_layer.clone(),
            rephrased_prompt.clone(),
            search_content.clone(),
            chat_id,
        )
        .await?;
    }

    let all_saved_messages: Vec<LlmMessage> =
        app_state.local_db.read_from_local_db(&table_name).await?;
    let start_index_table_name = format!("{}_start_index", table_name.clone());
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
    if cached_messages.len() % 5 == 0 {
        rephrased_prompt.push_str(&all_layers_data.info_message);
    };

    Ok(rephrased_prompt)
}

fn update_search_content(token_limit: usize, concatenated_texts: String) -> anyhow::Result<String> {
    info!("COMMON: concatenated text {}", concatenated_texts.clone());

    let bpe = cl100k_base()?;
    let mut tokens = bpe.split_by_token(&concatenated_texts, true)?;

    info!(
        "COMMON: tokens len {:?} and token limit {:?}",
        tokens.len(),
        token_limit
    );
    if tokens.len() > token_limit {
        tokens.truncate(token_limit);
        Ok(tokens.join(""))
    } else {
        Ok(concatenated_texts)
    }
    //info!("COMMON: search content result: {}", search_content);
}

fn concatenate_results(all_search_results: Vec<ScoredPoint>) -> anyhow::Result<String> {
    let mut concatenated_texts: String = String::new();

    for search_result in all_search_results {
        let Some(Kind::StringValue(text)) = &search_result.payload["text"].kind else {
            bail!("Oooops! Error")
        };
        concatenated_texts.push_str(text.as_str());
    }
    Ok(concatenated_texts)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use qdrant_client::qdrant::ScoredPoint;

    use crate::utils::ai_utils::{concatenate_results, update_search_content};

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
