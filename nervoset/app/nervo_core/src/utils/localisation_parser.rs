use crate::ai::nervo_llm::NervoLlm;
use crate::models::qdrant_search_layers::QdrantSearchLayer;
use crate::utils::ai_utils::formation_system_role_llm_message;
use anyhow::Result;
use nervo_sdk::api::spec::LlmChat;
use tracing::info;

pub struct LocalisationManager {
    pub user_language: String,
    pub nervo_llm: NervoLlm,
}

impl LocalisationManager {
    pub fn build(nervo_llm: NervoLlm) -> Result<Self> {
        Ok(LocalisationManager {
            user_language: "English".to_string().to_lowercase(),
            nervo_llm,
        })
    }
}

impl LocalisationManager {
    pub async fn detect_language(&mut self, text: &str) -> Result<()> {
        info!("Lang need to be detected! {}", text);
        let system_role_instructions = format!("You are provided with a text. - {}. Determine the language in which this text is written and return its name in one word in English.", text);
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
        self.user_language = llm_response.to_lowercase();

        Ok(())
    }

    pub async fn translate(&self, text: &str) -> Result<String> {
        info!("Starting translation");
        let system_role_instructions = format!("You are provided with: the user’s language - {}, as well as: the ready response for the user - {}. Your task: Translate the ready response for the user into the user’s language.", self.user_language, text);
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
            self.user_language, llm_response
        );
        Ok(llm_response)
    }
}
