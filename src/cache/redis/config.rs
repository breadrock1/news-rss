use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct RedisConfig {
    address: String,
    username: String,
    password: String,
    stream_name: String,
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    capacity_gb: u64,
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    expired_secs: u64,
}
