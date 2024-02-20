use anyhow::bail;
use anyhow::Result;
use async_openai::types::{CreateEmbeddingResponse, Embedding};
use config::Config as AppConfig;
use qdrant_client::prelude::*;
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::PointStruct;
use qdrant_client::qdrant::{
    Condition, Filter, PointsOperationResponse, SearchResponse, VectorParams, Vectors,
    VectorsConfig,
};
use rand::rngs::OsRng;
use rand::Rng;
use serde_json::json;

pub struct NervoAiDb {
    pub qdrant_client: QdrantClient,
}

impl TryFrom<&AppConfig> for NervoAiDb {
    type Error = anyhow::Error;

    fn try_from(config: &AppConfig) -> Result<Self, Self::Error> {
        let qdrant_server_url = config.get_string("qdrant_server_url")?;
        let qdrant_api_key = config.get_string("qdrant_api_key")?;

        let qdrant_client = QdrantClient::from_url(qdrant_server_url.as_str())
            // using an env variable for the API KEY for example
            .with_api_key(qdrant_api_key)
            .build()?;

        Ok(NervoAiDb { qdrant_client })
    }
}

impl NervoAiDb {
    pub async fn search(
        &self,
        user_id: u64,
        embedding: CreateEmbeddingResponse,
    ) -> Result<SearchResponse> {
        let collection_name = user_id.to_string();

        let maybe_vec_data = embedding.data.first();

        match maybe_vec_data {
            None => {
                bail!("No embedding data found.");
            }
            Some(vec_data) => {
                let search_result = self
                    .qdrant_client
                    .search_points(&SearchPoints {
                        collection_name: collection_name.into(),
                        vector: vec_data.embedding.clone(),
                        //filter: Some(Filter::all([Condition::matches("text", )])),
                        limit: 10,
                        with_payload: Some(true.into()),
                        ..Default::default()
                    })
                    .await?;

                Ok(search_result)
            }
        }
    }

    pub async fn save(
        &self,
        user_id: u64,
        embedding: CreateEmbeddingResponse,
        text: String,
    ) -> Result<PointsOperationResponse> {
        let mut rng = OsRng;

        let maybe_vec_data = embedding.data.first();

        let collection_name = user_id.to_string();

        match maybe_vec_data {
            None => {
                bail!("No embedding data found.");
            }
            Some(vec_data) => {
                let col_exists = self
                    .qdrant_client
                    .has_collection(user_id.to_string())
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

                    self.qdrant_client.create_collection(&details).await?;
                }

                let points = {
                    let id: u64 = rng.gen();
                    let point = PointStruct::new(
                        id,
                        vec_data.embedding.clone(),
                        json!({"text": text}).try_into().unwrap(),
                    );
                    vec![point]
                };

                self.qdrant_client
                    .upsert_points_blocking(collection_name, None, points, None)
                    .await
            }
        }
    }
}
