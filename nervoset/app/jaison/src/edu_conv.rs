use crate::utils::send_edu_menu;
use crate::UserState;
use std::fs;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::{Message, Requester};
use teloxide::types::ParseMode::MarkdownV2;
use teloxide::Bot;
use teloxide::types::ParseMode;
use tracing::info;

// pub async fn edu_conv_main(
//     bot: Bot,
//     msg: Message,
//     state: &mut UserState,
//     language_code: &str,
// ) -> anyhow::Result<()> {
//     let username = msg
//         .from()
//         .as_ref()
//         .and_then(|user| user.username.as_ref())
//         .map(|s| s.to_string())
//         .unwrap_or_default();
// 
//     let chapter_text_path = format!(
//         "resources/education_resources/{}.txt",
//         state.current_chapter
//     );
// 
//     let chapter_link_path = format!(
//         "resources/education_resources/links/{}.txt",
//         state.current_chapter
//     );
// 
//     let education_text = fs::read_to_string(&chapter_text_path)
//         .map_err(|e| format!("Failed to read '{}.txt' file: {}", chapter_text_path, e))
//         .unwrap();
// 
//     let education_text_link_result = fs::read_to_string(&chapter_link_path);
// 
//     // Function to split the text into parts not exceeding max_length
//     fn split_text(text: &str, max_length: usize) -> Vec<String> {
//         let mut parts = Vec::new();
//         let mut start = 0;
//         let mut end = 0;
//         // let mut char_count = 0;
// 
//         while start < text.len() {
//             // Find the next break point
//             let mut last_end = start;
//             let mut char_count = 0;
//             for (i, c) in text[start..].char_indices() {
//                 char_count += c.len_utf8();
//                 if char_count > max_length {
//                     break;
//                 }
//                 if ".!?".contains(c) {
//                     last_end = start + i + c.len_utf8();
//                 }
//             }
// 
//             // If we found a sentence end, use it; otherwise, cut at max_length
//             if last_end > start {
//                 end = last_end;
//             } else {
//                 end = (start + max_length).min(text.len());
//             }
// 
//             // Add the chunk to the result and move the start pointer
//             parts.push(text[start..end].to_string());
//             start = end;
// 
//             // Skip any whitespace at the start of the next part
//             while start < text.len() && text[start..].starts_with(char::is_whitespace) {
//                 start += 1;
//             }
//         }
// 
//         parts
//     }
// 
//     let max_message_length = 4096;
//     let text_parts = split_text(&education_text, max_message_length);
// 
//     for (i, part) in text_parts.iter().enumerate() {
//         println!("Sending part {}: {} (length: {})", i + 1, part, part.len());
//         if !part.trim().is_empty() {
//             bot.send_message(msg.chat.id, part)
//                 .protect_content(true)
//                 .await?;
//         }
//     }
// 
//     match education_text_link_result {
//         Ok(education_text_link) => {
//             bot.send_message(
//                 msg.chat.id,
//                 format!("[Ссылка на видео-урок]({})", education_text_link),
//             )
//             .parse_mode(MarkdownV2)
//             .protect_content(true)
//             .await?;
//         }
//         Err(e) => {
//             eprintln!(
//                 "File '{}.txt' not found for the reason: {}",
//                 chapter_link_path, e
//             );
//         }
//     }
//     
//     send_edu_menu(&bot, msg.chat.id, language_code.to_string()).await?;
// 
//     info!(
//         "Пользователь @{} изучил главу №{}",
//         username, state.current_chapter
//     );
// 
//     Ok(())
// }


pub async fn edu_conv(
    bot: Bot,
    msg: Message,
    state: &mut UserState,
    language_code: &str,
) -> anyhow::Result<()> {
    let username = msg
        .from()
        .as_ref()
        .and_then(|user| user.username.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_default();

    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;
    let user_folder_name = user_id.to_string();

    let chapter_text_path = format!(
        "resources/education_resources/{}/{}.txt",
        user_folder_name,
        state.current_chapter
    );

    let education_text = match fs::read_to_string(&chapter_text_path) {
        Ok(text) => text,
        Err(_) => {
            let end_message = "Обучающие материалы закончились.\nВозвращаю тебя к Главе №1:\n";
            bot.send_message(msg.chat.id, end_message)
                .await?;

            state.current_chapter = 1;

            let new_chapter_text_path = format!(
                "resources/education_resources/{}/{}.txt",
                user_folder_name,
                state.current_chapter
            );

            let new_education_text = fs::read_to_string(&new_chapter_text_path)
                .map_err(|e| format!("Failed to read '{}.txt' file: {}", chapter_text_path, e))
                .unwrap();
            bot.send_message(msg.chat.id, new_education_text)
                .await?;

            return Ok(());
        }
    };

    // Function to split the text into parts not exceeding max_length
    fn split_text(text: &str, max_length: usize) -> Vec<String> {
        let mut parts = Vec::new();
        let mut start = 0;
        let mut end = 0;
        // let mut char_count = 0;

        while start < text.len() {
            // Find the next break point
            let mut last_end = start;
            let mut char_count = 0;
            for (i, c) in text[start..].char_indices() {
                char_count += c.len_utf8();
                if char_count > max_length {
                    break;
                }
                if ".!?".contains(c) {
                    last_end = start + i + c.len_utf8();
                }
            }

            // If we found a sentence end, use it; otherwise, cut at max_length
            if last_end > start {
                end = last_end;
            } else {
                end = (start + max_length).min(text.len());
            }

            // Add the chunk to the result and move the start pointer
            parts.push(text[start..end].to_string());
            start = end;

            // Skip any whitespace at the start of the next part
            while start < text.len() && text[start..].starts_with(char::is_whitespace) {
                start += 1;
            }
        }

        parts
    }

    let max_message_length = 4096;
    let text_parts = split_text(&education_text, max_message_length);

    for (i, part) in text_parts.iter().enumerate() {
        println!("Sending part {}: {} (length: {})", i + 1, part, part.len());
        if !part.trim().is_empty() {
            bot.send_message(msg.chat.id, part)
                .protect_content(true)
                .await?;
        }
    }

    send_edu_menu(&bot, msg.chat.id, language_code.to_string()).await?;

    info!(
        "Пользователь @{} изучил главу №{}",
        username, state.current_chapter
    );

    Ok(())
}