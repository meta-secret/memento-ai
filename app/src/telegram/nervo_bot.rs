use std::collections::HashMap;
use crate::common::AppState;
use std::sync::Arc;
use anyhow::bail;
use qdrant_client::qdrant::value::Kind;
use serde_derive::Deserialize;
use teloxide::Bot as TelegramBot;
use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::net::Download;
use teloxide::types::{File, MediaKind, MessageKind};
use tokio::fs;

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
    #[command(description = "Remember a fact")]
    Save(String),
    #[command(description = "Search in the knowledge database")]
    Search(String),
    #[command(description = "Lean something new")]
    Learn,
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
        Command::Save(text) => {
            let MessageKind::Common(common_msg) = msg.kind else {
                bot.send_message(msg.chat.id, "Unsupported message type.")
                    .await?;
                return respond(());
            };

            let Some(user) = common_msg.from else {
                bot.send_message(msg.chat.id, "User not found. We can handle only direct messages.")
                    .await?;
                return respond(());
            };

            let UserId(user_id) = user.id;

            // do embedding using openai
            let embedding = app_state.nervo_llm.embedding(text.clone())
                .await
                .unwrap();

            //save the embedding into qdrant db
            let response = app_state.nervo_ai_db.save(user_id, embedding, text)
                .await
                .unwrap();

            bot.send_message(msg.chat.id, format!("{:?}", response.result.unwrap().status()))
                .await?;
            respond(())
        }
        Command::Search(search_text) => {
            let MessageKind::Common(common_msg) = msg.kind else {
                bot.send_message(msg.chat.id, "Unsupported message type.")
                    .await?;
                return respond(());
            };

            let Some(user) = common_msg.from else {
                bot.send_message(msg.chat.id, "User not found. We can handle only direct messages.")
                    .await?;
                return respond(());
            };

            let UserId(user_id) = user.id;

            // do embedding using openai
            let embedding = app_state.nervo_llm.embedding(search_text.clone())
                .await
                .unwrap();

            //save the embedding into qdrant db
            let response = app_state.nervo_ai_db.search(user_id, embedding)
                .await
                .unwrap();

            let mut results = vec![];
            response.result.iter().for_each(|point| {
                //if point.score > 0.5 {
                if let Kind::StringValue(txt) = point.payload.get("text").unwrap().kind.clone().unwrap() {
                    let trimmed_text: String = txt.chars().take(100).collect();
                    results.push((point.score, trimmed_text));
                };
                //}
            });
            let results = serde_json::to_string_pretty(&results).unwrap();

            bot.send_message(msg.chat.id, format!("{}", results)).await?;

            respond(())
        }
        Command::Learn => {
            let MessageKind::Common(msg_common) = &msg.kind else {
                bot.send_message(msg.chat.id, "Unsupported message type.")
                    .await?;
                return respond(());
            };

            let MediaKind::Document(training_file) = &msg_common.media_kind else {
                bot.send_message(msg.chat.id, "Unsupported message content type.")
                    .await?;
                return respond(());
            };

            let Some(user) = &msg_common.from else {
                bot.send_message(msg.chat.id, "User not found. We can handle only direct messages.")
                    .await?;
                return respond(());
            };
            let UserId(user_id) = user.id;

            let file_id = training_file.document.file.id.clone();
            let file: File = bot.get_file(file_id).await?;

            //TODO remove an old file
            let mut dst = fs::File::create(format!("/tmp/{}", user_id)).await?;
            bot.download_file(&file.path, &mut dst).await?;

            //TODO read json file and make test and validation files from it
            //TODO train the model, an example https://github.com/64bit/async-openai/blob/main/examples/fine-tune-cli/src/main.rs

            respond(())
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
        .nervo_llm
        .chat(msg_text)
        .await
        .unwrap()
        .unwrap_or(String::from("I'm sorry, internal error."));

    bot.send_message(chat_id, reply).await?;

    respond(())
}
