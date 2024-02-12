use crate::common::AppState;
use std::sync::Arc;
use teloxide::{prelude::*, utils::command::BotCommands};

pub async fn start(token: String, app_state: Arc<AppState>) -> anyhow::Result<()> {
    let bot = Bot::new(token);

    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(endpoint);

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "display help.")]
    Help,
    #[command(description = "send message")]
    Msg(String),
}

async fn endpoint(
    bot: Bot,
    msg: Message,
    cmd: Command,
    app_state: Arc<AppState>,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Msg(msg_text) => {
            if msg_text.is_empty() {
                bot.send_message(msg.chat.id, "Please provide a message to send.")
                    .await?;
                return respond(());
            }
            let reply = app_state
                .nervo_ai
                .chat(msg_text)
                .await
                .unwrap()
                .unwrap_or(String::from("I'm sorry, internal error."));

            bot.send_message(msg.chat.id, reply).await?;

            return respond(());
        }
    }

    return respond(());
}
