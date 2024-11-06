pub mod config;
#[cfg(feature = "crawler-llm")]
pub mod llm;
pub mod native;

#[async_trait::async_trait]
pub trait CrawlerService {
    type Error: std::fmt::Debug + std::fmt::Display;

    async fn scrape(&self, text_data: &str) -> Result<String, Self::Error>;
    async fn scrape_by_url(&self, url: &str) -> Result<String, Self::Error>;
}
