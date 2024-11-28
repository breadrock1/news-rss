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
        tracing::info!(url=url, "connecting to database");
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
    type TopicId = i32;

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

    async fn search_source(&self, query: &str) -> Result<Vec<Self::Topic>, Self::Error> {
        let connection = self.pool.as_ref();
        let sql_query = format!(
            r#"
                SELECT * FROM rss_sources
                WHERE name LIKE '%{}%' OR link LIKE '%{}%'
            "#,
            &query, &query,
        );
        let models = sqlx::query_as(&sql_query).fetch_all(connection).await?;

        Ok(models)
    }

    async fn add_source(&self, topic: &Self::Topic) -> Result<(), Self::Error> {
        let connection = self.pool.as_ref();
        let _ = sqlx::query!(
            r#"
                INSERT INTO rss_sources (name, link, run_at_launch)
                VALUES ($1, $2, $3)
            "#,
            topic.name,
            topic.link,
            topic.run_at_launch,
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn remove_source(&self, id: Self::TopicId) -> Result<(), Self::Error> {
        let connection = self.pool.as_ref();
        let _ = sqlx::query!(
            r#"
                DELETE FROM rss_sources
                WHERE id = $1
            "#,
            id,
        )
        .execute(connection)
        .await?;

        Ok(())
    }

    async fn update_source(&self, topic: &Self::Topic) -> Result<(), Self::Error> {
        let connection = self.pool.as_ref();
        let _ = sqlx::query!(
            r#"
                UPDATE rss_sources
                SET name = $2,
                    link = $3,
                    run_at_launch = $4,
                    max_retries = $5,
                    timeout = $6,
                    interval_secs = $7
                WHERE id = $1
            "#,
            topic.id,
            topic.name,
            topic.link,
            topic.run_at_launch,
            topic.max_retries,
            topic.timeout,
            topic.interval_secs,
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
