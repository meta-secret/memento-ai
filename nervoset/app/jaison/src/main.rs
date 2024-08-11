mod ask_question;
mod creation;
mod edu_conv;
mod user_interaction;
mod utils;

use crate::user_interaction::user_interaction;
use crate::utils::{send_creation_menu, send_main_menu};
use anyhow::Result;
use dotenv::dotenv;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode};
use tokio::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info"))
        .init();

    info!("Starting jAIson...");

    let bot = Bot::from_env();

    let app_state = Arc::new(AppState {
        user_states: Mutex::new(HashMap::new()),
    });

    let cmd_handler = Update::filter_message()
        .filter_command::<JaisonCommands>()
        .endpoint(command_handler);

    let chat_handler = Update::filter_message().endpoint(user_interaction);

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

struct AppState {
    user_states: Mutex<HashMap<i64, UserState>>,
}

#[derive(Default)]
struct UserState {
    awaiting_start_option_choice: bool,
    education: bool,
    awaiting_question: bool,
    current_chapter: usize,
    creation: bool,
    awaiting_sphere_description: bool,
    awaiting_behavior_model_description: bool,
    system_role_creation: bool,
    db_operation: bool,
    db_topping_up: bool,
    last_bot_message: Option<String>,
    language_code: Option<String>,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum JaisonCommands {
    Start,
    Create,
    Help,
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: JaisonCommands,
    app_state: Arc<AppState>,
) -> Result<()> {
    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;
    let mut user_states = app_state.user_states.lock().await;
    let state = user_states.entry(user_id).or_insert(UserState::default());

    if state.language_code.is_none() {
        state.language_code = msg.from().and_then(|user| user.language_code.clone());
    }

    let language_code = match state.language_code.as_deref() {
        Some("ru") => "ru".to_string(),
        _ => "en".to_string(),
    };

    // let language_code = state
    //     .language_code
    //     .clone()
    //     .unwrap_or_else(|| "ru".to_string());

    match cmd {
        JaisonCommands::Start => {
            state.awaiting_start_option_choice = true;
            state.current_chapter = 1;

            let start_message = fs::read_to_string("resources/start_message.txt")
                .map_err(|e| format!("Failed to read 'start_message': {}", e))
                .unwrap();
            bot.send_message(msg.chat.id, start_message)
                .parse_mode(ParseMode::Html)
                .await?;

            let start_pic_file =
                InputFile::file(Path::new("resources/photo_2023-10-16 18.49.28.jpeg"));
            bot.send_photo(msg.chat.id, start_pic_file)
                .caption("Brought to you by\n\n// nervoset, 2024 | @night_intelligence")
                .await?;

            send_main_menu(&bot, msg.chat.id, language_code).await?;
        }

        JaisonCommands::Create => {
            state.awaiting_start_option_choice = false;
            state.creation = true;

            let creation_mode_start_msg =
                fs::read_to_string("resources/creation_resources/creation_mode_msg.txt")
                    .map_err(|e| format!("Failed to read 'start_message': {}", e))
                    .unwrap();
            bot.send_message(msg.chat.id, creation_mode_start_msg)
                .parse_mode(ParseMode::Html)
                .await?;

            send_creation_menu(&bot, msg.chat.id, language_code).await?;
        }

        JaisonCommands::Help => {
            let help_message = fs::read_to_string("resources/help_message.txt")
                .map_err(|e| format!("Failed to read 'help_message': {}", e))
                .unwrap();
            bot.send_message(msg.chat.id, help_message)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }
    Ok(())
}
