use std::string::ToString;
use std::sync::Arc;

use anyhow::Result;
use teloxide::macros::BotCommands;
use teloxide::prelude::*;
use teloxide::Bot as TelegramBot;
use teloxide::types::{MessageKind};

use crate::common::AppState;
use crate::telegram::roles_and_permissions::{
    has_role,
    PROBIOT_MEMBER,
    PROBIOT_OWNER};
use crate::telegram::bot_utils::{chat, system_message, SystemMessage};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum ProbiotCommands {
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
enum ProbiotOwnerCommands {
    #[command(description = "Get whitelisted users list")]
    GetWhiteListMembers,
}


pub async fn permission_restricted(bot: Bot, msg: Message) -> Result<()> {
    bot.send_message(msg.chat.id, "сорян, я пока не работаю, приходите через 2 мес.").await?;
    Ok(())
}

static WHITELIST_MEMBERS: [&str; 0] = [
    // "afazulzyanov",
];

/// Start telegram bot
pub async fn start(token: String, app_state: Arc<AppState>) -> Result<()> {
    let bot = TelegramBot::new(token);

    app_state.local_db.init_db().await?;

    let handler = {
        let owner_handler = Update::filter_message()
            .filter_command::<ProbiotOwnerCommands>()
            .endpoint(owner_command_handler);

        let cmd_handler = Update::filter_message()
            .filter_command::<ProbiotCommands>()
            .endpoint(command_handler);

        let msg_handler = Update::filter_message().endpoint(chat);

        let authorized_user_handler = Update::filter_message()
            .branch(cmd_handler)
            .branch(msg_handler);

        let permission_restricted_handler = Update::filter_message().endpoint(
            permission_restricted
        );

        Update::filter_message()
            .branch(
                dptree::filter_async(
                    |msg: Message, app_state: Arc<AppState>| async move {
                        has_role(
                            app_state,
                            msg.from().clone(),
                            &PROBIOT_OWNER.to_string()).await
                    }
                ).chain(owner_handler))
            .branch(
                dptree::filter_async(
                    |msg: Message, app_state: Arc<AppState>| async move {
                        has_role(
                            app_state,
                            msg.from().clone(),
                            &PROBIOT_MEMBER.to_string()).await
                    }
                ).chain(authorized_user_handler.clone())
            )
            .branch(
                dptree::filter(|msg: Message, app_state: Arc<AppState>| {
                        msg.from().map(|user|
                            WHITELIST_MEMBERS
                                .contains(&user.username.clone().unwrap_or_default().as_str()))
                                .unwrap_or(false)
                    })
                .chain(authorized_user_handler))
            .branch(permission_restricted_handler)

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

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: ProbiotCommands,
    app_state: Arc<AppState>,
) -> Result<()> {
    match cmd {
        ProbiotCommands::Model => {
            bot.send_message(
                msg.chat.id,
                format!("LLM model: {}", app_state.nervo_llm.model_name()),
            )
            .await?;
            Ok(())
        }
        ProbiotCommands::Start => {
            system_message(&bot, &msg, SystemMessage::Start).await?;
            Ok(())
        }
        ProbiotCommands::Manual => {
            system_message(&bot, &msg, SystemMessage::Manual).await?;
            Ok(())
        }
    }
}

async fn owner_command_handler(
    bot: Bot,
    msg: Message,
    cmd: ProbiotOwnerCommands,
) -> Result<()> {
    match cmd {
        ProbiotOwnerCommands::GetWhiteListMembers => {
            let formatted_usernames = WHITELIST_MEMBERS.iter()
                .map(|username| format!("@{}", username).to_string())
                .collect::<Vec<String>>();
            bot.send_message(
                msg.chat.id,
                formatted_usernames.join("\n"),
            ).await?;
            Ok(())
        }
    }
}
