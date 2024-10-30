use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct ServerConfig {
    address: String,
}
