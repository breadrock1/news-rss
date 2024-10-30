use derive_builder::Builder;
use getset::{CopyGetters, Getters, Setters};
use serde::Deserialize;

#[derive(Builder, Clone, Deserialize, Getters, CopyGetters, Setters)]
#[getset(get_copy = "pub")]
pub struct RssConfig {
    #[getset(skip)]
    #[getset(get = "pub", set = "pub")]
    target_url: String,
    max_retries: u32,
    timeout: u64,
    #[getset(set = "pub")]
    interval_secs: u64,
}

impl RssConfig {
    pub fn builder() -> RssConfigBuilder {
        RssConfigBuilder::default()
    }
}
