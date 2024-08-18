mod models;

use crate::models::migration_model::{DataSample, MigrationModel, VectorData};
use crate::models::migration_path_model::{MigrationMetaData, MigrationPlan};
use anyhow::bail;
use nervo_bot_core::config::common::NervoConfig;
use nervo_bot_core::config::jarvis::JarvisAppState;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use clap::{Parser, Subcommand};
use futures::future::BoxFuture;
use futures::FutureExt;
use nervo_api::agent_type::{AgentType, NervoAgentType};
use nervo_bot_core::utils::cryptography::UuidGenerator;
use tokio::time::Instant;
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Dataset,
    Migration,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"))
        .add_directive("hyper=info".parse()?)
        .add_directive("h2=info".parse()?)
        .add_directive("tower=info".parse()?)
        .add_directive("sqlx=info".parse()?);

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // Use a more compact, abbreviated log format
        .compact()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // completes the builder.
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Start migrant app");
    // General Preparations. Getting QDrant DB client, app name
    let app_state = initial_setup().await?;

    //parse cli arguments
    let cli = Cli::parse();

    match cli.command {
        Commands::Dataset => {
            //work with json:
            // - update json files with embeddings
            // - commit and push changes to GitHub (manually)
            info!("Dataset preparation has been started");
            let migration_plan = collect_jsons_content("../../dataset".to_string()).await?;
            enrich_datasets_with_embeddings(app_state, migration_plan).await?;
            info!("Dataset preparation step has been finished");
        }
        Commands::Migration => {
            let start = Instant::now();

            // Update qdrant collection (remove old records in qdrant if needed)
            info!("Migration preparation has been started");
            let migration_plan = collect_jsons_content("../../dataset".to_string()).await?;
            migrate_qdrant_db(migration_plan, app_state).await?;

            let duration = start.elapsed();
            info!("Migration completed for: {:?}", duration);
        }
    }

    Ok(())
}

async fn initial_setup() -> anyhow::Result<Arc<JarvisAppState>> {
    let config = NervoConfig::load()?;
    let app_state = JarvisAppState::try_from(config.apps.jarvis)?;
    let app_state = Arc::from(app_state);
    Ok(app_state)
}

async fn collect_jsons_content(dataset_path: String) -> anyhow::Result<Vec<MigrationPlan>> {
    info!("Start collecting all jsons and paths");
    let mut result_vec: Vec<MigrationPlan> = vec![];

    let mut dir_entries = fs::read_dir(&dataset_path).await?;
    while let Some(entry) = dir_entries.next_entry().await? {
        let app_path = entry.path();

        let Some(file_name) = app_path.file_name() else {
            bail!("Invalid dataset project structure")
        };

        let Some(app_name_str) = file_name.to_str() else {
            bail!("Invalid app name")
        };

        let agent_type = NervoAgentType::try_from(app_name_str);

        if agent_type == AgentType::None {
            bail!("Empty app name")
        }

        let data_models = if app_path.is_dir() {
            find_json_files(app_path).await?
        } else {
            vec![]
        };

        let migration_config = MigrationPlan {
            agent_type,
            data_models,
        };

        result_vec.push(migration_config);
    }

    Ok(result_vec)
}

fn find_json_files(
    dir_path: PathBuf,
) -> BoxFuture<'static, anyhow::Result<Vec<MigrationMetaData>>> {
    async move {
        let mut data_models = vec![];
        let mut dir_entries = fs::read_dir(&dir_path).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();

            if path.is_dir() {
                let mut nested_data_models = find_json_files(path).await?;
                data_models.append(&mut nested_data_models);
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json_content = fs::read_to_string(&path).await?;
                let migration_model: MigrationModel = serde_json::from_str(&json_content)?;
                let migration_data_model = MigrationMetaData {
                    json_path: path,
                    migration_model,
                };

                data_models.push(migration_data_model);
            }
        }

        Ok(data_models)
    }
    .boxed()
}

