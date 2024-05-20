use crate::common::DatabaseParams;

use anyhow::bail;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::sqlite::SqliteConnection;
use sqlx::{ConnectOptions, Row};
use std::str::FromStr;
use tracing::{error, info};

pub struct LocalDb {
    db_params: DatabaseParams,
}

impl LocalDb {
    pub async fn try_init(db_params: DatabaseParams) -> anyhow::Result<Self> {
        Ok(Self { db_params })
    }
}

impl LocalDb {
    async fn connect_db(&self) -> anyhow::Result<SqliteConnection> {
        let conn = SqliteConnectOptions::from_str(self.db_params.url.as_str())?
            .create_if_missing(true)
            .connect()
            .await?;
        Ok(conn)
    }

    pub async fn save_to_local_db<T>(
        &self,
        message: T,
        table_name: &str,
        need_restriction: bool,
    ) -> anyhow::Result<()>
    where
        T: Serialize + Send + 'static,
    {
        self.create_table(table_name).await?;
        let items_count = self.count_items(table_name).await?;
        if items_count >= 10 && need_restriction {
            self.overwrite_messages(message, table_name).await?;
        } else {
            self.insert_message(message, table_name).await?;
        }
        Ok(())
    }

    pub async fn read_from_local_db<T>(&self, table_name: &str) -> anyhow::Result<Vec<T>>
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
            let message = match serde_json::from_str(&message_json) {
                Ok(res) => res,
                Err(err) => {
                    error!("error {:?}", err);
                    bail!("error");
                }
            };
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
        };
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

    pub async fn get_user_permissions_tg_id(&self, tg_user_id: u64) -> anyhow::Result<Vec<String>> {
        let mut conn = self.connect_db().await?;
        let sql = format!(
            "SELECT DISTINCT role FROM user_roles ur \
                LEFT JOIN user_external_ids uei ON uei.user_id=ur.user_id \
            WHERE external_resource_code='TELEGRAM' AND \
            external_resource_id='{}' \
            AND datetime('now') >= dt_from \
            AND (dt_to is NULL OR datetime('now') <= dt_to)",
            tg_user_id.to_string()
        );
        let roles_result = sqlx::query(&sql).fetch_all(&mut conn).await?;
        let mut result: Vec<String> = Vec::new();

        for row in roles_result {
            result.push(row.get(0));
        }
        Ok(result)
    }

    pub async fn init_db(&self) -> anyhow::Result<()> {
        let mut conn = self.connect_db().await?;

        let user_create_table_query = "CREATE TABLE IF NOT EXISTS user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE,
                info TEXT
            )";
        sqlx::query(&user_create_table_query)
            .execute(&mut conn)
            .await?;

        let user_external_ids_create_table_query = "CREATE TABLE
            IF NOT EXISTS user_external_ids (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id integer NOT NULL,
                external_resource_code TEXT NOT NULL,
                external_resource_id TEXT NOT NULL,
                FOREIGN KEY(user_id) REFERENCES user(id)
            )";
        sqlx::query(&user_external_ids_create_table_query)
            .execute(&mut conn)
            .await?;

        let roles_create_table_query = "CREATE TABLE
            IF NOT EXISTS user_roles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id integer,
                role TEXT NOT NULL,
                dt_from TEXT NOT NULL,
                dt_to TEXT,
                FOREIGN KEY(user_id) REFERENCES user(id)
            )";
        sqlx::query(&roles_create_table_query)
            .execute(&mut conn)
            .await?;

        //fill SUPERADMINS
        struct User {
            tg_id: String,
            username: String,
        }
        // тут список суперадминов, т.е. разработчиков которым будет доступно всё
        let super_admins = [
            User {
                tg_id: "121178660".to_string(),
                username: "ozatot".to_string(),
            },
            User {
                tg_id: "124607629".to_string(),
                username: "llio6oh".to_string(),
            },
            User {
                tg_id: "5964236329".to_string(),
                username: "spacewhaleblues".to_string(),
            },
            User {
                tg_id: "174703869".to_string(),
                username: "bynull".to_string(),
            },
        ];

        for user in super_admins {
            let query = format!(
                "SELECT EXISTS (SELECT 1 FROM user WHERE username = '{}')",
                user.username
            );

            let user_exists: bool = sqlx::query_scalar(&query).fetch_one(&mut conn).await?;
            if !user_exists {
                let create_user_query =
                    format!("INSERT INTO user (username) VALUES ('{}')", user.username);
                sqlx::query(&create_user_query).execute(&mut conn).await?;

                let user_bd_id_query =
                    format!("SELECT id FROM user WHERE username='{}'", user.username);
                let user_bd_id: i64 = sqlx::query_scalar(&user_bd_id_query)
                    .fetch_one(&mut conn)
                    .await?;

                let create_tg_id_query = format!(
                    "INSERT INTO user_external_ids (user_id, external_resource_code, \
                    external_resource_id) VALUES ({}, 'TELEGRAM', '{}')",
                    user_bd_id, user.tg_id
                );
                sqlx::query(&create_tg_id_query).execute(&mut conn).await?;

                let create_user_query = format!(
                    " INSERT INTO user_roles (user_id, role, dt_from) VALUES ({}, 'SUPERADMIN', datetime('now'));", user_bd_id);
                sqlx::query(&create_user_query).execute(&mut conn).await?;
            }
        }

        Ok(())
    }
}
