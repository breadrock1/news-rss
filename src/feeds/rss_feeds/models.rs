use crate::publish::models::PublishNews;

use chrono::NaiveDateTime;
use derive_builder::Builder;
use getset::Getters;
use serde::Deserialize;

#[derive(Builder, Deserialize, Getters)]
#[serde(rename_all = "kebab-case")]
#[getset(get = "pub")]
pub struct RssResponse {
    guid: String,
    title: String,
    link: String,
    content: String,
    description: String,
    #[serde(alias = "pubDate")]
    pub_date: NaiveDateTime,
    source: Option<String>,
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
