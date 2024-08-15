use crate::config::jarvis::JarvisAppState;
use std::sync::Arc;
use teloxide::dispatching::{Dispatcher, HandlerExt, UpdateFilterExt};
use teloxide::error_handlers::LoggingErrorHandler;
use teloxide::macros::BotCommands;
use teloxide::prelude::{Message, Requester, Update};
use teloxide::Bot as TelegramBot;
use teloxide::{dptree, Bot};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum LeoCommands {
    Start,
}

pub async fn start(token: String, app_state: Arc<JarvisAppState>) -> anyhow::Result<()> {
    let bot = TelegramBot::new(token);

    app_state.local_db.init_db().await?;

    let cmd_handler = Update::filter_message()
        .filter_command::<LeoCommands>()
        .endpoint(command_handler);

    let chat_handler = Update::filter_message().endpoint(command_handler);

    let handler = dptree::entry().branch(cmd_handler).branch(chat_handler);

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            teloxide::update_listeners::polling_default(bot).await,
            LoggingErrorHandler::with_custom_text("Dispatcher: an error from the update listener"),
        )
        .await;

    Ok(())
}

async fn command_handler(bot: Bot, msg: Message, cmd: LeoCommands) -> anyhow::Result<()> {
    match cmd {
        LeoCommands::Start => {
            let start_message = "hello...";
            bot.send_message(msg.chat.id, start_message).await?;
        }
    }
    Ok(())
}
