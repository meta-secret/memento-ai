use nervo_bot_core::common::{AppState, NervoConfig};
use nervo_bot_core::telegram::probiot_t1000;

use std::sync::Arc;

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
    // match start_probiot().instrument(debug_span!("probiot")).await {
    //     Ok(_) => {
    //         info!("OK");
    //         Ok(())
    //     }
    //     Err(err) => {
    //         info!("fuck! {:?}", err);
    //         Ok(())
    //     }
    // }
}

pub async fn start_probiot() -> anyhow::Result<()> {
    let nervo_config = NervoConfig::load()?;
    let bot_token = nervo_config.telegram.token.clone();
    let app_state = Arc::from(AppState::try_from(nervo_config)?);

    probiot_t1000::start(bot_token, app_state).await?;

    Ok(())
}
