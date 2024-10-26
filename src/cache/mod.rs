use crate::publish::models::PublishNews;

pub mod config;
mod error;
pub mod local;

#[cfg(feature = "cache-redis")]
pub mod redis;

#[async_trait::async_trait]
pub trait CacheService {
    async fn set(&self, key: &str, value: &PublishNews);
    async fn contains(&self, key: &str) -> bool;
}
