mod commands;
mod queries;

use crate::commands::{handle_main_menu, handle_start_button_click, mini_app_initializing, send_message};
use crate::queries::chat;
use axum::{
    routing::{get, post},
    Json, Router,
};
use http::{StatusCode, Uri};
use nervo_bot_core::config::common::NervoConfig;
use nervo_bot_core::config::jarvis::JarvisAppState;
use serde_derive::Serialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"))
        .add_directive("hyper=info".parse()?)
        .add_directive("h2=info".parse()?)
        .add_directive("tower=info".parse()?)
        .add_directive("sqlx=info".parse()?);
    
    let subscriber = FmtSubscriber::builder()
        .compact()
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Server...");

    let app_state = {
        info!("Loading config...");
        let nervo_config = NervoConfig::load()?;
        Arc::from(JarvisAppState::try_from(nervo_config.apps.jarvis)?)
    };
    
    let cors = CorsLayer::permissive();

    info!("Creating router...");
    let app = Router::new()
        .route("/chat/:chat_id", get(chat))
        .route("/send_message", post(send_message))
        .route("/user_action/mini_app_initializing", post(mini_app_initializing)) // TEMPORARY post, will be changed later
        .route("/user_action/start", post(handle_start_button_click)) // TEMPORARY post, will be changed later
        .route("/user_action/main_menu", post(handle_main_menu)) // TEMPORARY post, will be changed later
        .with_state(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .fallback(not_found_handler);

    let port = 3000;
    info!("Run axum server, on port: {}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{:?}", port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

async fn not_found_handler(uri: Uri) -> (StatusCode, Json<ErrorResponse>) {
    let error_response = ErrorResponse {
        message: format!("404. NervoServer has no route: {uri}"),
    };
    let response = Json(error_response);
    (StatusCode::NOT_FOUND, response)
}
