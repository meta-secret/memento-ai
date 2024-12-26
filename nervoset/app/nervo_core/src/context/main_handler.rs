use std::sync::{Arc};
use teloxide::prelude::*;
use tracing::log::{info};
use nervo_sdk::agent_type::{AgentType};
use crate::config::jarvis::JarvisAppState;
use crate::context::conclusions::ConclusionsManager;
use crate::context::permanent_memory::MemoryCell;
use crate::context::user_context::UserContext;
use crate::utils::ai_utils_data::system_role::{RolePathBuilder, RoleType};
use crate::utils::date_time_utils::get_time_stamp;

pub struct UserContextMainHandler {
    user_context: UserContext,
}

impl UserContextMainHandler {
    pub fn new() -> Self {
        Self {
            user_context: UserContext::default(),
        }
    }
    pub async fn speak_with_memory(
        &self,
        msg: &Message,
        agent_type: AgentType,
        app_state: Arc<JarvisAppState>
    ) -> anyhow::Result<String> {
        info!("Start speak_with_memory");
        let timestamp = get_time_stamp();
        let user_raw_request = msg.text().unwrap_or("Empty request");
        let timestamped_user_raw_request = format!("[{}] {}]", timestamp, user_raw_request);
        let user_id = msg.clone().from.map(|user| user.id.0).unwrap_or(0) as i64;
        let user_collection_name = user_id.to_string();
        
        let conclusions_manager = ConclusionsManager::search_keywords_by_conclusions(
            user_collection_name.as_str(),
            app_state.clone(),
            agent_type,
            &timestamped_user_raw_request,
        ).await?;
        info!("keywords was found: {:?}", &conclusions_manager.keywords);
        
        let vectorized_request = app_state.nervo_llm.embedding(timestamped_user_raw_request.as_str()).await?;
        let vectorized_request = vectorized_request.data.into_iter().next().unwrap().embedding;
        info!("User request has been vectorized");
        
        let qdrant_data_for_user_request = app_state.nervo_ai_db.qdrant.vector_search(
            user_collection_name.as_str(),
            vectorized_request,
            5
        ).await?;
        info!("User request vectors has been found");
        
        let current_dialogue_cache = self.user_context.get_dialogue_string(&msg.chat.id);
        // TODO: Need to refactor. Need to think about how to move it in file and read from it.
        let llm_request_message = format!(
            "Текущий запрос пользователя: {}\
        \nКраткосрочный кэш сообщений: {}\
        \nРелевантные факты о пользователе: {:?}\
        \nКэш релевантных сообщений по теме запроса: {:?}",
            timestamped_user_raw_request,
            current_dialogue_cache,
            conclusions_manager.conclusions,
            &qdrant_data_for_user_request
        );
        info!("llm_request_message: {:?}", llm_request_message);
        
        let role_path_builder = RolePathBuilder {
            agent_type,
            role_type: RoleType::AssistantMemory,
        };

        let system_role = role_path_builder.resource_path_content()?;
        
        let llm_request_response = app_state.nervo_llm.raw_llm_processing(
            system_role.as_str(),
            llm_request_message.as_str()
        ).await?;
        info!("llm_request_response: {:?}", llm_request_response);
        
        MemoryCell::create_memory_cell(
            app_state.clone(),
            timestamp.as_str(),
            user_raw_request,
            llm_request_response.as_str(),
            user_collection_name.as_str()
        ).await?;
        
        ConclusionsManager::set_conclusion(
            app_state.clone(),
            agent_type,
            &user_raw_request,
            &self.user_context,
            &timestamp,
            msg.chat.id,
            &user_collection_name
        ).await?;
        
        self.user_context.add_user_interaction_to_dialogue(
            timestamped_user_raw_request.as_str(),
            &msg.chat.id,
            llm_request_response.as_str(),
            timestamp
        );
        
        Ok(llm_request_response)
    }
}