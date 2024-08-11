use crate::ask_question::ask_question;
use crate::creation::creation_mode_processing;
use crate::edu_conv::edu_conv;
use crate::utils::{llm_processing_creation_utility, qdrant_upsert, save_data_for_local_use, send_creation_menu, send_db_operation_menu, send_db_topping_up_menu, send_edu_menu, send_main_menu, send_question_menu, send_system_role_creation_menu, text_to_speech};
use crate::AppState;
use std::fs;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::{InputFile, ParseMode, ReplyMarkup};
use tracing::info;

pub async fn user_interaction(
    bot: Bot,
    msg: Message,
    app_state: Arc<AppState>,
) -> anyhow::Result<()> {
    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;
    let mut user_states = app_state.user_states.lock().await;

    if let Some(state) = user_states.get_mut(&user_id) {
        let language_code = state
            .language_code
            .clone()
            .unwrap_or_else(|| "ru".to_string());

        if state.awaiting_start_option_choice {
            match msg.text().unwrap_or_default() {
                "К обучению!" | "To Education!" => {
                    state.awaiting_start_option_choice = false;
                    state.education = true;

                    return edu_conv(bot, msg, state, &language_code).await;
                }
                "Задать вопрос" | "Ask a Question" => {
                    state.awaiting_start_option_choice = false;
                    state.awaiting_question = true;

                    bot.send_message(msg.chat.id, "Пожалуйста, введите ваш вопрос:")
                        .await?;

                    send_question_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Создать Агента" | "Create an Agent" => {
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
                "Выход" | "Exit" => {
                    state.awaiting_start_option_choice = false;

                    bot.send_message(
                        msg.chat.id,
                        "До скорых встреч!\n\nНажми /start чтобы \
                    запустить программу обучения.",
                    )
                    .reply_markup(ReplyMarkup::kb_remove())
                    .await?;
                }
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        "Пожалуйста, выбери один из представленных \
                    вариантов или заверши программу кнопкой \'Выход\'.",
                    )
                    .await?;
                }
            }
        } else if state.education {
            match msg.text().unwrap_or_default() {
                "Предыдущая глава" | "Previous chapter" => {
                    if state.current_chapter > 1 {
                        state.current_chapter -= 1;
                    }
                    return edu_conv(bot, msg, state, &language_code).await;
                }
                "Следующая глава" | "Next chapter" => {
                    state.current_chapter += 1;
                    return edu_conv(bot, msg, state, &language_code).await;
                }
                "Задать вопрос" | "Ask a Question" => {
                    state.education = false;
                    state.awaiting_question = true;

                    bot.send_message(msg.chat.id, "Пожалуйста, введите ваш вопрос:")
                        .await?;

                    send_question_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Выход" | "Exit" => {
                    state.education = false;
                    state.awaiting_start_option_choice = true;

                    send_main_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        "Пожалуйста, выбери один из представленных \
                    вариантов или заверши взаимодействие кнопкой \'Выход\'.",
                    )
                    .await?;
                }
            }
        } else if state.awaiting_question {
            match msg.text().unwrap_or_default() {
                "Вернуться к обучению" | "Back to Education" => {
                    state.awaiting_question = false;
                    state.education = true;

                    send_edu_menu(&bot, msg.chat.id, language_code.clone()).await?;

                    return edu_conv(bot, msg, state, &language_code).await;
                }
                "Выход" | "Exit" => {
                    state.awaiting_question = false;
                    state.awaiting_start_option_choice = true;

                    send_main_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Голосовой ответ" | "Voice Response" => {
                    if let Some(bot_message_text) = &state.last_bot_message {
                        let audio_file_path =
                            text_to_speech(&bot, &msg, bot_message_text.clone()).await?;
                        bot.send_voice(msg.chat.id, InputFile::file(audio_file_path))
                            .await?;
                    } else {
                        bot.send_message(
                            msg.chat.id,
                            "Мне пока нечего сказать, задай вопрос, \
                        я с радостью отвечу на него голосом.",
                        )
                        .await?;
                    }
                }
                _ => {
                    return ask_question(bot, msg, state).await;
                }
            }
        } else if state.creation {
            match msg.text().unwrap_or_default() {
                "1. Добавить описание сферы знаний Агента"
                | "1. Add sphere of Agent's knowledge description" => {
                    state.creation = false;
                    state.awaiting_sphere_description = true;
                    bot.send_message(msg.chat.id, "Опиши сферу знаний твоего Ai-агента. Расскажи, чем он может быть полезен, какую информацию пользователи могут у него запросить:").await?;
                }
                "2. Добавить описание модели поведения Агента"
                | "2. Add Agent's role description" => {
                    state.creation = false;
                    state.awaiting_behavior_model_description = true;
                    bot.send_message(msg.chat.id, "Опиши поведенческую роль твоего Ai-агента. Как именно он должен взаимодействовать с пользователями - предоставлять информацию, продвигать какую-то идею и т.д.:").await?;
                }
                "3. Сформировать системную роль Агента" | "3. Create system role for Agent" =>
                {
                    state.creation = false;
                    state.system_role_creation = true;
                    send_system_role_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
                "4. Редактировать базу данных Агента" | "4. Edit Agent's database" =>
                {
                    state.creation = false;
                    state.db_operation = true;
                    send_db_operation_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Выход" | "Exit" => {
                    state.creation = false;
                    state.awaiting_start_option_choice = true;
                    send_main_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        "Пожалуйста выбери один из представленных \
                    вариантов или заверши программу кнопкой \'Выход\'.",
                    )
                    .await?;
                }
            }
        } else if state.awaiting_sphere_description {
            match msg.text().unwrap_or_default() {
                "Выход" | "Exit" => {
                    state.awaiting_sphere_description = false;
                    state.creation = true;

                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    creation_mode_processing(bot.clone(), msg.clone(), state).await?;
                    info!("Пользователь сохранил описание сферы знаний Агента.");
                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
            }
        } else if state.awaiting_behavior_model_description {
            match msg.text().unwrap_or_default() {
                "Выход" | "Exit" => {
                    state.awaiting_behavior_model_description = false;
                    state.creation = true;

                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    creation_mode_processing(bot.clone(), msg.clone(), state).await?;
                    info!("Пользователь сохранил описание модели поведения Агента.");
                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
            }
        } else if state.system_role_creation {
            match msg.text().unwrap_or_default() {
                "Да" | "Yes" => {
                    creation_mode_processing(bot.clone(), msg.clone(), state).await?;

                    info!("Пользователь сохранил главную системную роль Агента.");

                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Нет, вернуться назад" | "No, go back" => {
                    state.system_role_creation = false;
                    state.creation = true;

                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Выход" | "Exit" => {
                    state.system_role_creation = false;
                    state.awaiting_start_option_choice = true;

                    send_main_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        "Пожалуйста выбери один из представленных \
                    вариантов или заверши программу кнопкой \'Выход\'.",
                    )
                    .await?;
                }
            }
        } else if state.db_operation {
            match msg.text().unwrap_or_default() {
                "Пополнить" | "Top-up db" => {
                    state.db_operation = false;
                    state.db_topping_up = true;

                    send_db_topping_up_menu(&bot, msg.chat.id, language_code).await?;
                }
                "Редактировать" | "Edit db" => {
                    bot.send_message(
                        msg.chat.id,
                        "Функция редактирования базы данных появится чуть позже.\nА пока \
                        выбери один из представленных вариантов или заверши программу \
                        кнопкой \'Выход\'.",
                    )
                        .await?;
                }
                "Выход" | "Exit" => {
                    state.db_operation = false;
                    state.creation = true;

                    send_creation_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    bot.send_message(
                        msg.chat.id,
                        "Пожалуйста выбери один из представленных \
                    вариантов или заверши программу кнопкой \'Выход\'.",
                    )
                    .await?;
                }
            }
        } else if state.db_topping_up {
            match msg.text().unwrap_or_default() {
                "Выход" | "Exit" => {
                    state.db_topping_up = false;
                    state.db_operation = true;

                    send_db_operation_menu(&bot, msg.chat.id, language_code).await?;
                }
                _ => {
                    qdrant_upsert(msg.clone()).await?;

                    bot.send_message(msg.chat.id, "done!").await?;
                    
                    // Temporary code:
                    let data_to_store = msg.text().unwrap().to_string();
                    save_data_for_local_use(data_to_store.clone(), msg.clone()).await?;
                    info!("data saved!");
                    
                    state.db_topping_up = false;
                    state.db_operation = true;

                    send_db_operation_menu(&bot, msg.chat.id, language_code).await?;
                }
            }
        }
    }

    Ok(())
}
