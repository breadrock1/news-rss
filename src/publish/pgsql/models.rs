use crate::publish::models::PublishNews;

use chrono::NaiveDateTime;
use derive_builder::Builder;

#[derive(Builder)]
pub(super) struct PgPublishNewsModel {
    pub id: String,
    pub message_url: String,
    pub datetime: NaiveDateTime,
    pub source: Option<String>,
    pub photo_path: Option<String>,
    pub text: String,
}

impl PgPublishNewsModel {
    pub fn builder() -> PgPublishNewsModelBuilder {
        PgPublishNewsModelBuilder::default()
    }
}

impl From<&PublishNews> for PgPublishNewsModel {
    fn from(value: &PublishNews) -> Self {
        PgPublishNewsModel::builder()
            .id(value.id().to_owned())
            .message_url(value.message_url().to_owned())
            .datetime(value.date().to_owned())
            .source(value.source().to_owned())
            .photo_path(value.photo_path().to_owned())
            .text(value.text().to_owned())
            .build()
            .unwrap()
    }
}
