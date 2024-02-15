use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, CreateEmbeddingRequest, CreateEmbeddingRequestArgs};
use async_openai::Client;

pub struct NervoAiClient {
    model_name: String,
    max_tokens: u16,
    client: Client<OpenAIConfig>,
}

impl From<OpenAIConfig> for NervoAiClient {
    fn from(config: OpenAIConfig) -> Self {
        NervoAiClient {
            model_name: String::from("gpt-3.5-turbo"),
            max_tokens: 256u16,
            client: Client::with_config(config),
        }
    }
}

impl NervoAiClient {
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

        let text = CreateEmbeddingRequestArgs::default()
            .model("text-embedding-3-small")
            .input([
                "Why do programmers hate nature? It has too many bugs.",
                "Why was the computer cold? It left its Windows open.",
            ])
            .build()?;
        self.client.embeddings().create(text).await?;

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
