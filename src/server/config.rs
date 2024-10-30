use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct ServerConfig {
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    port: u16,
}
