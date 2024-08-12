use anyhow::bail;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{CreateCollection, DeletePointsBuilder, Distance, PointStruct, SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{SearchResponse, VectorParams, VectorsConfig};
use qdrant_client::Payload;
use rand::rngs::OsRng;
use rand::Rng;
use serde_json::json;
use tracing::info;
use crate::ai::nervo_llm::NervoLlm;
use crate::config::common::QdrantParams;
use crate::config::nervo_server::NervoServerAppState;

pub struct QdrantDb {
    pub qdrant_client: Qdrant,
    pub nervo_llm: NervoLlm,
}

impl QdrantDb {
    pub fn try_from(config: &QdrantParams, nervo_llm: NervoLlm) -> anyhow::Result<Self> {
        let qdrant_client = Qdrant::from_url(config.server_url.as_str())
            .api_key(config.api_key.clone())
            .build()?;

        Ok(QdrantDb { qdrant_client, nervo_llm })
    }
}


impl QdrantDb {
    pub async fn save_to_qdrant_db(
        &self,
        collection_name: String,
        text: String,
    ) -> anyhow::Result<()> {
        let mut rng = OsRng;

        let maybe_vec_data = self.nervo_llm
            .text_to_embeddings(text.as_str())
            .await?;

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
        collection_name: String,
        search_text: String,
        search_vectors_limit: u64,
    ) -> anyhow::Result<SearchResponse> {
        let maybe_vec_data = self.nervo_llm.text_to_embeddings(&search_text).await?;

        match maybe_vec_data {
            None => {
                bail!("No embedding data found.");
            }
            Some(vec_data) => {
                let builder = SearchPointsBuilder::new(
                    collection_name,
                    vec_data.embedding.clone(),
                    search_vectors_limit
                )
                    .with_payload(true)
                    .params(SearchParamsBuilder::default().exact(true));
                
                let search_result = self.qdrant_client
                    .search_points(builder)
                    .await?;
                
                Ok(search_result)
            }
        }
    }

    pub async fn delete_from_qdrant_db(
        &self,
        app_state: &NervoServerAppState,
        collection_name: String,
        text: String,
    ) -> anyhow::Result<DeleteResult> {
        let maybe_vec_data = app_state.nervo_llm.text_to_embeddings(&text).await?;

        let Some(vec_data) = maybe_vec_data else {
            bail!("No embedding data found.");
        };

        let builder = SearchPointsBuilder::new(
            collection_name.clone(),
            vec_data.embedding.clone(),
            1,
        )
            .with_payload(true)
            .params(SearchParamsBuilder::default().exact(true));
        
        let search_result = self
            .qdrant_client
            .search_points(builder)
            .await?;

        if search_result.result.is_empty() {
            info!("No matching points found.");
            return Ok(DeleteResult::Success);
        }

        let Some(point_id) = search_result.result[0].id.clone() else {
            info!("Couldn't find points");
            return Ok(DeleteResult::Fail);
        };

        let delete_request = DeletePointsBuilder::new(collection_name)
            .points(vec![point_id])
            .build();

        let delete_response = self.qdrant_client
            .delete_points(delete_request)
            .await?;

        if delete_response.result.is_some() {
            Ok(DeleteResult::Success)
        } else {
            Ok(DeleteResult::Fail)
        }
    }
}

pub enum DeleteResult {
    Success,
    Fail,
} 