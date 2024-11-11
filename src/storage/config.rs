#[cfg(feature = "storage-pgsql")]
use crate::storage::pgsql::config::PgsqlTopicStorageConfig;

use getset::Getters;
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct StorageConfig {
    #[cfg(feature = "storage-pgsql")]
    pgsql: PgsqlTopicStorageConfig,
}
