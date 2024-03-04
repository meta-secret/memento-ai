use std::sync::Arc;

use anyhow::bail;
use anyhow::Result;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs,
};
use qdrant_client::qdrant::value::Kind;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{File, MediaKind, MessageKind, ReplyMarkup, User};
use teloxide::Bot as TelegramBot;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::fs;

use crate::common::AppState;
use crate::telegram::tg_keyboard::NervoBotKeyboard;

type MyDialogue = Dialogue<ChatState, InMemStorage<ChatState>>;

trait NervoBot {}

pub struct NervoBotR2D2 {}

#[derive(Clone, Default, Debug)]
pub enum ChatState {
    #[default]
    Initial,
    Conversation,
    Embeddings,
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
    Chat(String),
    #[command(description = "Remember a fact")]
    Save(String),
    #[command(description = "Search in the knowledge database")]
    Search(String),
    #[command(
        description = "Search in the knowledge database and then analyse the result with an llm model."
    )]
    Analyse(String),
    #[command(description = "Lean something new")]
    Learn,
}

/// Start telegram bot
pub async fn start(token: String, app_state: Arc<AppState>) -> Result<()> {
    let bot = TelegramBot::new(token);

    let handler = {
        let cmd_handler = Update::filter_message()
            .filter_command::<Command>()
            .enter_dialogue::<Message, InMemStorage<ChatState>, ChatState>()
            //.branch(dptree::case![State::Chat].endpoint(chat_initialization))
            //.branch(dptree::case![State::ChatContinuation].endpoint(chat_continuation))
            .endpoint(endpoint);

        let msg_handler = Update::filter_message().endpoint(chat);

        let chat_handler = Update::filter_message()
            //.filter_command::<Command>()
            .enter_dialogue::<Message, InMemStorage<ChatState>, ChatState>()
            //.branch(dptree::case![State::Chat].endpoint(chat_initialization))
            .branch(dptree::case![ChatState::Conversation].endpoint(chat_continuation));

        Update::filter_message()
            .branch(chat_handler)
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
        .dependencies(dptree::deps![app_state, InMemStorage::<ChatState>::new()])
        .enable_ctrlc_handler()
        .build()
        //.dispatch_with_listener(listener, Arc::new(|err| async move {
        //    log::error!("An error occurred: {:?}", err);
        //}))
        .dispatch()
        .await;

    Ok(())
}

async fn chat_initialization(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    app_state: Arc<AppState>,
) -> Result<()> {
    bot.send_message(
        msg.chat.id,
        "State:Chat. Welcome to conversational chat. Let's go!",
    )
    .await?;
    dialogue.update(ChatState::Conversation).await?;
    Ok(())
}

async fn chat_continuation(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    app_state: Arc<AppState>,
) -> Result<()> {
    bot.send_message(msg.chat.id, "State:Continuation").await?;
    chat(bot, msg, app_state).await?;
    Ok(())
}

async fn embeddings_continuation(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    app_state: Arc<AppState>,
) -> Result<()> {
    bot.send_message(msg.chat.id, "Embeddings:Continuation")
        .await?;
    chat(bot, msg, app_state).await?;
    Ok(())
}

async fn chat(bot: Bot, msg: Message, app_state: Arc<AppState>) -> Result<()> {
    let (_, text) = parse_user_and_text(&msg).await?;
    if text.is_empty() {
        bot.send_message(msg.chat.id, "Please provide a message to send.")
            .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
            .await?;

        return Ok(());
    }

    let is_moderation_passed = app_state.nervo_llm.moderate(&text).await?;
    if is_moderation_passed {
        let user_msg = ChatCompletionRequestUserMessageArgs::default()
            .content(text)
            .build()?
            .into();

        chat_gpt_conversation(bot, msg.chat.id, app_state, user_msg).await
    } else {
        bot.send_message(
            msg.chat.id,
            "Your message is not allowed. Please rephrase it.",
        )
        .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
        .await?;
        Ok(())
    }
}

async fn endpoint(
    bot: Bot,
    msg: Message,
    cmd: Command,
    dialogue: MyDialogue,
    app_state: Arc<AppState>,
) -> Result<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
                .await?;
            Ok(())
        }
        Command::Chat(msg_text) => {
            /*let user_msg = ChatCompletionRequestUserMessageArgs::default()
                .content(msg_text)
                .build()?
                .into();

            chat_gpt_conversation(bot, msg.chat.id, app_state, user_msg).await*/
            bot.send_message(msg.chat.id, "Сhat gpt готов ответить на ваши ответы!")
                .await?;
            dialogue.update(ChatState::Conversation).await?;
            Ok(())
        }
        Command::Save(text) => {
            let (user, _) = parse_user_and_text(&msg).await?;
            let UserId(user_id) = user.id;

            // do embedding using openai
            let embedding = app_state.nervo_llm.embedding(text.as_str()).await.unwrap();

            //save the embedding into qdrant db
            let response = app_state
                .nervo_ai_db
                .save(user_id, embedding, text)
                .await
                .unwrap();

            bot.send_message(
                msg.chat.id,
                format!("{:?}", response.result.unwrap().status()),
            )
            .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
            .await?;
            Ok(())
        }
        Command::Search(search_text) => {
            let results = vector_search(&msg, app_state, search_text.as_str()).await?;
            for res in results {
                let res_json = serde_json::to_string_pretty(&res).unwrap();
                bot.send_message(msg.chat.id, format!("{:?}", res_json))
                    .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
                    .await?;
            }

            Ok(())
        }
        Command::Analyse(question) => {
            let result_strings = vector_search(&msg, app_state.clone(), question.as_str())
                .await?
                .iter()
                .map(|(_, text)| text.clone())
                .collect::<Vec<String>>();

            let mut messages = vec![];
            for text in &result_strings {
                let user_msg = ChatCompletionRequestUserMessageArgs::default()
                    .content(text.chars().take(1000).collect::<String>())
                    .build()?;
                messages.push(ChatCompletionRequestMessage::from(user_msg));
            }

            let enriched_question = format!(
                "You will be provided messages and you need to answer to the following question \
                by analysing those messages: {}",
                question
            );

            let system_msg = ChatCompletionRequestSystemMessageArgs::default()
                .content(enriched_question)
                .build()?
                .into();
            messages.insert(0, system_msg);

            let reply = app_state
                .nervo_llm
                .chat_batch(messages)
                .await?
                .unwrap_or(String::from("I'm sorry, internal error."));

            //for embeddings in &result_strings {
            //    bot.send_message(msg.chat.id, embeddings).await?;
            //}

            bot.send_message(msg.chat.id, format!("{}", reply))
                .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
                .await?;

            Ok(())
        }
        Command::Learn => {
            let MessageKind::Common(msg_common) = &msg.kind else {
                bot.send_message(msg.chat.id, "Unsupported message type.")
                    .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
                    .await?;
                return Ok(());
            };

            let MediaKind::Document(training_file) = &msg_common.media_kind else {
                bot.send_message(msg.chat.id, "Unsupported message content type.")
                    .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
                    .await?;
                return Ok(());
            };

            let (user, _) = parse_user_and_text(&msg).await?;
            let UserId(user_id) = user.id;

            let file_id = training_file.document.file.id.clone();
            let file: File = bot.get_file(file_id).await?;

            //TODO remove an old file
            let mut dst = fs::File::create(format!("/tmp/{}", user_id)).await?;
            bot.download_file(&file.path, &mut dst).await?;

            //TODO read json file and make test and validation files from it
            //TODO train the model, an example https://github.com/64bit/async-openai/blob/main/examples/fine-tune-cli/src/main.rs

            Ok(())
        }
    }
}

