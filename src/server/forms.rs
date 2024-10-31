use crate::feeds::rss_feeds::config::RssConfig;
use crate::server::swagger::SwaggerExamples;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

const EXAMPLE_TARGET_URL: &str = "https://bbc-news.com/rss.xml";

#[derive(Deserialize, Serialize, Getters, IntoParams, ToSchema)]
#[getset(get = "pub")]
pub struct GetInfoForm {
    #[schema(example = "https://bbc-news.com/rss.xml")]
    source_url: String,
    #[schema(example = "BBC")]
    source_name: String,
}

impl SwaggerExamples for GetInfoForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        GetInfoForm {
            source_url: EXAMPLE_TARGET_URL.to_string(),
            source_name: "BBC".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Getters, IntoParams, ToSchema)]
#[getset(get = "pub")]
pub struct TerminateWorkerForm {
    #[schema(example = "https://bbc-news.com/rss.xml")]
    source_url: String,
}

impl SwaggerExamples for TerminateWorkerForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        TerminateWorkerForm {
            source_url: EXAMPLE_TARGET_URL.to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Getters, CopyGetters, IntoParams, ToSchema)]
#[getset(get_copy = "pub")]
pub struct CreateWorkerForm {
    #[getset(skip)]
    #[getset(get = "pub")]
    #[schema(example = "https://bbc-news.com/rss.xml")]
    target_url: String,

    #[schema(example = 3)]
    max_retries: u32,

    #[schema(example = 100)]
    timeout: u64,

    #[schema(example = 300)]
    interval_secs: u64,

    #[schema(example = false)]
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

impl SwaggerExamples for CreateWorkerForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        CreateWorkerForm {
            target_url: EXAMPLE_TARGET_URL.to_string(),
            max_retries: 3,
            timeout: 300,
            interval_secs: 300,
            create_force: true,
        }
    }
}

#[derive(Builder, Serialize, IntoParams, ToSchema)]
pub struct GetInfoResponse {
    #[schema(example = "https://bbc-news.com/rss.xml")]
    source_url: String,

    #[schema(example = "BBC")]
    source_name: String,

    #[schema(example = false)]
    is_launched: bool,
}

impl GetInfoResponse {
    pub fn builder() -> GetInfoResponseBuilder {
        GetInfoResponseBuilder::default()
    }
}

impl SwaggerExamples for GetInfoResponse {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        GetInfoResponse {
            source_url: "https://bbc-news.com/rss.xml".to_string(),
            source_name: "BBC".to_string(),
            is_launched: false,
        }
    }
}
