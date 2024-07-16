use anyhow::bail;
use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessageContent, Role,
};
use async_openai::types::ChatCompletionRequestUserMessage;
use async_openai::types::CreateChatCompletionRequestArgs;
use async_openai::types::CreateEmbeddingRequestArgs;
use async_openai::types::CreateEmbeddingResponse;
use async_openai::types::CreateModerationRequest;
use async_openai::types::CreateTranscriptionRequest;
use async_openai::types::ModerationInput;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub async fn embedding(&self, text: &str) -> Result<CreateEmbeddingResponse> {
        let response = {
            let embedding = CreateEmbeddingRequestArgs::default()
                .model(self.llm_config.embedding_model_name.clone())
                .input(text)
                .build()?;

            self.client
                .embeddings()
                .create(embedding)
                .await?
        };

        Ok(response)
    }
}

impl NervoLlm {
    pub async fn send_msg_batch(&self, chat: LlmChat) -> Result<String> {
        let mut messages = vec![];
        for msg in chat.messages {
            let gpt_msg = ChatCompletionRequestMessage::try_from(msg)?;
            messages.push(gpt_msg);
        }

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(self.llm_config.max_tokens)
            .model(self.llm_config.model_name.clone())
            .temperature(self.llm_config.temperature)
            .messages(messages)
            .build()?;

        let chat_response = self.client.chat()
            .create(request)
            .await?;

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

    pub async fn send_msg(
        &self,
        message: LlmMessage,
        chat_id: u64,
    ) -> Result<String> {
        let chat = LlmChat {
            chat_id,
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
        Ok(!response.results.iter().any(|property| property.flagged) && (text.len() < 10000))
    }

    pub async fn voice_transcription(
        &self,
        request: CreateTranscriptionRequest,
    ) -> Result<String> {
        let response = self.client.audio().transcribe(request).await?;
        Ok(response.text)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmChat {
    pub chat_id: u64,
    pub messages: Vec<LlmMessage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmMessage {
    pub save_to_context: LlmSaveContext,
    pub message_owner: LlmOwnerType,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LlmSaveContext {
    True,
    False,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub enum LlmOwnerType {
    System(LlmMessageContent),
    User(UserLlmMessage),
    Assistant(LlmMessageContent),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserLlmMessage {
    pub sender_id: u64,
    pub content: LlmMessageContent,
}

impl LlmMessage {
    pub fn role(&self) -> String {
        match self.message_owner {
            LlmOwnerType::System(_) => String::from("system"),
            LlmOwnerType::User(_) => String::from("user"),
            LlmOwnerType::Assistant(_) => String::from("assistant")
        }
    }

    pub fn content_text(&self) -> String {
        match &self.message_owner {
            LlmOwnerType::System(LlmMessageContent(content)) => content.clone(),
            LlmOwnerType::User(UserLlmMessage { content: LlmMessageContent(text), .. }) => text.clone(),
            LlmOwnerType::Assistant(LlmMessageContent(content)) => content.clone(),
        }
    }

    pub fn content(&self) -> LlmMessageContent {
        match &self.message_owner {
            LlmOwnerType::System(content) => content.clone(),
            LlmOwnerType::User(user) => user.content.clone(),
            LlmOwnerType::Assistant(content) => content.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmMessageContent(pub String);

impl LlmMessageContent {
    pub fn text(&self) -> String {
        self.0.clone()
    }
}

impl From<&str> for LlmMessageContent {
    fn from(content: &str) -> Self {
        LlmMessageContent(content.to_string())
    }
}

impl TryFrom<LlmMessage> for ChatCompletionRequestMessage {
    type Error = anyhow::Error;

    fn try_from(msg: LlmMessage) -> std::result::Result<Self, Self::Error> {
        match msg.message_owner {
            LlmOwnerType::System(LlmMessageContent(content)) => {
                let message = ChatCompletionRequestSystemMessage {
                    content,
                    role: Role::System,
                    name: None,
                };
                Ok(ChatCompletionRequestMessage::from(message))
            }
            LlmOwnerType::User(UserLlmMessage { sender_id, content }) => {
                let message = ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(content.0),
                    role: Role::User,
                    name: Some(sender_id.to_string()),
                };
                Ok(ChatCompletionRequestMessage::from(message))
            }
            LlmOwnerType::Assistant(LlmMessageContent(content)) => {
                let message = ChatCompletionRequestAssistantMessage {
                    content: Some(content),
                    role: Role::Assistant,
                    name: None,
                    tool_calls: None,
                    function_call: None,
                };

                Ok(ChatCompletionRequestMessage::from(message))
            }
        }
    }
}
