use nervo_bot_core::telegram::jarvis;

use std::sync::Arc;

use clap::Parser;
use nervo_bot_core::config::common::NervoConfig;
use nervo_bot_core::config::jarvis::JarvisAppState;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
use tracing::{debug_span, info, Instrument, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, required = true)]
    agent_type: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Jarvis as {:?} ...", args.agent_type);
    start_jarvis(NervoAgentType::try_from(args.agent_type.as_str()))
        .instrument(debug_span!("jarvis"))
        .await?;
    Ok(())
}

pub async fn start_jarvis(agent_type: AgentType) -> anyhow::Result<()> {
    let nervo_config = NervoConfig::load()?;
    let app_state = Arc::from(JarvisAppState::try_from(nervo_config.apps.jarvis)?);

    let telegram_agent_params = nervo_config.telegram.clone().agent_params(agent_type)?;

    jarvis::start(telegram_agent_params, app_state, agent_type).await?;

    Ok(())
}
