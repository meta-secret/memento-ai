use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use nervo_bot_core::config::jarvis::JarvisAppState;
use nervo_bot_core::utils::ai_utils::llm_conversation;
use nervo_sdk::api::spec::{LlmMessage, LlmMessageContent, LlmMessageMetaInfo, LlmMessagePersistence, LlmMessageRole, SendMessageRequest, ServerResponse, UserAction};
use std::sync::Arc;
use tracing::{error, info};

pub async fn send_message(
    State(state): State<Arc<JarvisAppState>>,
    Json(msg_request): Json<SendMessageRequest>,
) -> Result<Json<LlmMessage>, StatusCode> {
    let LlmMessageContent(content) = &msg_request.llm_message.content;

    let is_moderation_passed = state
        .nervo_llm
        .moderate(content.as_str())
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Is moderation passed: {:?}", is_moderation_passed);
    if is_moderation_passed {
        happy_path_of_moderation(state, msg_request).await
    } else {
        fail_path_of_moderation(state, msg_request).await
    }
}

async fn happy_path_of_moderation(
    app_state: Arc<JarvisAppState>,
    msg_request: SendMessageRequest,
) -> Result<Json<LlmMessage>, StatusCode> {
    info!("SERVER: HAPPY PATH");
    let agent_type = msg_request.agent_type;

    let llm_reply = llm_conversation(app_state, msg_request, agent_type)
        .await
        .map_err(|err| {
            error!("Error2 {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("SERVER: reply {:?}", &llm_reply);
    Ok(Json(llm_reply))
}

async fn fail_path_of_moderation(
    app_state: Arc<JarvisAppState>,
    msg: SendMessageRequest,
) -> Result<Json<LlmMessage>, StatusCode> {
    info!("SERVER: FAIL PATH");
    let user_question = {
        // Moderation is not passed
        let question = format!(
            "I have a message from the user, I know the message is unacceptable, \
        can you please read the message and reply that the message is not acceptable. \
        Reply using the same language the massage uses. Here is the message: {:?}",
            &msg.llm_message.content.text()
        );

        let content = LlmMessageContent::from(question.as_str());
        LlmMessage {
            meta_info: LlmMessageMetaInfo {
                sender_id: Some(msg.llm_message.sender_id),
                role: LlmMessageRole::User,
                persistence: LlmMessagePersistence::Temporal,
            },
            content,
        }
    };

    let reply_text = app_state
        .nervo_llm
        .send_msg(user_question, msg.chat_id)
        .await
        .map_err(|err| {
            error!("Error {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    info!("REPLY: {:?}", reply_text.clone());

    let llm_response = LlmMessage {
        meta_info: LlmMessageMetaInfo {
            sender_id: None,
            role: LlmMessageRole::Assistant,
            persistence: LlmMessagePersistence::Temporal,
        },
        content: LlmMessageContent(reply_text),
    };

    Ok(Json(llm_response))
}

pub async fn mini_app_initializing(
    // State(state): State<Arc<JarvisAppState>>,
    Json(user_action): Json<UserAction>,
) -> Result<Json<ServerResponse>, StatusCode> {
    info!("Receiving request from frontend: {:?}", &user_action);
    
    Ok(Json(ServerResponse {
        message: "Hello, nice to see ya".to_string(),
        buttons: vec!["Button 1".to_string(), "Button 2".to_string()],
        action_buttons: vec!["Action button 3".to_string(), "Action button 4".to_string()],
        can_input: false,
    }))
}

pub async fn handle_start_button_click(
    Json(user_action): Json<UserAction>,
) -> Result<Json<ServerResponse>, StatusCode> {
    info!("Receiving request from frontend: {:?}", &user_action);

    Ok(Json(ServerResponse {
        message: "Hello, nice to see ya\nYou are in main menu".to_string(),
        buttons: vec!["Button 1".to_string(), "Button 2".to_string()],
        action_buttons: vec!["Action button 3".to_string(), "Action button 4".to_string()],
        can_input: true,
    }))
}

pub async fn handle_main_menu(
    Json(user_action): Json<UserAction>,
) -> Result<Json<ServerResponse>, StatusCode> {
    info!("Receiving request from frontend: {:?}", &user_action);

    Ok(Json(ServerResponse {
        message: "Some button clicked from main menu\nYou are in main menu".to_string(),
        buttons: vec!["Button 1".to_string(), "Button 2".to_string()],
        action_buttons: vec!["Action button 3".to_string(), "Action button 4".to_string()],
        can_input: true,
    }))
}
