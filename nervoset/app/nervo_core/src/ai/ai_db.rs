use crate::ai::nervo_llm::NervoLlm;
use crate::ai::qdrant_db::QdrantDb;
use crate::config::common::QdrantParams;
use anyhow::Result;
use async_openai::types::Embedding;
use qdrant_client::qdrant::SearchResponse;
use tracing::log::info;

pub struct NervoAiDb {
    pub qdrant: QdrantDb,
    pub nervo_llm: NervoLlm,
}

impl NervoAiDb {
    pub fn build(config: &QdrantParams, nervo_llm: NervoLlm) -> Result<Self> {
        let qdrant = QdrantDb::try_from(config, nervo_llm.clone())?;
        Ok(NervoAiDb { qdrant, nervo_llm })
    }
}

impl NervoAiDb {
    pub async fn text_search(
        &self,
        collection_name: &str,
        search_text: String,
        vectors_limit: u64,
    ) -> Result<SearchResponse> {
        info!("Starting QDrant db search...");
        self.qdrant
            .text_search(collection_name, search_text, vectors_limit)
            .await
    }

    pub async fn vector_search(
        &self,
        collection_name: &str,
        embedding: Embedding,
        limit: u64,
    ) -> Result<SearchResponse> {
        info!("Starting QDrant db search...");
        self.qdrant
            .vector_search(collection_name, embedding.embedding, limit)
            .await
    }
}
