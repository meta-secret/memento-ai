use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
    CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs, CreateEmbeddingResponse,
    CreateModerationRequest, ModerationInput,
};

#[derive(Clone, Debug)]
pub struct NervoLlmConfig {
    model_name: String,
    embedding_model_name: String,
    max_tokens: u16,
    temperature: f32,
    open_ai_config: OpenAIConfig
}

impl From<OpenAIConfig> for NervoLlmConfig {
    fn from(open_ai_config: OpenAIConfig) -> Self {
        NervoLlmConfig {
            model_name: String::from("gpt-3.5-turbo"),
            embedding_model_name: String::from("text-embedding-3-small"),
            //embedding_model_name: String::from("text-embedding-3-large"),
            max_tokens: 512u16,
            temperature: 0.1f32,
            open_ai_config,
        }       
    }
}

impl NervoLlmConfig {
    pub fn with_model_name(mut self, model_name: String) -> Self {
        self.model_name = model_name;
        self
    }
    
    pub fn model_name(&self) -> &str {
        &self.model_name
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
            client: Client::with_config(llm_config.open_ai_config)  
        }
    }
}

impl NervoLlm {
    pub fn model_name(&self) -> &str {
        self.llm_config.model_name()
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
