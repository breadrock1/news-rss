use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct PgsqlConfig {
    address: String,
    database: String,
    username: String,
    password: String,
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    max_pool_size: u32,
}
