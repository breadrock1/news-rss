use chrono::NaiveDateTime;
use derive_builder::Builder;
use getset::Getters;
use serde::Serialize;

#[derive(Builder, Clone, Debug, Getters, Serialize)]
#[getset(get = "pub")]
pub struct PublishNews {
    id: String,
    text: String,
    message_url: String,
    datetime: NaiveDateTime,
    source: Option<String>,
    photo_path: Option<String>,
}

impl PublishNews {
    pub fn builder() -> PublishNewsBuilder {
        PublishNewsBuilder::default()
    }
}
