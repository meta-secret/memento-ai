use crate::ai::nervo_llm::NervoLlm;
use crate::config::common::QdrantParams;
use crate::utils::cryptography::UuidGenerator;
use anyhow::bail;
use anyhow::Result;
use nervo_api::agent_type::{AgentType, NervoAgentType};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{
    CreateCollection, DeletePointsBuilder, Distance, GetPointsBuilder,
    GetResponse, PointStruct, SearchParamsBuilder,
    SearchPointsBuilder, UpsertPointsBuilder,
};
use qdrant_client::qdrant::{SearchResponse, VectorParams, VectorsConfig};
use qdrant_client::Payload;
use qdrant_client::Qdrant;
use serde_json::json;
use tracing::info;
use uuid::Uuid;

pub struct QdrantDb {
    pub qdrant_client: Qdrant,
    pub nervo_llm: NervoLlm,
}

impl QdrantDb {
    pub fn try_from(config: &QdrantParams, nervo_llm: NervoLlm) -> Result<Self> {
        let qdrant_client = Qdrant::from_url(config.server_url.as_str())
            .api_key(config.api_key.clone())
            .build()?;

        Ok(QdrantDb {
            qdrant_client,
            nervo_llm,
        })
    }
}

impl QdrantDb {
    pub async fn save(&self, agent_type: AgentType, text: &str) -> Result<()> {
        let collection_name = NervoAgentType::get_name(agent_type);

        let maybe_vec_data = self.nervo_llm.text_to_embeddings(text).await?;

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
            let point_id = UuidGenerator::from(text).to_string();

            let payload: Payload = json!({"text": text}).try_into()?;
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
        agent_type: AgentType,
        search_text: String,
        search_vectors_limit: u64,
    ) -> Result<SearchResponse> {
        let collection_name = NervoAgentType::get_name(agent_type);

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

                let search_result = self.qdrant_client.search_points(builder).await?;

                Ok(search_result)
            }
        }
    }

    pub async fn find_by_id(&self, agent_type: AgentType, id: Uuid) -> Result<GetResponse> {
        let collection_name = NervoAgentType::get_name(agent_type);
        let col_exists = self
            .qdrant_client
            .collection_exists(&collection_name)
            .await?;

        if col_exists {
            let col_name = NervoAgentType::get_name(agent_type);
            let query = GetPointsBuilder::new(col_name, vec![id.to_string().into()]);

            let search_result = self.qdrant_client.get_points(query).await?;

            Ok(search_result)
        } else {
            Ok(GetResponse::default())
        }
    }

    pub async fn delete_by_id(&self, agent_type: AgentType, id: Uuid) -> Result<DeleteResult> {
        let col_name = NervoAgentType::get_name(agent_type);
        let search_result = self.find_by_id(agent_type, id).await?;

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

        let delete_response = self.qdrant_client.delete_points(delete_request).await?;

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
