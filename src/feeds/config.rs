use crate::feeds::rss_feeds::config::RssConfig;

use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct TopicsConfig {
    rss: RssConfig,
}

impl TopicsConfig {
    pub fn rss(&self) -> RssConfig {
        self.rss.clone()
    }
}
