use crate::common::NervoConfig;

use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteConnection;
use sqlx::{ConnectOptions, Row};
use std::str::FromStr;
use tracing::{debug};

pub struct LocalDb {
    app_config: NervoConfig,
}

impl LocalDb {
    pub async fn try_init(app_config: NervoConfig) -> anyhow::Result<Self> {
        Ok(Self { app_config })
    }
}

impl LocalDb {
    async fn connect_db(&self) -> anyhow::Result<SqliteConnection> {
        let conn = SqliteConnectOptions::from_str(self.app_config.database_url.as_str())?
            .create_if_missing(true)
            .connect()
            .await?;
        Ok(conn)
    }

    pub async fn save_message<T>(
        &self,
        message: T,
        table_name: &str,
        need_restriction: bool,
    ) -> anyhow::Result<()>
    where
        T: Serialize + Send + 'static,
    {
        debug!("save message");
        self.create_table(table_name).await?;

        let items_count = self.count_items(table_name).await?;
        if items_count >= 10 && need_restriction {
            self.overwrite_messages(message, table_name).await?;
        } else {
            self.insert_message(message, table_name).await?;
        }

        Ok(())
    }

    pub async fn read_messages<T>(&self, table_name: &str) -> anyhow::Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        self.create_table(table_name).await?;
        let query = format!("SELECT message FROM table_{}", table_name);
        let mut conn = self.connect_db().await?;
        let rows = sqlx::query(&query).fetch_all(&mut conn).await?;
        let mut messages = Vec::new();

        for row in rows {
            let message_json: String = row.try_get("message")?;
            let message = serde_json::from_str(&message_json)?;

            messages.push(message);
        }

        Ok(messages)
    }

    async fn create_table(&self, table_name: &str) -> anyhow::Result<()> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'table_{}')",
            table_name
        );

        let mut conn = self.connect_db().await?;

        let table_exists: bool = sqlx::query_scalar(&query).fetch_one(&mut conn).await?;

        if !table_exists {
            let query = format!(
                "CREATE TABLE IF NOT EXISTS table_{} (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               message TEXT,
               timestamp TEXT
           )",
                table_name
            );

            sqlx::query(&query).execute(&mut conn).await?;
        }

        Ok(())
    }

    async fn count_items(&self, table_name: &str) -> anyhow::Result<i64> {
        let query = format!("SELECT COUNT(*) FROM table_{}", table_name);
        let mut conn = self.connect_db().await?;
        let count: i64 = sqlx::query_scalar(&query).fetch_one(&mut conn).await?;
        Ok(count)
    }

    async fn overwrite_messages<T>(&self, message: T, table_name: &str) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.connect_db().await?;
        let sql = format!(
            "SELECT id FROM table_{} ORDER BY id ASC LIMIT 1",
            table_name
        );
        let oldest_message_id: Option<i64> =
            sqlx::query_scalar(&sql).fetch_optional(&mut conn).await?;

        if let Some(id) = oldest_message_id {
            sqlx::query(&format!("DELETE FROM table_{} WHERE id = ?", table_name))
                .bind(id)
                .execute(&mut conn)
                .await?;
        }

        self.insert_message(message, table_name).await?;

        Ok(())
    }

    async fn insert_message<T>(&self, message: T, table_name: &str) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        let message_json = serde_json::to_string(&message)?;
        let mut conn = self.connect_db().await?;

        let query = format!("INSERT INTO table_{} (message) VALUES (?)", &table_name);
        sqlx::query(&query)
            .bind(&message_json)
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}
