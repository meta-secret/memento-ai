use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use teloxide::prelude::{Message, Requester};
use teloxide::Bot;
use tracing::info;

use crate::utils::llm_processing_creation_utility;
use crate::UserState;


pub async fn creation_mode_processing(bot: Bot, msg: Message, state: &mut UserState) -> Result<()> {
    let user_task = msg.text().unwrap_or_default();
    let user_id = msg.from().map(|user| user.id.0).unwrap_or(0) as i64;
    let user_org_name = user_id.to_string();

    let user_input_folder_path =
        format!("resources/creation_resources/user_input/{}", user_org_name);
    let user_output_folder_path = format!("resources/creation_resources/output/{}", user_org_name);

    if !Path::new(&user_input_folder_path).exists() {
        fs::create_dir_all(&user_input_folder_path)?;
    }
    if !Path::new(&user_output_folder_path).exists() {
        fs::create_dir_all(&user_output_folder_path)?;
    }

    let mut current_step = 0;

    if state.system_role_creation {
        let sphere_description = fs::read_to_string(format!(
            "{}/user_sphere_output.txt",
            user_output_folder_path
        ))
        .with_context(|| "Failed to read 'user_sphere_output.txt'".to_string())?;
        let behavior_model = fs::read_to_string(format!(
            "{}/user_behavior_model_output.txt",
            user_output_folder_path
        ))
        .with_context(|| "Failed to read 'user_system_role_output.txt'".to_string())?;

        let combined_text = format!(
            "Описание сферы познаний Агента: {}\nПоведенческая модель агента: {}",
            sphere_description, behavior_model
        );
        info!("combined_text: {}", combined_text);
        let main_system_role_creation_task = fs::read_to_string(
            "resources/creation_resources/main_system_role_creation_task.txt",
        )
        .with_context(|| "Failed to read 'main_system_role_creation_task.txt'".to_string())?;

        let user_task_processing =
            llm_processing_creation_utility(main_system_role_creation_task, combined_text).await?;

        let output_path = format!("{}/main_system_role_output.txt", user_output_folder_path);
        let mut output_file = File::create(&output_path)
            .with_context(|| format!("Failed to create file '{}'", output_path))?;
        writeln!(output_file, "{}", user_task_processing)
            .with_context(|| format!("Failed to write to file '{}'", output_path))?;

        current_step = 4;

        bot.send_message(
            msg.chat.id,
            format!("Отличная работа!\nГлавная системная роль сформирована!\n\nДавай передём к финальному шагу {}",
                    current_step
            ),
        )
            .await?;

        state.awaiting_sphere_description = false;
        state.awaiting_behavior_model_description = false;
        state.system_role_creation = false;
        state.creation = true;
    } else {
        let (user_input_path, creation_system_role_path, output_path) =
            if state.awaiting_sphere_description {
                current_step = 2;
                (
                    format!("{}/user_input_for_sphere.txt", user_input_folder_path),
                    "resources/creation_resources/sphere_creation_system_role.txt".to_string(),
                    format!("{}/user_sphere_output.txt", user_output_folder_path),
                )
            } else if state.awaiting_behavior_model_description {
                current_step = 3;
                (
                    format!(
                        "{}/user_input_for_behavior_model.txt",
                        user_input_folder_path
                    ),
                    "resources/creation_resources/role_creation_system_role.txt".to_string(),
                    format!("{}/user_behavior_model_output.txt", user_output_folder_path),
                )
            } else {
                // If the state does not match the expected values, we return an error
                return Err(anyhow::anyhow!("Unexpected user state"));
            };

        let mut user_input_file = File::create(&user_input_path)
            .with_context(|| format!("Failed to create file '{}'", user_input_path))?;
        writeln!(user_input_file, "{}", user_task)
            .with_context(|| format!("Failed to write to file '{}'", user_input_path))?;

        let creation_system_role = fs::read_to_string(&creation_system_role_path)
            .with_context(|| format!("Failed to read file '{}'", creation_system_role_path))?;

        let user_task_processing =
            llm_processing_creation_utility(creation_system_role, user_task.to_string()).await?;

        let mut output_file = File::create(&output_path)
            .with_context(|| format!("Failed to create file '{}'", output_path))?;
        writeln!(output_file, "{}", user_task_processing)
            .with_context(|| format!("Failed to write to file '{}'", output_path))?;

        bot.send_message(
            msg.chat.id,
            format!(
                "Отличная работа!\nДанные записаны.\n\nДавай перейдём к шагу {}.",
                current_step
            ),
        )
        .await?;

        state.awaiting_sphere_description = false;
        state.awaiting_behavior_model_description = false;
        state.system_role_creation = false;
        state.creation = true;
    }

    Ok(())
}
