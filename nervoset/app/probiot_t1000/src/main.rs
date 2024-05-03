use nervo_bot_core::ai::ai_db::NervoAiDb;
use nervo_bot_core::ai::nervo_llm::NervoLlm;
use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::telegram::probiot_t1000;

use std::sync::Arc;

use nervo_bot_core::db::local_db::LocalDb;
use tracing::{debug_span, info, Instrument, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Define a filter that excludes logs from the particular crate
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

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("Starting Probiot...");
    start_probiot().instrument(debug_span!("probiot")).await?;
    Ok(())
}

pub async fn start_probiot() -> anyhow::Result<()> {
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

    probiot_t1000::start(bot_token, app_state).await?;

    Ok(())
}