async fn parse_user_and_text(msg: &Message) -> Result<(&User, String)> {
    let MessageKind::Common(msg_common) = &msg.kind else {
        bail!("Unsupported message type.");
    };

    let MediaKind::Text(media_text) = &msg_common.media_kind else {
        bail!("Unsupported message content type.");
    };

    let Some(user) = &msg_common.from else {
        bail!("User not found. We can handle only direct messages.");
    };

    Ok((user, media_text.text.clone()))
}

async fn vector_search(
    msg: &Message,
    app_state: Arc<AppState>,
    search_text: &str,
) -> Result<Vec<(f32, String)>> {
    let (user, _) = parse_user_and_text(&msg).await?;
    let UserId(user_id) = user.id;

    // do embedding using openai
    let embedding = app_state.nervo_llm.embedding(search_text).await?;

    //save the embedding into qdrant db
    let response = app_state.nervo_ai_db.search(user_id, embedding).await?;

    let mut results = vec![];
    for point in response.result {
        //if point.score > 0.5 {
        let maybe_kind = point
            .payload
            .get("text")
            .and_then(|payload_text| payload_text.kind.clone());

        if let Some(Kind::StringValue(txt)) = maybe_kind {
            results.push((point.score, txt.clone()));
        };
        //}
    }

    Ok(results)
}

async fn chat_gpt_conversation(
    bot: Bot,
    chat_id: ChatId,
    app_state: Arc<AppState>,
    msg: ChatCompletionRequestUserMessage,
) -> Result<()> {
    let reply = app_state
        .nervo_llm
        .chat(msg)
        .await?
        .unwrap_or(String::from("I'm sorry, internal error."));

    bot.send_message(chat_id, reply)
        .reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
        .await?;

    Ok(())
}
