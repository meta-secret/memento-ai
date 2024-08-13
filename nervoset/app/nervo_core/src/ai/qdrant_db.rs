use anyhow::bail;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{Condition, CreateCollection, DeletePointsBuilder, Distance, Filter, PointStruct, ScrollPointsBuilder, ScrollResponse, SearchParamsBuilder, SearchPointsBuilder, UpsertPointsBuilder};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{SearchResponse, VectorParams, VectorsConfig};
use qdrant_client::Payload;
use serde_json::json;
use tracing::info;
use crate::ai::nervo_llm::NervoLlm;
use crate::config::common::QdrantParams;
use uuid::Uuid;
use crate::utils::cryptography;
use anyhow::Result;

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
    pub async fn save(&self, collection_name: String, text: &str) -> Result<()> {
        let maybe_vec_data = self.nervo_llm
            .text_to_embeddings(text)
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
            // Generate a UUID
            let point_id = Uuid::new_v4().to_string();
            let idd = cryptography::generate_uuid(text)?;
            let idd = idd.to_string();

            let payload: Payload = json!({"idd": idd, "text": text}).try_into()?;
            let point = PointStruct::new(point_id, vec_data.embedding.clone(), payload);
            vec![point]
        };

        self.qdrant_client
            .upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;
        Ok(())
    }


    pub async fn vector_search(
        &self,
        collection_name: String,
        search_text: String,
        search_vectors_limit: u64,
    ) -> Result<SearchResponse> {
        let maybe_vec_data = self.nervo_llm.text_to_embeddings(&search_text).await?;

        match maybe_vec_data {
            None => {
                bail!("No embedding data found.");
            }
            Some(vec_data) => {
                let builder = SearchPointsBuilder::new(
                    collection_name,
                    vec_data.embedding.clone(),
                    search_vectors_limit,
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

    pub async fn find_by_idd(&self, col_name: &str, text: &str) -> Result<ScrollResponse> {
        let idd = cryptography::generate_uuid(text)?;
        let idd = idd.to_string();

        let filter = ScrollPointsBuilder::new(col_name)
            .filter(Filter::must([Condition::matches("idd", idd)]));

        let search_result = self.qdrant_client.scroll(filter).await?;
        Ok(search_result)
    }

    pub async fn delete_by_idd(&self, col_name: &str, text: &str) -> Result<DeleteResult> {
        let search_result = self.find_by_idd(col_name, text).await?;
        
        if search_result.result.is_empty() {
            info!("No matching points found.");
            return Ok(DeleteResult::Success);
        }

        let Some(point_id) = search_result.result[0].id.clone() else {
            info!("Couldn't find any points");
            return Ok(DeleteResult::Success);
        };

        let delete_request = DeletePointsBuilder::new(col_name)
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