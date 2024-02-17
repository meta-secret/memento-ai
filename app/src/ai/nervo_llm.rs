use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateEmbeddingRequest, CreateEmbeddingRequestArgs, CreateEmbeddingResponse};
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
    pub async fn embedding(&self, text: String) -> anyhow::Result<CreateEmbeddingResponse> {
        let embedding = CreateEmbeddingRequestArgs::default()
            .model(self.embedding_model_name.clone())
            .input(text)
            .build()?;
        let response = self.client.embeddings().create(embedding).await?;
        Ok(response)
    }
}

impl NervoLlm {
    pub async fn chat(&self, message: String) -> anyhow::Result<Option<String>> {
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(self.max_tokens)
            .model(self.model_name.clone())
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are a helpful assistant.")
                    .build()
                    .unwrap()
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(message)
                    .build()
                    .unwrap()
                    .into(),
            ])
            .build()?;

        let chat_response = self.client.chat().create(request).await?;
        let maybe_reply = chat_response.choices.first();
        let maybe_msg = maybe_reply.and_then(|reply| reply.message.content.clone());

        Ok(maybe_msg)
    }
}

/*
ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant.")
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("Who won the world series in 2020?")
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestAssistantMessageArgs::default()
                .content("The Los Angeles Dodgers won the World Series in 2020.")
                .build()
                .unwrap()
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("Where was it played?")
                .build()
                .unwrap()
                .into(),
 */
