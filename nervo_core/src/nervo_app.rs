use async_openai::config::OpenAIConfig;
use std::sync::Arc;

use crate::ai::ai_db::NervoAiDb;
use config::Config as AppConfig;

use crate::ai::nervo_llm::NervoLlm;
use crate::common::{AppState, NervoConfig};
use crate::telegram::r2_d2;
use crate::ai::nervo_llm::NervoLlmConfig;

pub async fn start_nervo_bot() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let nervo_config: NervoConfig = {
        let app_config = AppConfig::builder()
            .add_source(config::File::with_name("config"))
            .build()?;
        app_config.try_deserialize()?
    };

    let open_ai_config = {
        let cfg = OpenAIConfig::new();
        cfg.with_api_key(nervo_config.openai_api_key.clone())
    };
    
    let nervo_llm_config = NervoLlmConfig::from(open_ai_config);

    let nervo_llm = NervoLlm::from(nervo_llm_config);
    let nervo_ai_db = NervoAiDb::try_from(&nervo_config)?;

    let app_state = Arc::from(AppState {
        nervo_llm,
        nervo_ai_db,
    });

    r2_d2::start(nervo_config.telegram_bot_token, app_state).await?;

    Ok(())
}
