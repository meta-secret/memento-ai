use async_openai::config::OpenAIConfig;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Error;

use config::Config as AppConfig;

use crate::ai::open_ai::NervoAiClient;
use crate::common::AppState;
use crate::telegram::nervo_bot;

pub async fn start_nervo_bot() -> Result<Result<(), Error>, Error> {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let app_config = AppConfig::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize::<HashMap<String, String>>()?;

    let api_key = app_config.get("openai_api_key").unwrap();
    let open_ai_config = {
        let cfg = OpenAIConfig::new();
        cfg.with_api_key(api_key)
    };

    let nervo_ai = NervoAiClient::from(open_ai_config);

    let token = app_config.get("telegram_bot_token").unwrap().clone();
    let app_state = Arc::from(AppState { nervo_ai });

    nervo_bot::start(token, app_state).await?;

    Ok(Ok(()))
}
