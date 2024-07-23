use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use tracing::{error, info};
use nervo_api::{LlmChat, LlmMessage};
use nervo_bot_core::common::AppState;

pub async fn chat(
    Path(chat_id): Path<u64>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<LlmChat>, StatusCode> {
    // LLM interacting
    info!("Read messages from DB");
    let cached_messages: Vec<LlmMessage> = state
        .local_db
        .read_from_local_db(format!("{}", chat_id).as_str())
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("CACHED MESSAGES {:?}", &cached_messages);
    let chat = LlmChat { chat_id, messages: cached_messages };
    info!("CHAT {:?}", &chat);
    Ok(Json(chat))
}