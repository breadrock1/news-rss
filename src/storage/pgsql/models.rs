use crate::feeds::rss_feeds::config::RssConfig;

use serde::Deserialize;
use sqlx::{Decode, FromRow};

#[derive(FromRow, Deserialize, Decode)]
pub struct PgsqlTopicModel {
    pub id: i32,
    pub name: String,
    pub link: String,
    pub run_at_launch: bool,
    pub max_retries: i32,
    pub timeout: i32,
    pub interval_secs: i32,
}

impl From<PgsqlTopicModel> for RssConfig {
    fn from(value: PgsqlTopicModel) -> Self {
        RssConfig::builder()
            .source_name(value.name.to_owned())
            .target_url(value.link.to_owned())
            .max_retries(value.max_retries.to_owned() as u32)
            .timeout(value.timeout.to_owned() as u64)
            .interval_secs(value.interval_secs.to_owned() as u64)
            .build()
            .unwrap()
    }
}
