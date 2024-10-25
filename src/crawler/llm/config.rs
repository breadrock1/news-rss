use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
pub struct LlmConfig {
    #[getset(get = "pub")]
    api_key: String,
    #[getset(get = "pub")]
    base_url: String,
}
