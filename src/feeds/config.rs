use crate::feeds::rss_feeds::config::RssConfig;

use getset::Getters;
use serde::Deserialize;

#[derive(Clone, Deserialize, Getters)]
pub struct TopicsConfig {
    rss: RssConfig,
}

impl TopicsConfig {
    pub fn rss(&self) -> RssConfig {
        self.rss.clone()
    }
}
