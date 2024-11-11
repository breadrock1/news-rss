use crate::storage::pgsql::config::PgsqlTopicStorageConfig;

use getset::Getters;
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct StorageConfig {
    pgsql: PgsqlTopicStorageConfig
}
