use nervo_bot_core::ai::ai_db::NervoAiDb;
use nervo_bot_core::ai::nervo_llm::{NervoLlm, NervoLlmConfig};
use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::telegram::probiot_t1000;

use std::sync::Arc;

use async_openai::config::OpenAIConfig;
use config::Config as AppConfig;
use nervo_bot_core::db::local_db::LocalDb;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Start Probiot");

    start_probiot().await?;
    Ok(())
}

pub async fn start_probiot() -> anyhow::Result<()> {
    let nervo_config: NervoConfig = {
        let app_config = AppConfig::builder()
            .add_source(config::File::with_name("config"))
            .build()?;
        app_config.try_deserialize()?
    };

    let nervo_llm = {
        let open_ai_config: OpenAIConfig = {
            let cfg = OpenAIConfig::new();
            cfg.with_api_key(nervo_config.openai_api_key.clone())
        };

        let mut llm_config = NervoLlmConfig::from(open_ai_config);
        llm_config = llm_config.with_model_name(nervo_config.model_name.clone());
        llm_config = llm_config.with_max_tokens(nervo_config.max_tokens);
        llm_config = llm_config.with_temperature(nervo_config.temperature);

        NervoLlm::from(llm_config)
    };

    let nervo_ai_db = NervoAiDb::try_from(&nervo_config)?;

    let local_db = LocalDb::try_init(nervo_config.clone()).await?;

    let bot_token = nervo_config.telegram_bot_token.clone();

    let app_state = Arc::from(AppState {
        nervo_llm,
        nervo_ai_db,
        local_db,
        nervo_config,
    });

    probiot_t1000::start(bot_token, app_state).await?;

    Ok(())
}
