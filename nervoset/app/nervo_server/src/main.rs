use std::sync::Arc;

use async_openai::types::Role;
use axum::{
    http::StatusCode,
    Json,
    Router, routing::{get, post},
};
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};

use nervo_bot_core::ai::ai_db::NervoAiDb;
use nervo_bot_core::ai::nervo_llm::{LlmChat, LlmMessage, NervoLlm};
use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::db::local_db::LocalDb;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

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
        .route("/chat/:user_id", get(chat))
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
    Path(user_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<LlmChat>, StatusCode> {
    // LLM interacting

    //TODO получить историю сообщений из базы данных

    // Формируем объект сообщения
    let chat = LlmChat {
        user: user_id,
        messages: vec![LlmMessage {
            role: Role::User,
            content: String::from("hey hey"),
        }],
    };

    Ok(Json(chat))
}

async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(msg): Json<LlmMessage>,
) -> Result<Json<LlmMessage>, StatusCode> {
    let reply = state.nervo_llm.send_msg(msg)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    //TODO Сохранить сообщение в базу данных

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
