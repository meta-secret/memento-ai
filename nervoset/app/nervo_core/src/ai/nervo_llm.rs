use anyhow::bail;
use anyhow::Result;
use async_openai::config::OpenAIConfig;
use async_openai::types::ChatCompletionRequestUserMessage;
use async_openai::types::CreateChatCompletionRequestArgs;
use async_openai::types::CreateEmbeddingRequestArgs;
use async_openai::types::CreateEmbeddingResponse;
use async_openai::types::CreateModerationRequest;
use async_openai::types::CreateTranscriptionRequest;
use async_openai::types::ModerationInput;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessageContent, Role,
};
use async_openai::Client;
use serde_derive::{Deserialize, Serialize};
use tracing::error;

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
    pub async fn send_msg_batch(
        &self,
        chat: LlmChat,
        sender_id: u64,
    ) -> anyhow::Result<LlmMessage> {
        let mut messages = vec![];
        for msg in chat.messages {
            match msg.role {
                Role::System => {
                    let message = ChatCompletionRequestSystemMessage {
                        content: msg.content.clone(),
                        role: msg.role,
                        name: Some(msg.sender_id.to_string()),
                    };
                    messages.push(ChatCompletionRequestMessage::from(message));
                }
                Role::User => {
                    let message = ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                        role: msg.role,
                        name: Some(msg.sender_id.to_string()),
                    };
                    messages.push(ChatCompletionRequestMessage::from(message));
                }
                Role::Assistant => {
                    let message = ChatCompletionRequestAssistantMessage {
                        content: Some(msg.content.clone()),
                        role: msg.role,
                        name: Some(msg.sender_id.to_string()),
                        tool_calls: None,
                        function_call: None,
                    };

                    messages.push(ChatCompletionRequestMessage::from(message));
                }
                Role::Tool => {
                    bail!("Role::Tool is not supported")
                }
                Role::Function => {
                    bail!("Role::Function is not supported")
                }
            }
        }

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(self.llm_config.max_tokens)
            .model(self.llm_config.model_name.clone())
            .messages(messages)
            .temperature(self.llm_config.temperature)
            .build()?;

        let chat_response = self.client.chat().create(request).await?;
        let reply = chat_response.choices.first().unwrap();

        let response = LlmMessage {
            sender_id,
            role: Role::Assistant,
            content: reply
                .message
                .content
                .clone()
                .unwrap_or(String::from("error")),
        };

        Ok(response)
    }

    pub async fn send_msg(
        &self,
        message: LlmMessage,
        chat_id: u64,
        sender_id: u64,
    ) -> Result<LlmMessage> {
        self.send_msg_batch(
            LlmChat {
                chat_id,
                messages: vec![message],
            },
            sender_id,
        )
        .await
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
                error!("RESPONSE: ERR {:?}", err);
                return Err(err.into());
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmChat {
    pub chat_id: u64,
    pub messages: Vec<LlmMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LlmMessage {
    pub sender_id: u64,
    pub role: Role,
    pub content: String,
}
