use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use nervo_bot_core::config::jarvis::JarvisAppState;
use nervo_sdk::api::spec::{LlmChat, LlmMessage};
use std::sync::Arc;
use tracing::{error, info};

pub async fn chat(
    Path(chat_id): Path<u64>,
    State(state): State<Arc<JarvisAppState>>,
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
    let chat = LlmChat {
        chat_id: Some(chat_id),
        messages: cached_messages,
    };
    info!("CHAT {:?}", &chat);
    Ok(Json(chat))
}
