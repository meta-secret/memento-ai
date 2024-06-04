use std::sync::Arc;

use tracing::info;

use crate::common::{AppState, NervoConfig};
use crate::telegram::r2_d2;

pub async fn start_nervo_bot() -> anyhow::Result<()> {
    info!("Starting command bot...");

    let nervo_config = NervoConfig::load()?;

    let bot_token = nervo_config.telegram.token.clone();

    let app_state = Arc::from(AppState::try_from(nervo_config)?);

    r2_d2::start(bot_token, app_state).await?;

    Ok(())
}
