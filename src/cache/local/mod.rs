pub mod config;

use crate::cache::error::CacheError;
use crate::cache::local::config::LocalCacheConfig;
use crate::cache::CacheService;
use crate::publish::models::PublishNews;
use crate::ServiceConnect;

use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

pub struct LocalCache {
    config: Arc<LocalCacheConfig>,
    client: Cache<String, PublishNews>,
}

impl LocalCache {
    pub fn config(&self) -> Arc<LocalCacheConfig> {
        self.config.clone()
    }
}

#[async_trait::async_trait]
impl ServiceConnect for LocalCache {
    type Config = LocalCacheConfig;
    type Error = CacheError;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let cacher = Cache::builder()
            .time_to_live(Duration::from_secs(config.expired()))
            .build();

        Ok(LocalCache {
            config: Arc::new(config.to_owned()),
            client: cacher,
        })
    }
}

#[async_trait::async_trait]
impl CacheService for LocalCache {
    async fn set(&self, key: &str, value: &PublishNews) {
        let cache = &self.client;
        cache.insert(key.to_string(), value.to_owned()).await;
    }

    async fn contains(&self, key: &str) -> bool {
        let cache = &self.client;
        cache.contains_key(key)
    }
}
