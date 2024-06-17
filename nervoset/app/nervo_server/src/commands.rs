use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tracing::info;
use nervo_bot_core::ai::nervo_llm::LlmMessage;
use nervo_bot_core::common::AppState;
use crate::models::server_message_model::SendMessageRequest;

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(msg_request): Json<SendMessageRequest>,
) -> Result<Json<LlmMessage>, StatusCode> {
    info!("Save message to DB");
    let user_id_number: u64 = msg_request.llm_message.sender_id;
    let chat_id_number: u64 = msg_request.chat_id.parse().expect("Failed to parse string");
    let table_name = msg_request.chat_id; //let table_name = format!("user_{}_chat_{}", user_id_number, chat_id_number);
    info!("table name {:?}", &table_name);
    state
        .local_db
        .save_to_local_db(msg_request.llm_message.clone(), &table_name, false)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("STATE");
    let reply = state
        .nervo_llm
        .send_msg(
            msg_request.llm_message.clone(),
            chat_id_number,
            user_id_number,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("REPLY: {:?}", reply);
    
    state
        .local_db
        .save_to_local_db(reply.clone(), &table_name, false)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    info!("reply {:?}", reply);
    Ok(Json(reply))
}