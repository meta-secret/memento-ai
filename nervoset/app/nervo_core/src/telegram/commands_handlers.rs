use crate::config::jarvis::JarvisAppState;
use crate::models::message_transcription_type::MessageTranscriptionType::{Stt, Tts};
use crate::models::system_messages::SystemMessage;
use crate::telegram::bot_utils::{start_conversation, system_message, transcribe_message};
use crate::telegram::message_parser::MessageParser;
use anyhow::bail;
use nervo_sdk::agent_type::{AgentType, NervoAgentType};
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
    info!("Command handling");
    match &msg.from {
        Some(user) => match &user.language_code {
            Some(locale) => {
                let mut loc_manager = app_state.localisation_manager.write().await;
                info!("User's locale is {}", locale);
                loc_manager.set_language_as_locale(locale.as_str()).await?;
            }
            None => {
                info!("User has no language code");
            }
        },
        None => {
            info!("Message doesn't have a sender");
        }
    }

    match cmd {
        JarvisCommands::Start => {
            system_message(app_state, &bot, &msg, SystemMessage::Start(agent_type)).await?;
        }
        JarvisCommands::Model | JarvisCommands::Manual => {
            if agent_type != AgentType::Kevin {
                match cmd {
                    JarvisCommands::Model => {
                        bot.send_message(
                            msg.chat.id,
                            format!("LLM model: {}", app_state.nervo_llm.model_name()),
                        )
                        .await?;
                    }
                    JarvisCommands::Manual => {
                        system_message(app_state, &bot, &msg, SystemMessage::Manual(agent_type))
                            .await?;
                    }
                    _ => {}
                }
            } else {
                bot.send_message(msg.chat.id, "This command is only available for bots.")
                    .await?;
            }
        }
    }

    Ok(())
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
                    transcribe_message(app_state, &bot, regular_message, Stt).await?;
                } else if data == Tts.as_str() {
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
    nervo_agent_type: NervoAgentType,
) -> anyhow::Result<()> {
    info!("Start chat...");
    let agent_type = nervo_agent_type.agent_type;

    // Need to parse type of TG message. Text or Audio
    let mut parser = MessageParser {
        bot: &bot,
        msg: &msg,
        app_state: &app_state,
        is_voice: false,
    };
    info!("Parser created.");
    // Get info about bot and user
    let bot_info = &bot.get_me().await?;
    let bot_name: String = match bot_info.clone().user.username {
        None => {
            bail!("CRITICAL! No bot name");
        }
        Some(name) => name,
    };
    let user = parser.parse_user().await?;
    let UserId(user_id) = user.id;

    info!("Start conversation with bot: {}", bot_name);
    match start_conversation(
        app_state.clone(),
        &bot,
        user_id,
        &msg,
        bot_name,
        agent_type,
        parser,
    )
    .await
    {
        Ok(_) => {
            info!("Conversation has been finish successfully")
        }
        Err(err) => {
            info!("Can't finish conversation because of {}", err)
        }
    };

    Ok(())
}

pub static WHITELIST_MEMBERS: [&str; 0] = [];
