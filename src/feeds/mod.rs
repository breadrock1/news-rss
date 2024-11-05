pub mod config;
pub mod rss_feeds;

#[async_trait::async_trait]
pub trait FetchTopic {
    type Error;
    type Response;

    async fn load_news(&self) -> Result<Self::Response, Self::Error>;
    async fn launch_fetching(&self) -> Result<(), anyhow::Error>;
}
