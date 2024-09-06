use crate::config::jarvis::JarvisAppState;
use crate::telegram::bot_utils::MessageTranscriptionType::{Stt, Tts};
use crate::telegram::bot_utils::{
    start_conversation, system_message, transcribe_message, SystemMessage,
};
use crate::telegram::message_parser::MessageParser;
use nervo_sdk::agent_type::AgentType;
use std::sync::Arc;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::Bot;
use tracing::info;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum JarvisOwnerCommands {
    #[command(description = "Get whitelisted users list")]
    GetWhiteListMembers,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum JarvisCommands {
    #[command(description = "Ai model name.")]
    Model,
    Start,
    Manual,
}

pub async fn owner_command_handler(
    bot: Bot,
    msg: Message,
    cmd: JarvisOwnerCommands,
) -> anyhow::Result<()> {
    match cmd {
        JarvisOwnerCommands::GetWhiteListMembers => {
            let formatted_usernames = WHITELIST_MEMBERS
                .iter()
                .map(|username| format!("@{}", username).to_string())
                .collect::<Vec<String>>();
            bot.send_message(msg.chat.id, formatted_usernames.join("\n"))
                .await?;
            Ok(())
        }
    }
}

pub async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: JarvisCommands,
    app_state: Arc<JarvisAppState>,
    agent_type: AgentType,
) -> anyhow::Result<()> {
    match cmd {
        JarvisCommands::Model => {
            bot.send_message(
                msg.chat.id,
                format!("LLM model: {}", app_state.nervo_llm.model_name()),
            )
            .await?;
            Ok(())
        }
        JarvisCommands::Start => {
            system_message(&bot, &msg, SystemMessage::Start(agent_type)).await?;
            Ok(())
        }
        JarvisCommands::Manual => {
            system_message(&bot, &msg, SystemMessage::Manual(agent_type)).await?;
            Ok(())
        }
    }
}

pub async fn handle_callback_query(
    bot: Bot,
    q: CallbackQuery,
    app_state: Arc<JarvisAppState>,
) -> anyhow::Result<()> {
    if let Some(data) = q.data {
        if let Some(message) = q.message {
            if let Some(regular_message) = message.regular_message() {
                if data == Stt.as_str() {
                    info!("STT");
                    transcribe_message(app_state, &bot, regular_message, Stt).await?;
                } else if data == Tts.as_str() {
                    info!("TTS");
                    transcribe_message(app_state, &bot, regular_message, Tts).await?;
                }
            }
        }
    }
    Ok(())
}

pub async fn chat(
    bot: Bot,
    msg: Message,
    app_state: Arc<JarvisAppState>,
    agent_type: AgentType,
) -> anyhow::Result<()> {
    info!("Start chat...");
    let mut parser = MessageParser {
        bot: &bot,
        msg: &msg,
        app_state: &app_state,
        is_voice: false,
    };

    // Get info about bot and user
    let bot_info = &bot.get_me().await?;
    let bot_name = bot_info.clone().user.username.unwrap();
    let user = parser.parse_user().await?;
    let UserId(user_id) = user.id;

    info!("Bot info {}", bot_name);
    start_conversation(
        &app_state, &bot, user_id, &msg, bot_name, agent_type, parser,
    )
    .await?;
    Ok(())
}

// pub async fn permission_restricted(bot: Bot, msg: Message) -> Result<()> {
//     bot.send_message(
//         msg.chat.id,
//         "сорян, я пока не работаю, приходите через 2 мес.",
//     )
//     .await?;
//     Ok(())
// }

pub static WHITELIST_MEMBERS: [&str; 0] = [
    // "afazulzyanov",
];
