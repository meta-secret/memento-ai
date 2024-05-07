use std::sync::Arc;

use crate::ai::ai_db::NervoAiDb;
use tracing::info;

use crate::ai::nervo_llm::NervoLlm;
use crate::common::{AppState, NervoConfig};
use crate::db::local_db::LocalDb;
use crate::telegram::r2_d2;

pub async fn start_nervo_bot() -> anyhow::Result<()> {
    info!("Starting command bot...");

    let nervo_config = NervoConfig::load()?;

    let nervo_llm = NervoLlm::from(nervo_config.llm.clone());

    let nervo_ai_db = NervoAiDb::try_from(&nervo_config.qdrant)?;

    let local_db = LocalDb::try_init(nervo_config.database.clone()).await?;

    let bot_token = nervo_config.telegram.token.clone();

    let app_state = Arc::from(AppState {
        nervo_llm,
        nervo_ai_db,
        local_db,
        nervo_config,
    });

    r2_d2::start(bot_token, app_state).await?;

    Ok(())
}
