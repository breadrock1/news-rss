#[cfg(feature = "crawler-llm")]
use crate::crawler::llm::config::LlmConfig;

use getset::{CopyGetters, Getters};
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters, CopyGetters)]
pub struct CrawlerConfig {
    #[getset(get = "pub")]
    #[cfg(feature = "crawler-llm")]
    llm: LlmConfig,
}
