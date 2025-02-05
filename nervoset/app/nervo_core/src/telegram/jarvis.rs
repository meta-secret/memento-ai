use std::sync::Arc;

use crate::config::common::TelegramBotParams;
use crate::config::jarvis::JarvisAppState;
use crate::telegram::commands_handlers::{
    chat, command_handler, handle_callback_query, owner_command_handler, JarvisCommands,
    JarvisOwnerCommands,
};
use anyhow::Result;
use nervo_sdk::agent_type::NervoAgentType;
use teloxide::prelude::*;
use teloxide::Bot as TelegramBot;

/// Start telegram bot
pub async fn start(
    params: TelegramBotParams,
    app_state: Arc<JarvisAppState>,
    nervo_agent_type: NervoAgentType,
) -> Result<()> {
    let bot = TelegramBot::new(params.token.as_str());
    let agent_type = nervo_agent_type.agent_type;

    app_state.local_db.init_db().await?;

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<JarvisOwnerCommands>()
                .endpoint(owner_command_handler),
        )
        .branch(
            // Handle /commands from chat
            Update::filter_message()
                .filter_command::<JarvisCommands>()
                .endpoint(command_handler),
        )
        .branch(Update::filter_message().endpoint(chat)) // Handle all messages
        .branch(Update::filter_callback_query().endpoint(handle_callback_query)); // Handle button

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![app_state, agent_type])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
