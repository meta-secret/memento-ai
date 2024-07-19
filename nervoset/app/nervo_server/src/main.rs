mod commands;
mod queries;
mod cors;

use std::sync::Arc;
use axum::{routing::{get, post}, Router, Json};
use http::{StatusCode, Uri};
use serde_derive::Serialize;
use tokio::net::TcpListener;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use nervo_bot_core::common::{AppState, NervoConfig};
use crate::commands::send_message;
use crate::queries::chat;
use tower_http::cors::{CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"))
        .add_directive("hyper=info".parse()?)
        .add_directive("h2=info".parse()?)
        .add_directive("tower=info".parse()?)
        .add_directive("sqlx=info".parse()?);

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // Use a more compact, abbreviated log format
        .compact()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // completes the builder.
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Server...");

    let app_state = {
        info!("Loading config...");
        let nervo_config = NervoConfig::load()?;
        Arc::from(AppState::try_from(nervo_config)?)
    };

    info!("Creating router...");
    let cors = CorsLayer::permissive();

    info!("Creating router...");
    let app = Router::new()
        .route("/chat/:chat_id", get(chat))
        .route("/send_message", post(send_message))
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
