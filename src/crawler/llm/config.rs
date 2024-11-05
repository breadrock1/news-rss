use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct LlmConfig {
    api_key: String,
    base_url: String,
}
