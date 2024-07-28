use crate::common::{AppState, QdrantParams};
use anyhow::bail;
use async_openai::types::Embedding;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    CreateCollection,
    Distance,
    PointStruct,
    SearchParamsBuilder,
    SearchPointsBuilder,
    UpsertPointsBuilder
};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{SearchResponse, VectorParams, VectorsConfig};
use qdrant_client::Payload;
use rand::rngs::OsRng;
use rand::Rng;
use serde_json::json;


pub struct QdrantDb {
    pub qdrant_client: Qdrant,
}


impl TryFrom<&QdrantParams> for QdrantDb {
    type Error = anyhow::Error;

    fn try_from(config: &QdrantParams) -> anyhow::Result<Self, Self::Error> {
        let qdrant_client = Qdrant::from_url(config.server_url.as_str())
            .api_key(config.api_key.clone())
            .build()?;

        Ok(QdrantDb { qdrant_client })
    }
}


impl QdrantDb {
    pub async fn save_to_qdrant_db(
        &self,
        app_state: &AppState,
        collection_name: String,
        text: String,
    ) -> anyhow::Result<()> {
        let mut rng = OsRng;

        let maybe_vec_data = self.text_to_embeddings(app_state, &text).await?;

        let Some(vec_data) = maybe_vec_data else {
            bail!("No embedding data found.");
        };

        let col_exists = self
            .qdrant_client
            .collection_exists(&collection_name)
            .await?;

        if !col_exists {
            let details = CreateCollection {
                collection_name: collection_name.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: vec_data.embedding.len() as u64,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            };

            self.qdrant_client.create_collection(details).await?;
        }

        let points = {
            let id: u64 = rng.gen();
            let payload: Payload = json!({"text": text}).try_into().unwrap();
            let point = PointStruct::new(id, vec_data.embedding.clone(), payload);
            vec![point]
        };

        self.qdrant_client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;
        Ok(())
    }


    pub async fn search_in_qdrant_db(
        &self,
        app_state: &AppState,
        collection_name: String,
        search_text: String,
        search_vectors_limit: u64,
    ) -> anyhow::Result<SearchResponse> {
        let maybe_vec_data = self.text_to_embeddings(app_state, &search_text).await?;

        match maybe_vec_data {
            None => {
                bail!("No embedding data found.");
            }
            Some(vec_data) => {
                let search_result = self
                    .qdrant_client
                    .search_points(
                        SearchPointsBuilder::new(
                            collection_name,
                            vec_data.embedding
                                .clone(),
                            search_vectors_limit
                        )
                            .with_payload(true)
                            .params(SearchParamsBuilder::default()
                                .exact(true))
                    )
                    .await?;
                Ok(search_result)
            }
        }
    }


    async fn text_to_embeddings(
        &self,
        app_state: &AppState,
        text: &str,
    ) -> anyhow::Result<Option<Embedding>> {
        let embedding = app_state.nervo_llm.embedding(text).await?;
        Ok(embedding.data.first().cloned())
    }
}
