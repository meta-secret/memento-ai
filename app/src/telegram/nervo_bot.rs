use crate::common::AppState;
use std::sync::Arc;
use teloxide::Bot as TelegramBot;
use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::types::{MediaKind, MessageKind};
use teloxide::update_listeners::webhooks;

/// Start telegram bot
pub async fn start(token: String, app_state: Arc<AppState>) -> anyhow::Result<()> {
    let bot = TelegramBot::new(token);

    let handler = {
        let cmd_handler = Update::filter_message()
            .filter_command::<Command>()
            .endpoint(endpoint);

        let msg_handler = Update::filter_message().endpoint(message);

        Update::filter_message()
            .branch(cmd_handler)
            .branch(msg_handler)
    };

    //let port: u16 = 3000;
    //let host = "localhost";
    //let url = format!("https://{host}/webhook").parse().unwrap();

    //let addr = ([0, 0, 0, 0], port).into();

    /*
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");
    */

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![app_state])
        .enable_ctrlc_handler()
        .build()
        //.dispatch_with_listener(listener, Arc::new(|err| async move {
        //    log::error!("An error occurred: {:?}", err);
        //}))
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
    #[command(description = "Send message")]
    Msg(String),
}

async fn message(bot: Bot, msg: Message, app_state: Arc<AppState>) -> ResponseResult<()> {
    let MessageKind::Common(msg_common) = &msg.kind else {
        bot.send_message(msg.chat.id, "Unsupported message type.")
            .await?;
        return respond(());
    };

    let MediaKind::Text(media_text) = &msg_common.media_kind else {
        bot.send_message(msg.chat.id, "Unsupported message content type.")
            .await?;
        return respond(());
    };

    chat_gpt_conversation(bot, msg.chat.id, app_state, media_text.text.clone()).await
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
            respond(())
        }
        Command::Msg(msg_text) => {
            chat_gpt_conversation(bot, msg.chat.id, app_state, msg_text).await
        }
    }
}

async fn chat_gpt_conversation(bot: Bot, chat_id: ChatId, app_state: Arc<AppState>, msg_text: String) -> ResponseResult<()> {
    if msg_text.is_empty() {
        bot.send_message(chat_id, "Please provide a message to send.")
            .await?;
        return respond(());
    }
    let reply = app_state
        .nervo_ai
        .chat(msg_text)
        .await
        .unwrap()
        .unwrap_or(String::from("I'm sorry, internal error."));

    bot.send_message(chat_id, reply).await?;

    respond(())
}
