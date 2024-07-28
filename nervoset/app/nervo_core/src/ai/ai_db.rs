use crate::ai::qdrant_utils::QdrantDb;
use crate::common::{AppState, QdrantParams};
use anyhow::Result;
use qdrant_client::qdrant::SearchResponse;
use tracing::log::info;

pub struct NervoAiDb {
    pub qdrant: QdrantDb,
}

impl TryFrom<&QdrantParams> for NervoAiDb {
    type Error = anyhow::Error;

    fn try_from(config: &QdrantParams) -> Result<Self, Self::Error> {
        let qdrant = QdrantDb::try_from(config)?;
        Ok(NervoAiDb { qdrant })
    }
}

impl NervoAiDb {
    pub async fn search(
        &self,
        app_state: &AppState,
        collection_name: String,
        search_text: String,
        vectors_limit: u64,
    ) -> Result<SearchResponse> {
        info!("Starting QDrant db search...");
        self.qdrant
            .search_in_qdrant_db(app_state, collection_name, search_text, vectors_limit)
            .await
    }

    pub async fn save(&self, app_state: &AppState, user_id: u64, text: String) -> Result<()> {
        self.qdrant
            .save_to_qdrant_db(app_state, user_id.to_string(), text)
            .await
    }
}
