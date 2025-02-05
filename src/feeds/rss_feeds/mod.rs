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

use chrono::Utc;
use getset::{CopyGetters, Getters};
use regex::Regex;
use reqwest::Url;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[derive(Clone, Getters, CopyGetters)]
#[getset(get = "pub")]
pub struct RssFeeds<P, C, S>
where
    P: Publisher,
    C: CacheService,
    S: CrawlerService,
{
    config: RssConfig,
    cacher: Arc<C>,
    crawler: Arc<S>,
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
                        tracing::error!(err=?err, topic=topic, "failed while processing rss event");
                        continue;
                    };
                }
                Err(err) => {
                    tracing::error!(err=?err, "failed to fetch rss channel");
                    return Err(err.into());
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
        config: RssConfig,
        publish: Arc<P>,
        cache: Arc<C>,
        crawler: Arc<S>,
    ) -> Result<Self, RssError> {
        Ok(RssFeeds {
            config: config.to_owned(),
            publisher: publish,
            cacher: cache,
            crawler,
        })
    }

    pub async fn processing_event(&self, channel: rss::Channel) -> Result<(), anyhow::Error> {
        let topic = channel.title();
        tracing::info!(topic = topic, "received new rss content");

        for item in channel.items() {
            let response = match self.extract_item(item).await {
                Ok(it) => it,
                Err(err) => {
                    tracing::error!(topic=topic, err=?err, "failed while converting rss item");
                    continue;
                }
            };

            let art_id = response.guid();
            if self.cacher().contains(art_id).await {
                tracing::warn!(
                    topic = topic,
                    article = art_id,
                    "news article has been already parsed"
                );
                continue;
            }

            let art = PublishNews::from(response);
            let art_id = art.id();
            let publish = self.publisher();
            if let Err(err) = publish.publish(&art).await {
                tracing::error!(topic=topic, article=art_id, err=?err, "failed to send article");
                continue;
            }

            tracing::info!(
                topic = topic,
                article = art_id,
                "article has been published successful"
            );
            self.cacher.set(art_id, &art).await;
        }

        Ok(())
    }

    async fn extract_item(&self, item: &rss::Item) -> Result<RssResponse, anyhow::Error> {
        let guid = item.guid().ok_or(anyhow::Error::msg("empty guid"))?;
        let title = item.title().ok_or(anyhow::Error::msg("empty title"))?;
        let link = item.link().unwrap_or(guid.value());

        let source = Url::parse(link)
            .map(|it| it.domain().map(|t| t.to_string()))
            .unwrap_or(Some(link.to_string()));

        let description = item
            .description()
            .ok_or(anyhow::Error::msg("empty description"))?;

        let content = match item.content() {
            Some(data) => self.clear_html_tags(data)?,
            None => {
                #[allow(unused_variables)]
                let data = description.to_string();

                #[cfg(feature = "crawler-llm")]
                let data = self.scrape(link).await?;

                data
            }
        };

        let pub_date = item
            .pub_date()
            .map(|it| match dateparser::DateTimeUtc::from_str(it) {
                Ok(time) => time.0.naive_utc(),
                Err(err) => {
                    tracing::warn!(time=it, err=?err, "failed to extract datetime");
                    Utc::now().naive_utc()
                }
            })
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

    fn clear_html_tags(&self, content: &str) -> Result<String, regex::Error> {
        let regex = Regex::new(r#"<[^>]*>"#)?;
        let result_text = regex.replace_all(content, "").to_string();
        Ok(result_text)
    }

    #[cfg(feature = "crawler-llm")]
    async fn scrape(&self, link: &str) -> Result<String, anyhow::Error> {
        let result = self
            .crawler()
            .scrape_by_url(link)
            .await
            .map_err(|err| anyhow::Error::msg(err.to_string()))?;

        Ok(result)
    }
}
