mod commands;
mod queries;
mod models;
mod cors;

use std::sync::Arc;
use anyhow::bail;
use axum::{ routing::{get, post}, Router, };
use serde::{Deserialize, Serialize};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use nervo_bot_core::ai::ai_db::NervoAiDb;
use nervo_bot_core::ai::nervo_llm::{LlmChat, LlmMessage, NervoLlm};
use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::db::local_db::LocalDb;
use crate::commands::send_message;
use crate::queries::chat;
use tower_http::cors::{Any, Cors, CorsLayer};
use tower_http::trace::TraceLayer;
use crate::cors::cors_middleware;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
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

    // let cors = CorsLayer::new()
    //     .allow_headers(Any)
    //     .allow_methods(Any)
    //     .allow_origin(Any)
    //     .expose_headers(Any)
    //     .allow_headers(vec![http::header::CONTENT_TYPE]);

    info!("App State...");
    let app = Router::new()
        .route("/chat/:chat_id", get(chat))
        .route("/send_message", post(send_message))
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(result) => {result}
        Err(err) => {
            error!("ERROR!!!: {:?}", err);
            bail!("ERROR!!!: {:?}", err)
        }
    };
    axum::serve(listener, app).await?;

    Ok(())
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