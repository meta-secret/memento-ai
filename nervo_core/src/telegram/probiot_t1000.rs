use std::sync::Arc;

use anyhow::Result;
use teloxide::Bot as TelegramBot;
use teloxide::prelude::*;

use crate::common::AppState;
use crate::telegram::bot_utils::{chat};

/// Start telegram bot
pub async fn start(token: String, app_state: Arc<AppState>) -> Result<()> {
    let bot = TelegramBot::new(token);

    let handler = Update::filter_message().endpoint(chat);

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
