use async_openai::config::OpenAIConfig;
use std::sync::Arc;

use config::Config as AppConfig;
use crate::ai::ai_db::NervoAiDb;

use crate::ai::nervo_llm::NervoLlm;
use crate::common::AppState;
use crate::telegram::nervo_bot;

pub async fn start_nervo_bot() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let app_config = AppConfig::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let api_key = app_config.get_string("openai_api_key")?;
    let open_ai_config = {
        let cfg = OpenAIConfig::new();
        cfg.with_api_key(api_key)
    };

    let nervo_llm = NervoLlm::from(open_ai_config);
    let nervo_ai_db = NervoAiDb::try_from(&app_config)?;

    let app_state = Arc::from(AppState { nervo_llm, nervo_ai_db });

    let token = app_config.get_string("telegram_bot_token")?;
    nervo_bot::start(token, app_state).await?;

    Ok(())
}
