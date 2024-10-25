use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
pub struct RedisConfig {
    #[getset(get = "pub")]
    address: String,
    #[getset(get = "pub")]
    username: String,
    #[getset(get = "pub")]
    password: String,
    #[getset(get = "pub")]
    stream_name: String,
    #[getset(get_copy = "pub")]
    capacity_gb: u64,
}
