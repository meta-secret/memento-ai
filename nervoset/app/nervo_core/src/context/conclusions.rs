use std::sync::Arc;
use crate::config::jarvis::JarvisAppState;
use crate::utils::ai_utils::{filter_search_result};
use crate::telegram::bot_utils::{get_message_related_points, get_payload};
use crate::utils::ai_utils_data::TruncatingType;
use serde_json::Value;
use teloxide::types::ChatId;
use tracing::info;
use nervo_sdk::agent_type::AgentType;
use crate::context::user_context::UserContext;
use crate::utils::ai_utils_data::SortingType::Ascending;
use crate::utils::ai_utils_data::system_role::{RolePathBuilder, RoleType};

pub struct ConclusionsManager {
    pub keywords: Vec<String>,
    pub conclusions: Vec<String>,
}

impl ConclusionsManager {
    pub async fn search_keywords_by_conclusions(
        user_collection_name: &str,
        app_state: Arc<JarvisAppState>,
        agent_type: AgentType,
        timestamped_user_raw_request: &str,
    ) -> anyhow::Result<ConclusionsManager> {
        info!("Searching conclusions for keywords");
        let conclusions_json = Self::get_conclusions_for_user_message(
            &timestamped_user_raw_request,
            app_state.clone(),
            agent_type.clone()
        ).await?;

        let instructions_value: Value = serde_json::from_str(&conclusions_json)?;
        let keywords: Vec<String> = match instructions_value.get("keywords") {
            Some(Value::Array(keyword_array)) => keyword_array
                .iter()
                .filter_map(|kw| kw.as_str().map(|s| s.to_string()))
                .collect(),
            _ => Vec::new(),
        };
        info!("{} keywords was found", keywords.len());
        
        let user_conclusions_collection_name = format!("{}_conclusions", user_collection_name);

        let path_builder = RolePathBuilder {
            agent_type,
            role_type: RoleType::Clearing,
        };

        let system_role_to_clear_request = path_builder.resource_path_content()?;
        
        let mut all_possible_conclusions = Vec::new();
        for keyword in &keywords {
            all_possible_conclusions = get_message_related_points(
                keyword,
                user_conclusions_collection_name.as_str(),
                system_role_to_clear_request.as_str(),
                app_state.clone(),
            ).await?;
        }

        Ok(ConclusionsManager { 
            keywords,
            conclusions: all_possible_conclusions,
        })
    }
    
    async fn get_conclusions_for_user_message(
        timestamped_user_raw_request: &str,
        app_state: Arc<JarvisAppState>,
        agent_type: AgentType,
    ) -> anyhow::Result<String> {
        info!("Create conclusions for user message by llm");
        
        let conclusion_system_role = {
            let role_path_builder = RolePathBuilder {
                agent_type,
                role_type: RoleType::ConclusionsPreprocessing,
            };
            
            role_path_builder.resource_path_content()?
        };

        let keywords_json = app_state.nervo_llm.raw_llm_processing(
            conclusion_system_role.as_str(),
            timestamped_user_raw_request
        ).await?;
        
        info!("{} => conclusions was found", keywords_json);
        Ok(keywords_json)
    }

    pub async fn set_conclusion(
        app_state: Arc<JarvisAppState>,
        agent_type: AgentType,
        user_raw_request: &str,
        user_context: &UserContext,
        timestamp: &str,
        chat_id: ChatId,
        user_collection_name: &str,
    ) -> anyhow::Result<()> {

        if let Some(prev_response) = user_context.last_llm_response(&chat_id) {
            let conclusion_message = format!(
                "Твоё предыдущее сообщение: {}\
                \nОтвет пользователя (реакция) на твоё сообщение: {}",
                prev_response, user_raw_request
            );
            info!("Previous conclusion message: {}", conclusion_message);
            
            let role_path_builder = RolePathBuilder {
                agent_type,
                role_type: RoleType::SearchKeywords,
            };

            let conclusion_system_role = role_path_builder.resource_path_content()?;

            let conclusions_list_str = app_state.nervo_llm.raw_llm_processing(
                conclusion_system_role.as_str(),
                conclusion_message.as_str()
            ).await?;
            
            let conclusions_list_json: Value = serde_json::from_str(&conclusions_list_str)?;

            let conclusions_keywords_for_struct: Vec<String> =
                match conclusions_list_json.get("conclusions") {
                    Some(Value::Array(keyword_array)) => keyword_array
                        .iter()
                        .filter_map(|kw| kw.as_str().map(|s| s.to_string()))
                        .collect(),
                    _ => Vec::new(),
                };

            info!(
            "Conclusions keywords for searching in user's db: {:?}",
            conclusions_keywords_for_struct
        );

            if conclusions_keywords_for_struct.is_empty() ||
                conclusions_keywords_for_struct == vec!["None".to_string()]
            {
                info!("No useful info in current interaction.");
                return Ok(());
            }

            let user_conclusions_collection_name = format!("{}_conclusions", user_collection_name);
            for conclusion in &conclusions_keywords_for_struct {
                let conclusion_vector = app_state.nervo_llm.embedding(conclusion).await?;

                let search_result = app_state.nervo_ai_db.qdrant.vector_search(
                    user_conclusions_collection_name.as_str(),
                    conclusion_vector.data.into_iter().next().unwrap().embedding,
                    1
                ).await?;

                let search_result = search_result.result;
                let filtered_search_result = filter_search_result(
                    search_result,
                    Ascending,
                    TruncatingType::None,
                    0.7
                )?;
                let payload = get_payload(filtered_search_result)?;

                if payload.contains(&"No data found.".to_string()) {
                    let timestamped_conclusion = format!("{}: {}", timestamp, conclusion);
                    app_state.nervo_ai_db.qdrant.save_text(
                        &user_conclusions_collection_name,
                        &timestamped_conclusion
                    ).await?;

                    info!("New conclusion saved to the vector database.");
                } else {
                    info!("Conclusion is already known info.");
                }
            }
        }
        Ok(())
    }
}