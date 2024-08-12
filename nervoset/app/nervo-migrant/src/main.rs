mod models;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::process::exit;
use std::sync::Arc;
use anyhow::bail;
use tokio::fs;
use crate::models::migration_model::MigrationModel;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use nervo_api::{AppType, NervoAppType};
use nervo_bot_core::config::common::NervoConfig;
use nervo_bot_core::config::nervo_server::NervoServerAppState;
use crate::models::migration_path_model::{MigrationPlan, MigrationMetaData};

use clap::{Parser, Subcommand};

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
            update_json_structure(app_state, migration_plan).await?;
            info!("Dataset preparation step has been finished");
        }
        Commands::Migration => {
            // Update QDrant collection (remove old records in qdrant if needed)
            let app_state = initial_setup().await?;
            let datasets = collect_jsons_content("dataset".to_string()).await?;
            //let update_models = update_collection_with_json(datasets, &app_state).await?;
        }
    }

    Ok(())

}

async fn initial_setup() -> anyhow::Result<Arc<NervoServerAppState>> {
    let config = NervoConfig::load()?;
    let app_state = NervoServerAppState::try_from(config.nervo_server)?;
    let app_state = Arc::from(app_state);
    Ok(app_state)
}

async fn collect_jsons_content(dataset_path: String) -> anyhow::Result<Vec<MigrationPlan>> {
    info!("Start collecting all jsons ant paths");
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

        let app_type = NervoAppType::try_from(app_name_str);

        if app_type == AppType::None {
            bail!("Empty app name")
        }

        let mut data_models = vec![];

        if app_path.is_dir() {
            let mut app_dir_entries = fs::read_dir(&app_path).await?;
            while let Some(app_dir_entry) = app_dir_entries.next_entry().await? {
                let app_path_dataset = app_dir_entry.path();

                let mut subdir_entries = fs::read_dir(&app_path_dataset).await?;
                while let Some(file_entry) = subdir_entries.next_entry().await? {
                    let file_path = file_entry.path();

                    if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
                        let json_content = fs::read_to_string(&file_path).await?;
                        let migration_model: MigrationModel = serde_json::from_str(json_content.as_str())?;
                        let migration_data_model = MigrationMetaData {
                            json_path: file_path,
                            migration_model
                        };

                        data_models.push(migration_data_model);
                    }
                }
            }
        }

        let migration_config = MigrationPlan { app_type, data_models };

        result_vec.push(migration_config);
    }

    Ok(result_vec)
}

async fn update_collection_with_json(
    migration_models: Vec<MigrationPlan>,
    app_state: Arc<NervoServerAppState>,
) -> anyhow::Result<HashMap<String, MigrationModel>> {
    info!("Start updating QDrant collection");

    let mut result_dictionary: HashMap<String, MigrationModel> = HashMap::new();
    let qdrant_db = &app_state.nervo_ai_db.qdrant;

    /*
    for migration_model in migration_models.iter() {
        for data_model in migration_model.data_models.iter() {
            match serde_json::from_str::<MigrationModel>(&data_model.json_content) {
                Ok(mut migration_model) => {
                    // Need to delete an old version at first
                    /*if !migration_model.delete.is_empty() {
                        info!("Found old data. Need to delete");
                        delete_collection_content_in_qdrant(&migration_model, migration_model.app_name(), app_state).await?;
                    }*/

                    /*{
                        info!("Save text of json to qdrant collection");
                        let text = migration_model.clone().create.text;
                        let app_name = migration_model.app_name();
                        qdrant_db
                            .save_to_qdrant_db(app_name, text)
                            .await?;
                    }*/

                    if let Some(str_value) = data_model.json_path.to_str() {
                        result_dictionary.insert(String::from(str_value), migration_model);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse JSON. Err {}", e);
                }
            }
        }
    }
     */

    Ok(result_dictionary)
}

async fn delete_collection_content_in_qdrant(
    migration_model: &MigrationModel,
    app_name: String,
    app_state: &Arc<NervoServerAppState>,
) -> anyhow::Result<()> {
    for delete_item in migration_model.delete.iter() {
        let qdrant_db = &app_state.nervo_ai_db.qdrant;

        let _ = qdrant_db.delete_from_qdrant_db(
            app_state,
            app_name.clone(),
            delete_item.text.clone(),
        ).await?;
    }
    Ok(())
}

async fn update_json_structure(app_state: Arc<NervoServerAppState>, migration_plans: Vec<MigrationPlan>) -> anyhow::Result<()> {
    let qdrant_db = &app_state.nervo_ai_db.qdrant;
    
    for migration in migration_plans.iter() {
        for data_model in &migration.data_models {
            if data_model.migration_model.create.embedding.is_none() {
                // get embeddings and update model
                let embedding = qdrant_db
                    .nervo_llm
                    .text_to_embeddings(&data_model.migration_model.create.text)
                    .await?;
                
                let mut updated_model = data_model.migration_model.clone();
                updated_model.create.embedding = embedding;

                let json_file = File::create(data_model.json_path.clone())?;
                let writer = BufWriter::new(json_file);
                serde_json::to_writer_pretty(writer, &updated_model)?;
                
                info!("Embedding has been created for: {:?}", data_model.json_path);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use nervo_api::app_type;
    use crate::{collect_jsons_content};

    #[tokio::test]
    async fn test_collect_jsons_content() -> anyhow::Result<()> {
        let jsons_content = collect_jsons_content("../../dataset".to_string()).await?;
        assert_eq!(jsons_content.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_collect_jsons_content_one() -> anyhow::Result<()> {
        let jsons_content = collect_jsons_content("../../dataset".to_string()).await?;
        assert_eq!(jsons_content[0].app_name(), app_type::PROBIOT);
        assert_eq!(jsons_content[1].app_name(), app_type::JAISON);
        Ok(())
    }
}