use crate::common::NervoConfig;
use crate::db::nervo_message_model::TelegramMessage;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteConnection;
use sqlx::{ConnectOptions, Row};
use std::str::FromStr;

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

    pub async fn save_message(
        &self,
        message: TelegramMessage,
        username: &str,
    ) -> anyhow::Result<()> {
        self.create_table(username).await?;

        let message_count = self.count_messages(username).await?;
        if message_count >= 10 {
            self.overwrite_messages(message, username).await?;
        } else {
            self.insert_message(message, username).await?;
        }

        Ok(())
    }

    pub async fn read_messages(&self, username: &str) -> anyhow::Result<Vec<TelegramMessage>> {
        self.create_table(username).await?;
        let query = format!("SELECT message FROM user_{}", username);

        let mut conn = self.connect_db().await?;
        let rows = sqlx::query(&query).fetch_all(&mut conn).await?;

        let mut messages = Vec::new();

        for row in rows {
            let message_json: String = row.try_get("message")?;
            let message: TelegramMessage = serde_json::from_str(&message_json)?;

            messages.push(message);
        }

        Ok(messages)
    }

    async fn create_table(&self, username: &str) -> anyhow::Result<()> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'user_{}')",
            username
        );

        let mut conn = self.connect_db().await?;

        let table_exists: bool = sqlx::query_scalar(&query).fetch_one(&mut conn).await?;

        if !table_exists {
            let query = format!(
                "CREATE TABLE IF NOT EXISTS user_{} (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               message TEXT,
               timestamp TEXT
           )",
                username
            );

            sqlx::query(&query).execute(&mut conn).await?;
        }

        Ok(())
    }

    async fn count_messages(&self, username: &str) -> anyhow::Result<i64> {
        let query = format!("SELECT COUNT(*) FROM user_{}", username);
        let mut conn = self.connect_db().await?;
        let count: i64 = sqlx::query_scalar(&query).fetch_one(&mut conn).await?;
        Ok(count)
    }

    async fn overwrite_messages(
        &self,
        message: TelegramMessage,
        username: &str,
    ) -> anyhow::Result<()> {
        let mut conn = self.connect_db().await?;

        let sql = format!("SELECT id FROM user_{} ORDER BY id ASC LIMIT 1", username);
        let oldest_message_id: Option<i64> =
            sqlx::query_scalar(&sql).fetch_optional(&mut conn).await?;

        if let Some(id) = oldest_message_id {
            sqlx::query(&format!("DELETE FROM user_{} WHERE id = ?", username))
                .bind(id)
                .execute(&mut conn)
                .await?;
        }

        self.insert_message(message, username).await?;

        Ok(())
    }

    async fn insert_message(&self, message: TelegramMessage, username: &str) -> anyhow::Result<()> {
        let message_json = serde_json::to_string(&message)?;

        let mut conn = self.connect_db().await?;

        let query = format!("INSERT INTO user_{} (message) VALUES (?)", username);
        sqlx::query(&query)
            .bind(&message_json)
            .execute(&mut conn)
            .await?;
        Ok(())
    }
}
