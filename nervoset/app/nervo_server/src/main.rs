use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, Level};
use tracing_subscriber::FmtSubscriber;

use nervo_bot_core::ai::ai_db::NervoAiDb;
use nervo_bot_core::ai::nervo_llm::{LlmChat, LlmMessage, NervoLlm};
use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::db::local_db::LocalDb;
use nervo_bot_core::models::nervo_message_model::TelegramMessage;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::INFO)
        // completes the builder.
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    info!("Starting Server...");

    let nervo_config = NervoConfig::load()?;

    let nervo_llm = NervoLlm::from(nervo_config.llm.clone());

    let nervo_ai_db = NervoAiDb::try_from(&nervo_config.qdrant)?;

    let local_db = LocalDb::try_init(nervo_config.database.clone()).await?;

    let app_state = Arc::from(AppState {
        nervo_llm,
        nervo_ai_db,
        local_db,
        nervo_config,
    });

    let app = Router::new()
        .route("/", get(root))
        .route("/chat/:chat_id", get(chat)) // Chat id should be like "${chat_id}_${user_id}"
        .route("/send_message", post(send_message))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Nervo!!!"
}

async fn chat(
    Path(chat_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<LlmChat>, StatusCode> {
    // LLM interacting
    info!("Read messages from DB");
    let cached_messages: Vec<LlmMessage> = state
        .local_db
        .read_from_local_db(&chat_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let chat = LlmChat {
        chat_id: chat_id.parse().expect("Failed to parse string"),
        messages: cached_messages,
    };

    Ok(Json(chat))
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(msg_request): Json<SendMessageRequest>,
) -> Result<Json<LlmMessage>, StatusCode> {
    info!("Save message to DB");
    let user_id_number: u64 = msg_request.user_id.parse().expect("Failed to parse string");
    let chat_id_number: u64 = msg_request.chat_id.parse().expect("Failed to parse string");
    let table_name = format!("{:?}_{:?}", msg_request.chat_id, msg_request.user_id);
    state
        .local_db
        .save_to_local_db(msg_request.llm_message.clone(), &table_name, true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reply = state
        .nervo_llm
        .send_msg(
            msg_request.llm_message.clone(),
            chat_id_number,
            user_id_number,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(reply))
}

// the input to our `create_user` handler
#[derive(Serialize, Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Serialize, Deserialize)]
struct ChatParams {
    user_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct SendMessageRequest {
    user_id: String,
    chat_id: String,
    llm_message: LlmMessage,
}
