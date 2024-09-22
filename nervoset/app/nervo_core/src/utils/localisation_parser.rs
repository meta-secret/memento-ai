use std::fmt::Display;
use crate::ai::nervo_llm::{NervoLlm};
use crate::models::qdrant_search_layers::QdrantSearchLayer;
use crate::utils::ai_utils::formation_system_role_llm_message;
use anyhow::Result;
use nervo_sdk::api::spec::LlmChat;
use tracing::info;

pub struct LocalisationManager {
    pub user_language: UserLang,
    pub nervo_llm: NervoLlm,
}

impl LocalisationManager {
    pub fn build(nervo_llm: NervoLlm) -> Result<Self> {
        Ok(LocalisationManager {
            user_language: UserLang::None,
            nervo_llm,
        })
    }
}

impl LocalisationManager {
    pub async fn detect_language(&mut self, text: &str) -> Result<()> {
        info!("Lang need to be detected! {}", text);
        let system_role_instructions = format!("You are provided with a text - {}. Determine the language in which this text is written and as a response, return only the language of the provided text, without additional remarks or comments, example: Russian, English.", text);
        let language_detecting_layer = QdrantSearchLayer {
            index: None,
            user_role_params: vec![],
            system_role_text: system_role_instructions.to_string(),
            temperature: 0.2,
            max_tokens: 4096,
            common_token_limit: 30000,
            vectors_limit: 0,
            layer_for_search: false,
        };

        let system_role_msg = formation_system_role_llm_message(language_detecting_layer).await?;

        info!("Full detecting role message: {}", system_role_msg.content.0);
        let chat: LlmChat = LlmChat {
            chat_id: None,
            messages: vec![system_role_msg],
        };
        let llm_response = self.nervo_llm.send_msg_batch(chat).await?;
        info!("Lang has been detected! {}", llm_response);
        self.user_language = UserLang::from(llm_response.to_lowercase().as_ref());

        Ok(())
    }
    
    pub async fn set_language_as_locale(&mut self, locale: &str) -> Result<()> {
        match self.user_language {
            UserLang::None => {
                info!("Set lang according to user locale {}", locale);
                self.user_language = UserLang::from(locale.to_lowercase().as_ref());
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn translate(&self, text: &str) -> Result<String> {
        info!("Starting translation");
        let language = self.user_language.to_string();
        let system_role_instructions = format!("You are provided with: the user’s language - {}, as well as: the ready response for the user - {}. Your task: Translate the ready response for the user into the user’s language.", language, text);
        let translation_layer = QdrantSearchLayer {
            index: None,
            user_role_params: vec![],
            system_role_text: system_role_instructions.to_string(),
            temperature: 0.2,
            max_tokens: 4096,
            common_token_limit: 30000,
            vectors_limit: 0,
            layer_for_search: false,
        };

        let system_role_msg = formation_system_role_llm_message(translation_layer).await?;

        info!(
            "Full translator role message: {}",
            system_role_msg.content.0
        );
        let chat: LlmChat = LlmChat {
            chat_id: None,
            messages: vec![system_role_msg],
        };
        let llm_response = self.nervo_llm.send_msg_batch(chat).await?;
        info!(
            "Translated on {} response is {}",
            language, llm_response
        );
        Ok(llm_response)
    }
}

pub enum UserLang {
    Ru,
    En,
    Other(String),
    None
}

impl From<&str> for UserLang {
    fn from(lang_string: &str) -> Self {
        match lang_string {
            "english" | "eng" | "en" => UserLang::En,
            "russian" | "rus" | "ru" => UserLang::Ru,
            _ => UserLang::Other(lang_string.to_string()),
        }
    }
}

impl Display for UserLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            UserLang::Ru => "russian".to_string(),
            UserLang::En => "english".to_string(),
            UserLang::Other(lang) => lang.clone(),
            UserLang::None => "english".to_string(),
        };
        write!(f, "{}", str)
    }
}