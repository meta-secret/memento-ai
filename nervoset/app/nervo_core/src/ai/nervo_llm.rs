use crate::db::local_db::LocalDb;
use crate::models::nervo_message_model::TelegramMessage;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    CreateEmbeddingRequestArgs, CreateEmbeddingResponse, CreateModerationRequest,
    CreateTranscriptionRequest, ModerationInput, Role,
};
use async_openai::Client;
use serde_derive::Deserialize;
use tracing::info;

#[derive(Clone, Debug, Deserialize)]
pub struct NervoLlmConfig {
    pub api_key: String,
    pub model_name: String,
    pub embedding_model_name: String,
    pub max_tokens: u16,
    pub temperature: f32
}

impl NervoLlmConfig {
    pub fn open_ai_config(&self) -> OpenAIConfig {
        let cfg = OpenAIConfig::new();
        cfg.with_api_key(self.api_key.clone())
    }
}

pub struct NervoLlm {
    llm_config: NervoLlmConfig,
    client: Client<OpenAIConfig>,
}

impl From<NervoLlmConfig> for NervoLlm {
    fn from(llm_config: NervoLlmConfig) -> Self {
        NervoLlm {
            llm_config: llm_config.clone(),
            client: Client::with_config(llm_config.open_ai_config()),
        }
    }
}

impl NervoLlm {
    pub fn model_name(&self) -> &str {
        self.llm_config.model_name.as_str()
    }
    pub fn api_key(&self) -> &str {
        self.llm_config.api_key.as_str()
    }
}

impl NervoLlm {
    pub async fn embedding(&self, text: &str) -> anyhow::Result<CreateEmbeddingResponse> {
        let embedding = CreateEmbeddingRequestArgs::default()
            .model(self.llm_config.embedding_model_name.clone())
            .input(text)
            .build()?;
        let response = self.client.embeddings().create(embedding).await?;
        Ok(response)
    }
}

impl NervoLlm {
    pub async fn chat_batch(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> anyhow::Result<Option<String>> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(self.llm_config.max_tokens)
            .model(self.llm_config.model_name.clone())
            .messages(messages)
            .temperature(self.llm_config.temperature)
            .build()?;

        let chat_response = self.client.chat().create(request).await?;
        let maybe_reply = chat_response.choices.first();
        let maybe_msg = maybe_reply.and_then(|reply| reply.message.content.clone());

        Ok(maybe_msg)
    }

    pub async fn chat(
        &self,
        username: &str,
        message: ChatCompletionRequestUserMessage,
        local_db: &LocalDb,
    ) -> anyhow::Result<Option<String>> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        let cached_messages: Vec<TelegramMessage> = local_db.read_messages(&username).await?;

        for msg in cached_messages {
            let cached_msg = ChatCompletionRequestUserMessageArgs::default()
                .content(msg.message)
                .build()?;
            messages.push(ChatCompletionRequestMessage::from(cached_msg))
        }

        let system_msg = ChatCompletionRequestUserMessageArgs::default()
            .content("Ты эксперт в области живой гигиены, тебя зовут Пробиоша, но ты мужского пола. Ты отвечаешь на вопросы развёрнуто и подробно с точки зрения концепции микробиома, микробной гигиены и микробной очистки.")
            .role(Role::System)
            .build()?;
        messages.push(ChatCompletionRequestMessage::from(system_msg));

        messages.push(ChatCompletionRequestMessage::from(message));
        self.chat_batch(messages).await
    }

    pub async fn moderate(&self, text: &str) -> anyhow::Result<bool> {
        let request = CreateModerationRequest {
            input: ModerationInput::from(text),
            model: None,
        };

        let response = self.client.moderations().create(request).await?;
        Ok(!response.results.iter().any(|property| property.flagged) && (text.len() < 10000))
    }

    pub async fn voice_transcription(
        &self,
        request: CreateTranscriptionRequest,
    ) -> anyhow::Result<String> {
        let response = self.client.audio().transcribe(request).await;
        match response {
            Ok(text) => Ok(text.text),
            Err(err) => {
                info!("RESPONSE: ERR {:?}", err);
                return Err(err.into());
            }
        }
    }
}
