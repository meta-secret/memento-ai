use crate::ai::qdrant_db::QdrantDb;
use anyhow::Result;
use qdrant_client::qdrant::SearchResponse;
use tracing::log::info;
use crate::ai::nervo_llm::NervoLlm;
use crate::config::common::QdrantParams;

pub struct NervoAiDb {
    pub qdrant: QdrantDb,
    pub nervo_llm: NervoLlm
}

impl NervoAiDb {
    pub fn build(config: &QdrantParams, nervo_llm: NervoLlm) -> Result<Self> {
        let qdrant = QdrantDb::try_from(config, nervo_llm.clone())?;
        Ok(NervoAiDb { qdrant, nervo_llm })
    }
}

impl NervoAiDb {
    pub async fn search(
        &self,
        collection_name: String,
        search_text: String,
        vectors_limit: u64,
    ) -> Result<SearchResponse> {
        info!("Starting QDrant db search...");
        self.qdrant
            .vector_search(collection_name, search_text, vectors_limit)
            .await
    }

    pub async fn save(&self, user_id: u64, text: &str) -> Result<()> {
        self.qdrant
            .save(user_id.to_string(), text)
            .await
    }
}
