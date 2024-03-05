use std::sync::Arc;
use anyhow::bail;
use async_openai::types::{ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageArgs};
use teloxide::Bot;
use teloxide::prelude::ChatId;
use teloxide::types::{MediaKind, MessageKind, ReplyMarkup, User};
use crate::common::AppState;
use crate::telegram::tg_keyboard::NervoBotKeyboard;
use teloxide::prelude::*;

pub async fn chat(bot: Bot, msg: Message, app_state: Arc<AppState>) -> anyhow::Result<()> {
    let (_, text) = parse_user_and_text(&msg).await?;
    if text.is_empty() {
        bot.send_message(msg.chat.id, "Please provide a message to send.")
            //.reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
            .await?;

        return Ok(());
    }

    let is_moderation_passed = app_state.nervo_llm.moderate(&text).await?;
    if is_moderation_passed {
        let user_msg = ChatCompletionRequestUserMessageArgs::default()
            .content(text)
            .build()?;

        chat_gpt_conversation(bot, msg.chat.id, app_state, user_msg).await
    } else {
        bot.send_message(
            msg.chat.id,
            "Your message is not allowed. Please rephrase it.",
        )
            //.reply_markup(ReplyMarkup::Keyboard(NervoBotKeyboard::build_keyboard()))
            .await?;
        Ok(())
    }
}

pub async fn chat_gpt_conversation(
    bot: Bot,
    chat_id: ChatId,
    app_state: Arc<AppState>,
    msg: ChatCompletionRequestUserMessage,
) -> anyhow::Result<()> {
    let reply = app_state
        .nervo_llm
        .chat(msg)
        .await?
        .unwrap_or(String::from("I'm sorry, internal error."));

    bot.send_message(chat_id, reply).await?;

    Ok(())
}

pub async fn parse_user_and_text(msg: &Message) -> anyhow::Result<(&User, String)> {
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
