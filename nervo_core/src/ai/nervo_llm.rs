use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs, CreateEmbeddingResponse, CreateModerationRequest, ModerationInput};
use async_openai::Client;

pub struct NervoLlm {
    model_name: String,
    embedding_model_name: String,

    max_tokens: u16,
    client: Client<OpenAIConfig>,
}

impl From<OpenAIConfig> for NervoLlm {
    fn from(config: OpenAIConfig) -> Self {
        NervoLlm {
            model_name: String::from("gpt-3.5-turbo"),
            embedding_model_name: String::from("text-embedding-3-small"),
            //embedding_model_name: String::from("text-embedding-3-large"),
            max_tokens: 256u16,
            client: Client::with_config(config),
        }
    }
}

impl NervoLlm {
    pub async fn embedding(&self, text: &str) -> anyhow::Result<CreateEmbeddingResponse> {
        let embedding = CreateEmbeddingRequestArgs::default()
            .model(self.embedding_model_name.clone())
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
            .max_tokens(self.max_tokens)
            .model(self.model_name.clone())
            .messages(messages)
            .build()?;

        let chat_response = self.client.chat().create(request).await?;
        let maybe_reply = chat_response.choices.first();
        let maybe_msg = maybe_reply.and_then(|reply| reply.message.content.clone());

        Ok(maybe_msg)
    }

    pub async fn chat(
        &self,
        message: ChatCompletionRequestUserMessage,
    ) -> anyhow::Result<Option<String>> {
        let messages = vec![ChatCompletionRequestMessage::from(message)];
        self.chat_batch(messages).await
    }

    pub async fn moderate(&self, text: &str) -> anyhow::Result<bool> {
        let request = CreateModerationRequest {
            input: ModerationInput::from(text),
            model: None,
        };

        let response = self.client.moderations().create(request).await?;
        Ok(!response.results.iter().any(|property| property.flagged))
    }
}
