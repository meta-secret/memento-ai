use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tracing::{error, info};
use nervo_bot_core::ai::nervo_llm::{LlmMessage, LlmMessageContent, UserLlmMessage};
use nervo_bot_core::common::AppState;
use nervo_bot_core::common_utils::common_utils::{llm_conversation, SendMessageRequest};

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(msg_request): Json<SendMessageRequest>,
) -> Result<Json<LlmMessage>, StatusCode> {
    let user_id_number: u64 = msg_request.llm_message.sender_id;
    let chat_id_number: u64 = msg_request.chat_id;
    
    let LlmMessageContent(content) = &msg_request.llm_message.content;

    let is_moderation_passed = state.nervo_llm.moderate(content.as_str())
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Is moderation passed: {:?}", is_moderation_passed);
    if is_moderation_passed {
        happy_path_of_moderation(state, msg_request).await
    } else {
        fail_path_of_moderation(state, content.as_str(), user_id_number, chat_id_number)
            .await
    }
}

async fn happy_path_of_moderation(
    app_state: Arc<AppState>,
    msg_request: SendMessageRequest
) -> Result<Json<LlmMessage>, StatusCode> {
    info!("Save message to DB");
    let table_name = msg_request.chat_id;
    info!("table name {:?}", table_name);

    // Create question for LLM
    let question_msg = msg_request.llm_message.clone();
    let chat_id = msg_request.chat_id.to_string();

    app_state
        .local_db
        .save_to_local_db(question_msg.clone(), table_name.to_string().as_str(), false)
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    //app_state: &Arc<AppState>, message: &str, chat_id: u64, user_id: u64, table_name: String
    let reply = llm_conversation(&app_state, msg_request, chat_id)
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    app_state
        .local_db
        .save_to_local_db(reply.clone(), table_name.to_string().as_str(), false)
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("reply {:?}", reply);
    Ok(Json(reply))
}

async fn fail_path_of_moderation(app_state: Arc<AppState>, msg_text: &str, user_id: u64, chat_id: u64) -> Result<Json<LlmMessage>, StatusCode> {
    let question_msg = {
        // Moderation is not passed
        let question = format!("I have a message from the user, I know the message is unacceptable, \
        can you please read the message and reply that the message is not acceptable. \
        Reply using the same language the massage uses. Here is the message: {:?}", &msg_text);
        
        let content = LlmMessageContent::from(question.as_str());
        LlmMessage::User(UserLlmMessage { sender_id: user_id, content })
    };

    let reply = app_state
        .nervo_llm
        .send_msg(question_msg, chat_id)
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("REPLY: {:?}", reply);
    Ok(Json(reply))
}