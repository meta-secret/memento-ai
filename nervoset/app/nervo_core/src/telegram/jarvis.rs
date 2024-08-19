use std::string::ToString;
use std::sync::Arc;

use crate::config::common::TelegramBotParams;
use crate::config::jarvis::JarvisAppState;
use crate::telegram::bot_utils::{chat, system_message, SystemMessage};
use crate::telegram::roles_and_permissions::{has_role, MEMBER, OWNER};
use anyhow::Result;
use nervo_api::agent_type::AgentType;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::Bot as TelegramBot;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum JarvisCommands {
    #[command(description = "Ai model name.")]
    Model,
    Start,
    Manual,
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum JarvisOwnerCommands {
    #[command(description = "Get whitelisted users list")]
    GetWhiteListMembers,
}

// pub async fn permission_restricted(bot: Bot, msg: Message) -> Result<()> {
//     bot.send_message(
//         msg.chat.id,
//         "сорян, я пока не работаю, приходите через 2 мес.",
//     )
//     .await?;
//     Ok(())
// }

static WHITELIST_MEMBERS: [&str; 0] = [
    // "afazulzyanov",
];

/// Start telegram bot
pub async fn start(
    params: TelegramBotParams,
    app_state: Arc<JarvisAppState>,
    agent_type: AgentType,
) -> Result<()> {
    let bot = TelegramBot::new(params.token.as_str());

    app_state.local_db.init_db().await?;

    let handler = {
        let owner_handler = Update::filter_message()
            .filter_command::<JarvisOwnerCommands>()
            .endpoint(owner_command_handler);

        let cmd_handler = Update::filter_message()
            .filter_command::<JarvisCommands>()
            .endpoint(command_handler);

        let msg_handler = Update::filter_message().endpoint(chat);

        Update::filter_message()
            .branch(owner_handler)
            .branch(cmd_handler)
            .branch(msg_handler)

    };

    Dispatcher::builder(bot, handler)
        // Pass the shared state to the handler as a dependency.
        .dependencies(dptree::deps![app_state, agent_type])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: JarvisCommands,
    app_state: Arc<JarvisAppState>,
    agent_type: AgentType,
) -> Result<()> {
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

async fn owner_command_handler(bot: Bot, msg: Message, cmd: JarvisOwnerCommands) -> Result<()> {
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
