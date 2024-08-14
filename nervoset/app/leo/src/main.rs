use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode};
use tokio::sync::Mutex;
use tracing::{debug_span, info, Instrument};
use tracing_subscriber::EnvFilter;
use nervo_bot_core::config::common::NervoConfig;
use nervo_bot_core::config::nervo_server::NervoServerAppState;
use nervo_bot_core::telegram::{leo, probiot_t1000};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("Starting Leo...");

    start_leo().await?;
    // start_leo().instrument(debug_span!("leo")).await?;

    Ok(())
}


pub async fn start_leo() -> anyhow::Result<()> {
    let nervo_config = NervoConfig::load()?;
    let bot_token = nervo_config.leo.telegram.token.clone();
    let app_state = Arc::from(NervoServerAppState::try_from(nervo_config.nervo_server)?);
    
    leo::start(bot_token, app_state).await?;

    Ok(())
}