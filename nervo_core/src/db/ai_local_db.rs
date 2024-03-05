use crate::db::nervo_message_model::TelegramMessage;
use anyhow::Error;
use config::Config as AppConfig;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteConnection;
use sqlx::{ConnectOptions, Row};
use std::str::FromStr;

pub async fn save_message(message: &TelegramMessage, username: &str) -> anyhow::Result<()> {
    let mut connection = create_table(username).await?;

    let message_json_result = serde_json::to_string(message);
    match message_json_result {
        Ok(message_json) => {
            let query = format!("INSERT INTO user_{} (message) VALUES (?)", username);
            sqlx::query(&query)
                .bind(&message_json)
                .execute(&mut connection)
                .await?;
            Ok(())
        }
        Err(err) => {
            println!("Cant serde JSON! Error occurred: {}", err);
            Err(Error::from(err))
        }
    }
}

pub async fn read_messages(username: &str) -> anyhow::Result<Vec<TelegramMessage>> {
    let mut connection = create_table(username).await?;
    let query = format!("SELECT message FROM user_{}", username);

    let rows = sqlx::query(&query).fetch_all(&mut connection).await?;

    let mut messages = Vec::new();

    for row in rows {
        let message_json: String = row.try_get("message")?;
        let message: TelegramMessage = serde_json::from_str(&message_json)?;

        messages.push(message);
    }

    Ok(messages)
}

async fn create_table(username: &str) -> anyhow::Result<SqliteConnection> {
    let mut connection = connect_db().await?;
    let table_exists: bool = sqlx::query_scalar(&format!(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'user_{}')",
        username
    ))
        .fetch_one(&mut connection)
        .await?;

    if !table_exists {
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS user_{} (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               message TEXT,
               timestamp TEXT
           )",
            username
        ))
            .execute(&mut connection)
            .await?;
    }

    Ok(connection)
}

async fn connect_db() -> anyhow::Result<SqliteConnection> {
    let app_config_result = AppConfig::builder()
        .add_source(config::File::with_name("config"))
        .build();
    match app_config_result {
        Ok(app_config) => {
            let database_url_result = app_config.get_string("database_url");
            match database_url_result {
                Ok(database_url) => {
                    let conn = SqliteConnectOptions::from_str(&database_url)?
                        .create_if_missing(true)
                        .connect()
                        .await?;

                    Ok(conn)
                }
                Err(err) => {
                    println!("Wrong DB url! Error occurred: {}", err);
                    Err(Error::from(err))
                }
            }
        }
        Err(err) => {
            println!("No AppConfig! Error occurred: {}", err);
            Err(Error::from(err))
        }
    }
}
