pub mod config;
#[cfg(feature = "crawler-llm")]
pub mod llm;
pub mod native;

#[async_trait::async_trait]
pub trait CrawlerService {
    type Error;

    async fn scrape_text(&self, text_data: &str) -> Result<String, Self::Error>;
}
