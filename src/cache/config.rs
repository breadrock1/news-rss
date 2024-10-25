use crate::cache::local::config::LocalCacheConfig;
#[cfg(feature = "cache-redis")]
use crate::cache::redis::config::RedisConfig;

use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, CopyGetters, Getters)]
pub struct CacheConfig {
    #[getset(get = "pub")]
    local: LocalCacheConfig,

    #[getset(get = "pub")]
    #[cfg(feature = "cache-redis")]
    redis: RedisConfig,
}
