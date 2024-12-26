use std::sync::Arc;
use tracing::log::info;
use crate::config::jarvis::JarvisAppState;

pub struct MemoryCell {
}

impl MemoryCell {
    pub async fn create_memory_cell(
        app_state: Arc<JarvisAppState>,
        timestamp: &str,
        user_request: &str,
        llm_request_response: &str,
        collection_name: &str,
    ) -> anyhow::Result<()> {
        let memory_cell = format!(
            "{{{}: [{{\"role\": \"user\", \"content\": {:?}}}, {{\"role\": \"Leo (You)\", \"content\": {:?}}}]}}",
            timestamp,
            user_request,
            llm_request_response
        );
        info!("memory cell check: {}", memory_cell);

        app_state.nervo_ai_db.qdrant.save_text(
            collection_name,
            memory_cell.as_str(),
        ).await?;

        Ok(())
    }
}