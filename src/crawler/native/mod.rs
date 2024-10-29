use crate::crawler::CrawlerService;

use regex::Regex;

#[derive(Default)]
pub struct NativeCrawler;

#[async_trait::async_trait]
impl CrawlerService for NativeCrawler {
    type Error = anyhow::Error;

    async fn scrape(&self, text_data: &str) -> Result<String, Self::Error> {
        let regex = Regex::new(r#"<[^>]*>"#)?;
        let result_text = regex.replace_all(text_data, "").to_string();
        Ok(result_text)
    }

    async fn scrape_by_url(&self, url: &str) -> Result<String, Self::Error> {
        let response = reqwest::Client::new()
            .get(url)
            .send()
            .await?
            .error_for_status()?;

        let html_str = response.text().await?;
        self.scrape(&html_str).await
    }
}

#[allow(clippy::default_constructed_unit_structs)]
impl NativeCrawler {
    pub fn new() -> Self {
        Self::default()
    }
}
