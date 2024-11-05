use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Builder, Deserialize, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct LlmConfig {
    api_key: String,
    base_url: String,
}

impl LlmConfig {
    pub fn builder() -> LlmConfigBuilder {
        LlmConfigBuilder::default()
    }
}
