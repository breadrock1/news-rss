pub mod config;
pub mod models;

use crate::storage::pgsql::config::PgsqlTopicStorageConfig;
use crate::storage::pgsql::models::PgsqlTopicModel;
use crate::storage::LoadTopic;
use crate::ServiceConnect;

use getset::Getters;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

#[derive(Clone, Getters)]
pub struct PgsqlTopicStorage {
    pool: Arc<Pool<Postgres>>,
}

#[async_trait::async_trait]
impl ServiceConnect for PgsqlTopicStorage {
    type Config = PgsqlTopicStorageConfig;
    type Error = sqlx::Error;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let db = config.database();
        let user = config.username();
        let passwd = config.password();
        let address = config.address();

        let url = format!("postgresql://{user}:{passwd}@{address}/{db}");
        tracing::info!("connecting to `{}` database", &url);
        let connection = PgPoolOptions::default()
            .max_connections(config.max_pool_size())
            .connect(&url)
            .await?;

        Ok(PgsqlTopicStorage {
            pool: Arc::new(connection),
        })
    }
}

#[async_trait::async_trait]
impl LoadTopic for PgsqlTopicStorage {
    type Error = sqlx::Error;
    type Topic = PgsqlTopicModel;

    async fn load_all(&self) -> Result<Vec<Self::Topic>, Self::Error> {
        let connection = self.pool.as_ref();
        let models = sqlx::query_as!(
            PgsqlTopicModel,
            r#"
                SELECT * FROM rss_sources
            "#
        )
        .fetch_all(connection)
        .await?;

        Ok(models)
    }

    async fn load_at_launch(&self) -> Result<Vec<Self::Topic>, Self::Error> {
        let connection = self.pool.as_ref();
        let models = sqlx::query_as!(
            PgsqlTopicModel,
            r#"
                SELECT * FROM rss_sources
                WHERE run_at_launch = $1
            "#,
            true
        )
        .fetch_all(connection)
        .await?;

        Ok(models)
    }
}
