use crate::config::jarvis::JarvisAppState;
use crate::context::conclusions::ConclusionsService;
use crate::context::permanent_memory::MemoryCell;
use crate::context::user_context::UserContext;
use crate::utils::ai_utils_data::system_role::{RolePathBuilder, RoleType};
use crate::utils::date_time_utils::get_time_stamp;
use nervo_sdk::agent_type::AgentType;
use qdrant_client::qdrant::SearchResponse;
use std::sync::Arc;
use teloxide::prelude::*;
use tracing::log::info;

pub struct UserContextMainHandler {
    user_context: UserContext,
}

impl UserContextMainHandler {
    pub fn new() -> Self {
        Self {
            user_context: UserContext::default(),
        }
    }
    pub async fn use_memory_in_conversation(
        &self,
        msg: &Message,
        app_state: Arc<JarvisAppState>,
        agent_type: AgentType,
    ) -> anyhow::Result<String> {
        info!("Start speak_with_memory");
        let timestamp = get_time_stamp();
        let user_raw_request = msg.text().unwrap_or("Empty request");
        let timestamped_user_raw_request = format!("[{}] {}]", timestamp, user_raw_request);
        let user_id = msg.clone().from.map(|user| user.id.0).unwrap_or(0) as i64;
        let user_collection_name = user_id.to_string();

        let conclusions_service =
            ConclusionsService::new(user_collection_name, app_state.clone(), agent_type).await?;

        let content_insights = conclusions_service
            .search_conclusions_by_user_request(&timestamped_user_raw_request)
            .await?;
        info!(
            "keywords was found: {:?} keywords",
            content_insights.keywords.len()
        );

        let vectorized_request = app_state
            .clone()
            .nervo_llm
            .embedding(timestamped_user_raw_request.as_str())
            .await?;
        let vectorized_request = vectorized_request
            .data
            .into_iter()
            .next()
            .unwrap()
            .embedding;
        info!("User request has been vectorized");

        let qdrant_data_for_user_request = app_state
            .nervo_ai_db
            .qdrant
            .vector_search(
                conclusions_service
                    .user_conclusions_collection_name
                    .as_str(),
                vectorized_request,
                5,
            )
            .await?;
        info!("User request vectors has been found");

        let llm_request_response = self
            .system_role_communication(
                &msg,
                timestamped_user_raw_request.as_str(),
                content_insights.conclusions,
                qdrant_data_for_user_request,
                &conclusions_service,
            )
            .await?;

        self.update_memory(
            timestamp.as_str(),
            user_raw_request,
            llm_request_response.as_str(),
            conclusions_service
                .user_conclusions_collection_name
                .as_str(),
            msg,
            &conclusions_service,
        )
        .await?;

        Ok(llm_request_response)
    }

    async fn update_memory(
        &self,
        timestamp: &str,
        user_raw_request: &str,
        llm_request_response: &str,
        user_collection_name: &str,
        msg: &Message,
        conclusions_service: &ConclusionsService,
    ) -> anyhow::Result<()> {
        info!("Updating memory");
        MemoryCell::create_memory_cell(
            conclusions_service.app_state.clone(),
            timestamp,
            user_raw_request,
            llm_request_response,
            user_collection_name,
        )
        .await?;

        conclusions_service
            .set_conclusion(
                &user_raw_request,
                &self.user_context,
                &timestamp,
                msg.chat.id,
            )
            .await?;

        let timestamped_user_raw_request = format!("[{}] {}]", timestamp, user_raw_request);
        self.user_context.add_user_interaction_to_dialogue(
            timestamped_user_raw_request.as_str(),
            &msg.chat.id,
            llm_request_response,
            String::from(timestamp),
        );
        info!("Updating memory is ok");
        Ok(())
    }

    async fn system_role_communication(
        &self,
        msg: &Message,
        timestamped_user_raw_request: &str,
        conclusions: Vec<String>,
        qdrant_data_for_user_request: SearchResponse,
        conclusions_service: &ConclusionsService,
    ) -> anyhow::Result<String> {
        let current_dialogue_cache = self.user_context.get_dialogue_string(&msg.chat.id);
        let llm_request_message = format!(
            "Текущий запрос пользователя: {}\
        \nКраткосрочный кэш сообщений: {}\
        \nРелевантные факты о пользователе: {:?}\
        \nКэш релевантных сообщений по теме запроса: {:?}",
            timestamped_user_raw_request,
            current_dialogue_cache,
            conclusions,
            &qdrant_data_for_user_request
        );
        info!("llm_request_message: {:?}", llm_request_message);

        let role_path_builder = RolePathBuilder {
            agent_type: conclusions_service.agent_type.clone(),
            role_type: RoleType::AssistantMemory,
        };

        let system_role = role_path_builder.resource_path_content()?;

        let llm_request_response = conclusions_service
            .app_state
            .nervo_llm
            .raw_llm_processing(system_role.as_str(), llm_request_message.as_str())
            .await?;
        info!("llm_request_response: {:?}", llm_request_response);

        Ok(llm_request_response)
    }
}
