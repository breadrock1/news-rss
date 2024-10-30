use crate::feeds::rss_feeds::config::RssConfig;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct GetInfoForm {
    source_url: String,
    source_name: String,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct TerminateForm {
    source_url: String,
}

#[derive(Deserialize, Getters, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct CreateWorkerForm {
    #[getset(skip)]
    #[getset(get = "pub")]
    target_url: String,
    max_retries: u32,
    timeout: u64,
    interval_secs: u64,
    create_force: bool,
}

impl From<&CreateWorkerForm> for RssConfig {
    fn from(form: &CreateWorkerForm) -> Self {
        RssConfig::builder()
            .target_url(form.target_url.to_owned())
            .max_retries(form.max_retries)
            .timeout(form.timeout)
            .interval_secs(form.interval_secs)
            .build()
            .unwrap()
    }
}

#[derive(Builder, Serialize)]
pub struct GetInfoResponse {
    source_url: String,
    source_name: String,
    is_launched: bool,
}

impl GetInfoResponse {
    pub fn builder() -> GetInfoResponseBuilder {
        GetInfoResponseBuilder::default()
    }
}
