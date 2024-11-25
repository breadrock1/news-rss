use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct RabbitConfig {
    address: String,
    username: String,
    password: String,
    exchange: String,
    routing_key: String,
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    no_wait: bool,
    #[getset(skip)]
    #[getset(get_copy = "pub")]
    durable: bool,
}
