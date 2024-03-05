use std::sync::Arc;

use anyhow::Result;
use teloxide::Bot as TelegramBot;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;

use crate::common::AppState;
use crate::telegram::bot_utils::chat;

#[derive(BotCommands, Clone)]
#[command(
rename_rule = "lowercase",
description = "These commands are supported:"
)]
enum ProbiotCommands {
    #[command(description = "Ai model name.")]
    Model
}

/// Start telegram bot
pub async fn start(token: String, app_state: Arc<AppState>) -> Result<()> {
    let bot = TelegramBot::new(token);

    let handler = {
        let cmd_handler = Update::filter_message()
            .filter_command::<ProbiotCommands>()
            .endpoint(command_handler);

        let msg_handler = Update::filter_message()
            .endpoint(chat);

        Update::filter_message()
            .branch(cmd_handler)
            .branch(msg_handler)
    };

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn command_handler(bot: Bot, msg: Message, cmd: ProbiotCommands, app_state: Arc<AppState>) -> Result<()> {
    match cmd { 
        ProbiotCommands::Model => {
            bot.send_message(msg.chat.id, format!("LLM model: {}", app_state.nervo_llm.model_name()))
                .await?;
            Ok(())
        } 
    }
}
