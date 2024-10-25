use crate::publish::models::PublishNews;
use chrono::NaiveDateTime;
use derive_builder::Builder;
use getset::Getters;
use serde::Deserialize;

#[derive(Builder, Deserialize, Getters)]
#[serde(rename_all = "kebab-case")]
pub struct RssResponse {
    #[getset(get = "pub")]
    guid: String,
    #[getset(get = "pub")]
    title: String,
    #[getset(get = "pub")]
    link: String,
    #[getset(get = "pub")]
    content: String,
    #[getset(get = "pub")]
    description: String,
    #[getset(get = "pub")]
    #[serde(alias = "pubDate")]
    pub_date: NaiveDateTime,
    #[getset(get = "pub")]
    source: Option<String>,
    #[getset(get = "pub")]
    photo_path: Option<String>,
}

impl RssResponse {
    pub fn builder() -> RssResponseBuilder {
        RssResponseBuilder::default()
    }
}

impl From<RssResponse> for PublishNews {
    fn from(response: RssResponse) -> Self {
        PublishNews::builder()
            .id(response.guid().to_owned())
            .source(response.source().to_owned())
            .datetime(response.pub_date().to_owned())
            .photo_path(response.photo_path().to_owned())
            .text(response.content().to_owned())
            .message_url(response.link().to_owned())
            .build()
            .unwrap()
    }
}
