use anyhow::bail;
use anyhow::Result;
use async_openai::config::OpenAIConfig;
use async_openai::types::CreateChatCompletionRequestArgs;
use async_openai::types::CreateEmbeddingRequestArgs;
use async_openai::types::CreateEmbeddingResponse;
use async_openai::types::CreateModerationRequest;
use async_openai::types::CreateTranscriptionRequest;
use async_openai::types::ModerationInput;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessageContent,
};
use async_openai::types::{ChatCompletionRequestUserMessage, Embedding};
use async_openai::Client;
use serde_derive::Deserialize;
use tracing::info;

use nervo_sdk::api::spec::{LlmChat, LlmMessage, LlmMessageContent, LlmMessageRole};

#[derive(Clone, Debug, Deserialize)]
pub struct NervoLlmConfig {
    pub api_key: String,
    pub model_name: String,
    pub embedding_model_name: String,
    pub max_tokens: u16,
    pub temperature: f32,
}

impl NervoLlmConfig {
    pub fn open_ai_config(&self) -> OpenAIConfig {
        let cfg = OpenAIConfig::new();
        cfg.with_api_key(self.api_key.clone())
    }
}

#[derive(Clone, Debug)]
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
    pub async fn embedding(&self, text: &str) -> Result<CreateEmbeddingResponse> {
        let response = {
            let embedding = CreateEmbeddingRequestArgs::default()
                .model(self.llm_config.embedding_model_name.clone())
                .input(text)
                .build()?;

            self.client.embeddings().create(embedding).await?
        };

        Ok(response)
    }

    pub async fn text_to_embeddings(&self, text: &str) -> Result<Option<Embedding>> {
        let embedding = self.embedding(text).await?;
        Ok(embedding.data.first().cloned())
    }
}

impl NervoLlm {
    pub async fn send_msg_batch(&self, chat: LlmChat) -> Result<String> {
        let mut messages = vec![];
        for msg in chat.messages {
            let gpt_msg = ChatCompletionRequestMessage::transform_to(msg)?;
            messages.push(gpt_msg);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(self.llm_config.max_tokens)
            .model(self.llm_config.model_name.clone())
            .temperature(self.llm_config.temperature)
            .messages(messages)
            .build()?;

        let chat_response = self.client.chat().create(request).await?;

        let maybe_reply = chat_response
            .choices
            .first()
            .and_then(|chat_choice| chat_choice.message.content.clone())
            .map(|content| LlmMessageContent(content));

        let Some(reply) = maybe_reply else {
            bail!("No reply from LLM")
        };

        Ok(reply.0)
    }

    pub async fn send_msg(&self, message: LlmMessage, chat_id: u64) -> Result<String> {
        let chat = LlmChat {
            chat_id: Some(chat_id),
            messages: vec![message],
        };
        let llm_response_text = self.send_msg_batch(chat).await?;
        Ok(llm_response_text)
    }
    pub async fn moderate(&self, text: &str) -> Result<bool> {
        let request = CreateModerationRequest {
            input: ModerationInput::from(text),
            model: None,
        };

        let response = self.client.moderations().create(request).await?;
        info!(
            "Moderation is passed: {:?}",
            !response.results.iter().any(|property| property.flagged) && (text.len() < 10000)
        );
        Ok(!response.results.iter().any(|property| property.flagged) && (text.len() < 10000))
    }

    pub async fn voice_transcription(&self, request: CreateTranscriptionRequest) -> Result<String> {
        let response = self.client.audio().transcribe(request).await?;
        Ok(response.text)
    }
}

pub trait TransformTo<T>: Sized {
    type Error;

    /// Performs the conversion.
    fn transform_to(value: T) -> std::result::Result<Self, Self::Error>;
}

impl TransformTo<LlmMessage> for ChatCompletionRequestMessage {
    type Error = anyhow::Error;

    fn transform_to(msg: LlmMessage) -> std::result::Result<Self, Self::Error> {
        match msg.meta_info.role {
            LlmMessageRole::System => {
                let message = ChatCompletionRequestSystemMessage {
                    content: msg.content.text(),
                    name: None,
                };
                Ok(ChatCompletionRequestMessage::from(message))
            }
            LlmMessageRole::User => {
                let message = ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.text()),
                    name: msg.meta_info.sender_id.map(|id| id.to_string()),
                };
                Ok(ChatCompletionRequestMessage::from(message))
            }
            LlmMessageRole::Assistant => {
                let message = ChatCompletionRequestAssistantMessage {
                    content: Some(msg.content.text()),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                };

                Ok(ChatCompletionRequestMessage::from(message))
            }
        }
    }
}
