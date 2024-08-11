use crate::utils::llm_processing_with_rag;
use crate::UserState;
use anyhow::Result;
use std::fs;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ChatAction;
use teloxide::Bot;

pub async fn ask_question(bot: Bot, msg: Message, state: &mut UserState) -> Result<()> {
    let user_question = msg.text().unwrap_or_default().to_string();
    let system_role = fs::read_to_string("resources/creation_resources/output/5964236329/main_system_role_output.txt")
        .map_err(|e| format!("Failed to read 'assistant_system_role': {}", e))
        .unwrap();
    bot.send_chat_action(msg.chat.id, ChatAction::Typing)
        .await?;
    let response = llm_processing_with_rag(msg.clone(), system_role, user_question).await?;

    bot.send_message(msg.chat.id, &response)
        .protect_content(true)
        .await?;

    state.last_bot_message = Some(response);

    Ok(())
}
