pub mod config;
mod models;

use crate::cache::redis::config::RedisConfig;
use crate::cache::CacheService;
use crate::publish::models::PublishNews;
use crate::ServiceConnect;

use getset::CopyGetters;
use redis::{AsyncCommands, Client, RedisError, RedisResult};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, CopyGetters)]
pub struct RedisClient {
    options: Arc<RedisConfig>,
    client: Arc<RwLock<Client>>,
}

#[async_trait::async_trait]
impl ServiceConnect for RedisClient {
    type Config = RedisConfig;
    type Error = RedisError;
    type Client = RedisClient;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let address = config.address();
        let client = Client::open(address.clone())?;
        Ok(RedisClient {
            options: Arc::new(config.to_owned()),
            client: Arc::new(RwLock::new(client)),
        })
    }
}

#[async_trait::async_trait]
impl CacheService for RedisClient {
    async fn set(&self, key: &str, value: &PublishNews) {
        let expired_secs: u64 = self.options.expired_secs();
        let cxt = self.client.write().await;
        match cxt.get_multiplexed_tokio_connection().await {
            Err(err) => {
                tracing::warn!("cache: failed to get redis connection {err:#?}");
                return;
            }
            Ok(mut conn) => {
                let store_result: RedisResult<()> = conn.set_ex(key, value, expired_secs).await;
                if let Err(err) = store_result {
                    tracing::warn!("cache: failed to store value to redis: {err:#?}");
                    return;
                }
            }
        }
    }

    async fn contains(&self, key: &str) -> bool {
        let cxt = self.client.read().await;
        match cxt.get_multiplexed_tokio_connection().await {
            Ok(mut conn) => conn.get::<&str, PublishNews>(key).await.ok().is_some(),
            Err(err) => {
                tracing::warn!("failed to get redis service connection {err:#?}");
                false
            }
        }
    }
}
