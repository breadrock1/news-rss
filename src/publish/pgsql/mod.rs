pub mod config;
mod models;

use crate::publish::models::PublishNews;
use crate::publish::pgsql::config::PgsqlConfig;
use crate::publish::pgsql::models::PgPublishNewsModel;
use crate::publish::Publisher;
use crate::ServiceConnect;

use getset::Getters;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, Pool, Postgres};
use std::sync::Arc;

#[derive(Clone, Getters)]
pub struct PgsqlPublisher {
    pool: Arc<Pool<Postgres>>,
}

#[async_trait::async_trait]
impl ServiceConnect for PgsqlPublisher {
    type Config = PgsqlConfig;
    type Error = Error;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let db = config.database();
        let user = config.username();
        let passwd = config.password();
        let address = config.address();

        let url = format!("postgresql://{user}:{passwd}@{address}/{db}");
        tracing::info!(db_url=url, "connecting to database");
        let connection = PgPoolOptions::default()
            .max_connections(config.max_pool_size())
            .connect(&url)
            .await?;

        Ok(PgsqlPublisher {
            pool: Arc::new(connection),
        })
    }
}

#[async_trait::async_trait]
impl Publisher for PgsqlPublisher {
    type Error = Error;

    async fn publish(&self, msg_body: &PublishNews) -> Result<(), Self::Error> {
        let connection = self.pool.as_ref();
        let model = PgPublishNewsModel::from(msg_body);
        sqlx::query!(
            r#"
                INSERT INTO news(
                    id,
                    message_url,
                    datetime,
                    source,
                    photo_path,
                    text
                )
                VALUES ( $1, $2, $3, $4, $5, $6 )
            "#,
            model.id,
            model.message_url,
            model.datetime,
            model.source,
            model.photo_path,
            model.text
        )
        .execute(connection)
        .await?;

        Ok(())
    }
}