async fn migrate_qdrant_db(
    plans: Vec<MigrationPlan>,
    app_state: Arc<JarvisAppState>,
) -> anyhow::Result<()> {
    info!("Start updating Qdrant collection");

    let qdrant_db = &app_state.nervo_ai_db.qdrant;

    // delete old records
    for migration_plan in plans.iter() {
        for migration_info in migration_plan.data_models.iter() {
            let migration_model = &migration_info.migration_model;

            // Need to delete an old version at first
            delete_action(
                migration_model,
                migration_plan.agent_type,
                app_state.clone(),
            )
            .await?;

            let data_sample = migration_model.clone().create;
            let id = data_sample.id.unwrap();
            let text = data_sample.text;
            let embedding = data_sample.vector.unwrap().embedding;

            //check if qdrant already has this record
            let records = qdrant_db
                .find_by_id(migration_plan.agent_type, Uuid::parse_str(id.as_str())?)
                .await?;
            if records.result.is_empty() {
                info!(
                    "Save text of {:?} to qdrant: {:?}",
                    migration_info.json_path, migration_plan.agent_type
                );
                qdrant_db
                    .save(migration_plan.agent_type, text.as_str(), embedding)
                    .await?;
            }
        }
    }

    Ok(())
}

async fn delete_action(
    migration_model: &MigrationModel,
    agent_type: AgentType,
    app_state: Arc<JarvisAppState>,
) -> anyhow::Result<()> {
    for delete_item in migration_model.delete.iter() {
        let qdrant_db = &app_state.nervo_ai_db.qdrant;

        let id = delete_item
            .id
            .as_ref()
            .map(|id_val| Uuid::parse_str(id_val.as_str()))
            .unwrap()?;

        let _ = qdrant_db.delete_by_id(agent_type, id).await?;
    }
    Ok(())
}

async fn enrich_datasets_with_embeddings(
    app_state: Arc<JarvisAppState>,
    migration_plans: Vec<MigrationPlan>,
) -> anyhow::Result<()> {
    let qdrant_db = &app_state.nervo_ai_db.qdrant;

    for migration in migration_plans.iter() {
        for data_model in &migration.data_models {
            let mut need_update = false;

            let mut updated_model = data_model.migration_model.clone();

            if data_model.migration_model.create.id.is_none() {
                let text = data_model.migration_model.create.text.as_str();
                let id = UuidGenerator::from(text).to_string();

                updated_model.create.id = Some(id);

                need_update = true;
            }

            if data_model.migration_model.create.vector.is_none() {
                // get embeddings and update model
                let text = data_model.migration_model.create.text.as_str();

                let embedding = qdrant_db.nervo_llm.text_to_embeddings(text).await?.unwrap();

                let model_name = Some(app_state.nervo_config.llm.embedding_model_name.clone());
                updated_model.create.vector = Some(VectorData {
                    embedding,
                    embedding_model_name: model_name,
                });

                need_update = true;
            }

            if need_update {
                save_updated_model_to_json(data_model, &updated_model)?;
                info!("Dataset has been updated: {:?}", data_model.json_path);
            }
        }
    }

    Ok(())
}

fn save_updated_model_to_json(
    data_model: &MigrationMetaData,
    updated_model: &MigrationModel,
) -> anyhow::Result<()> {
    // save updated model to json
    let json_file = File::create(data_model.json_path.clone())?;
    let writer = BufWriter::new(json_file);
    serde_json::to_writer_pretty(writer, &updated_model)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::collect_jsons_content;
    use nervo_api::agent_type::AgentType;

    #[tokio::test]
    async fn test_collect_jsons_content() -> anyhow::Result<()> {
        let jsons_content = collect_jsons_content("../../dataset".to_string()).await?;
        assert_eq!(jsons_content.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_jsons_content_one() -> anyhow::Result<()> {
        let jsons_content = collect_jsons_content("../../dataset".to_string()).await?;
        let apps: Vec<AgentType> = jsons_content.iter().map(|plan| plan.agent_type).collect();

        assert!(apps.contains(&AgentType::Probiot));
        assert!(apps.contains(&AgentType::W3a));

        Ok(())
    }
}
