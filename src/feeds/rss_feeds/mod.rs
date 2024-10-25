pub mod config;
mod errors;
mod models;

use crate::cache::CacheService;
use crate::crawler::CrawlerService;
use crate::feeds::rss_feeds::config::RssConfig;
use crate::feeds::rss_feeds::errors::RssError;
use crate::feeds::rss_feeds::models::RssResponse;
use crate::feeds::FetchTopic;
use crate::publish::models::PublishNews;
use crate::publish::Publisher;

use chrono::NaiveDateTime;
use getset::{CopyGetters, Getters};
use reqwest::Url;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[derive(Getters, CopyGetters)]
pub struct RssFeeds<P, C, S>
where
    P: Publisher,
    C: CacheService,
    S: CrawlerService,
{
    #[getset(get = "pub")]
    config: RssConfig,
    #[getset(get_copy = "pub")]
    last_scan: NaiveDateTime,
    #[getset(get = "pub")]
    cacher: Arc<C>,
    #[getset(get = "pub")]
    crawler: Arc<S>,
    #[getset(get = "pub")]
    publisher: Arc<P>,
}

#[async_trait::async_trait]
impl<P, C, S> FetchTopic for RssFeeds<P, C, S>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
{
    type Error = RssError;
    type Response = rss::Channel;

    async fn load_news(&self) -> Result<Self::Response, Self::Error> {
        let max_retries = self.config().max_retries();
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(max_retries);

        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        let timeout = self.config().timeout();
        let target_url = self.config().target_url();
        let response = client
            .get(target_url)
            .timeout(Duration::from_secs(timeout))
            .send()
            .await?;

        let content = response.bytes().await?;
        let channel = rss::Channel::read_from(&content[..])?;
        Ok(channel)
    }

    async fn launch_fetching(&self) -> Result<(), anyhow::Error> {
        let interval_secs = self.config().interval_secs();
        let mut interval = time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            match self.load_news().await {
                Ok(channel) => {
                    let topic = &channel.title().to_string();
                    if let Err(err) = self.processing_event(channel).await {
                        tracing::error!("{topic}: failed while processing rss event: {err:#?}");
                        continue;
                    };
                }
                Err(err) => {
                    tracing::error!("failed to fetch rss channel: {err:#?}");
                    continue;
                }
            }
        }
    }
}

impl<P, C, S> RssFeeds<P, C, S>
where
    P: Publisher + Sync,
    C: CacheService + Sync,
    S: CrawlerService + Sync,
{
    pub fn new(
        config: &RssConfig,
        publish: Arc<P>,
        cache: Arc<C>,
        crawler: Arc<S>,
    ) -> Result<Self, RssError> {
        let last_date_scan: NaiveDateTime = NaiveDateTime::default();
        Ok(RssFeeds {
            config: config.to_owned(),
            last_scan: last_date_scan,
            publisher: publish,
            cacher: cache,
            crawler,
        })
    }

    pub fn get_topic_name(&self) -> String {
        self.config.target_url().clone()
    }

    pub async fn processing_event(&self, channel: rss::Channel) -> Result<(), anyhow::Error> {
        let topic = channel.title();
        tracing::info!("{topic}: received new content from {topic}");

        for item in channel.items() {
            let response = match self.extract_item(item).await {
                Ok(it) => it,
                Err(err) => {
                    tracing::error!("{topic}: failed while converting rss item: {err:#?}");
                    continue;
                }
            };

            let art_id = response.guid();
            if self.cacher().contains(art_id).await {
                tracing::warn!("news article {art_id} has been already parsed");
                continue;
            }

            let art = PublishNews::from(response);
            let art_id = art.id();
            let publish = self.publisher();
            if let Err(err) = publish.publish(&art).await {
                tracing::error!("{topic}: failed to send news article {art_id}: {err:#?}");
                continue;
            }

            tracing::info!("{topic}: article {art_id} published successful");
            self.cacher.set(art_id.to_owned(), art).await;
        }

        Ok(())
    }

    async fn extract_item(&self, item: &rss::Item) -> Result<RssResponse, anyhow::Error> {
        let guid = item.guid().ok_or(anyhow::Error::msg("empty guid"))?;
        let title = item.title().ok_or(anyhow::Error::msg("empty title"))?;
        let link = item.link().ok_or(anyhow::Error::msg("empty link"))?;
        let description = item
            .description()
            .ok_or(anyhow::Error::msg("empty description"))?;

        let content = item
            .content()
            .map(|it| it.to_string())
            .unwrap_or_else(|| description.to_string());

        let content = self
            .crawler()
            .scrape_text(&content)
            .await
            .unwrap_or(content);

        let source = Url::parse(link)
            .map(|it| it.domain().map(|t| t.to_string()))
            .unwrap_or(Some(link.to_string()));

        let pub_date = item
            .pub_date()
            .map(|it| NaiveDateTime::from_str(it).unwrap_or_default())
            .unwrap_or_default();

        let photo_path = match item.itunes_ext() {
            Some(ext) => ext.image().map(|it| it.to_string()),
            None => None,
        };

        let model = RssResponse::builder()
            .guid(guid.value().to_string())
            .title(title.to_string())
            .description(description.to_string())
            .link(link.to_string())
            .photo_path(photo_path)
            .pub_date(pub_date)
            .content(content)
            .source(source)
            .build()?;

        Ok(model)
    }
}
