use crate::feeds::rss_feeds::config::RssConfig;
use crate::server::swagger::SwaggerExamples;
use crate::storage::pgsql::models::PgsqlTopicModel;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

const EXAMPLE_SOURCE_NAME: &str = "BBC";
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
#[getset(get = "pub")]
pub struct DeleteWorkerForm {
    #[schema(example = "https://bbc-news.com/rss.xml")]
    target_url: String,

    #[getset(skip)]
    #[getset(get_copy = "pub")]
    #[schema(example = false)]
    is_force: bool,
}

impl SwaggerExamples for DeleteWorkerForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        DeleteWorkerForm {
            target_url: EXAMPLE_TARGET_URL.to_string(),
            is_force: false,
        }
    }
}

#[derive(Deserialize, Serialize, Getters, CopyGetters, IntoParams, ToSchema)]
#[getset(get = "pub")]
pub struct CreateWorkerForm {
    #[schema(example = "BBC")]
    source_name: String,

    #[schema(example = "https://bbc-news.com/rss.xml")]
    target_url: String,

    #[schema(example = 3)]
    config: RssConfigForm,

    #[getset(skip)]
    #[getset(get_copy = "pub")]
    #[schema(example = false)]
    create_force: bool,
}

impl CreateWorkerForm {
    pub fn to_rss_config(&self) -> RssConfig {
        RssConfig::builder()
            .source_name(self.source_name.to_owned())
            .target_url(self.target_url.to_owned())
            .max_retries(self.config.max_retries)
            .timeout(self.config.timeout)
            .interval_secs(self.config.interval_secs)
            .build()
            .unwrap()
    }
}

impl SwaggerExamples for RssConfigForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        RssConfigForm {
            max_retries: 3,
            timeout: 300,
            interval_secs: 300,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Getters, CopyGetters, IntoParams, ToSchema)]
#[getset(get_copy = "pub")]
pub struct RssConfigForm {
    #[schema(example = 3)]
    max_retries: u32,

    #[schema(example = 100)]
    timeout: u64,

    #[schema(example = 300)]
    interval_secs: u64,
}

impl From<&RssConfig> for RssConfigForm {
    fn from(value: &RssConfig) -> Self {
        RssConfigForm {
            max_retries: value.max_retries(),
            timeout: value.timeout(),
            interval_secs: value.interval_secs(),
        }
    }
}

impl SwaggerExamples for CreateWorkerForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        CreateWorkerForm {
            source_name: EXAMPLE_SOURCE_NAME.to_string(),
            target_url: EXAMPLE_TARGET_URL.to_string(),
            config: RssConfigForm::example(None),
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

    #[schema(example = RssConfigForm)]
    #[serde(skip_serializing_if = "Option::is_none")]
    configuration: Option<RssConfigForm>,
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
            configuration: Some(RssConfigForm::example(None)),
        }
    }
}

#[derive(Builder, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct GetSourcesResponse {
    id: i32,
    #[schema(example = "BBC")]
    name: String,
    #[schema(example = "https://bbc-news.com/rss.xml")]
    link: String,
    #[schema(example = false)]
    run_at_launch: bool,
    #[schema(example = 3)]
    max_retries: i32,
    #[schema(example = 100)]
    timeout: i32,
    #[schema(example = 3600)]
    interval_secs: i32,
}

impl From<PgsqlTopicModel> for GetSourcesResponse {
    fn from(value: PgsqlTopicModel) -> Self {
        GetSourcesResponseBuilder::default()
            .id(value.id)
            .name(value.name.to_owned())
            .link(value.link.to_owned())
            .run_at_launch(value.run_at_launch)
            .max_retries(value.max_retries)
            .timeout(value.timeout)
            .interval_secs(value.interval_secs)
            .build()
            .unwrap()
    }
}

impl SwaggerExamples for GetSourcesResponse {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        GetSourcesResponseBuilder::default()
            .id(1)
            .name("BBC".to_owned())
            .link("https://bbc-news.com/rss.xml".to_owned())
            .run_at_launch(true)
            .max_retries(3)
            .timeout(100)
            .interval_secs(3600)
            .build()
            .unwrap()
    }
}

#[derive(Builder, Deserialize, Serialize, IntoParams, ToSchema)]
pub struct CreateSourceForm {
    #[schema(example = "BBC")]
    name: String,
    #[schema(example = "https://bbc-news.com/rss.xml")]
    link: String,
    #[schema(example = false)]
    run_at_launch: bool,
    #[schema(example = 3)]
    max_retries: i32,
    #[schema(example = 100)]
    timeout: i32,
    #[schema(example = 3600)]
    interval_secs: i32,
}

impl From<CreateSourceForm> for PgsqlTopicModel {
    fn from(value: CreateSourceForm) -> Self {
        PgsqlTopicModel::builder()
            .id(0)
            .name(value.name.to_owned())
            .link(value.link.to_owned())
            .run_at_launch(value.run_at_launch)
            .max_retries(value.max_retries)
            .timeout(value.timeout)
            .interval_secs(value.interval_secs)
            .build()
            .unwrap()
    }
}

impl SwaggerExamples for CreateSourceForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        CreateSourceFormBuilder::default()
            .name("BBC".to_owned())
            .link("https://bbc-news.com/rss.xml".to_owned())
            .run_at_launch(true)
            .max_retries(3)
            .timeout(100)
            .interval_secs(3600)
            .build()
            .unwrap()
    }
}

#[derive(Getters, Deserialize, Serialize, IntoParams, ToSchema)]
#[getset(get = "pub")]
pub struct SearchSourcesForm {
    #[schema(example = "World")]
    query: String,
}

impl SwaggerExamples for SearchSourcesForm {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        SearchSourcesForm {
            query: "World".to_string(),
        }
    }
}
