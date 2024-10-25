use crate::crawler::CrawlerService;

use regex::Regex;

#[derive(Default)]
pub struct NativeCrawler;

#[async_trait::async_trait]
impl CrawlerService for NativeCrawler {
    type Error = regex::Error;

    async fn scrape_text(&self, text_data: &str) -> Result<String, Self::Error> {
        let regex = Regex::new(r#"<[^>]*>"#)?;
        let result_text = regex.replace_all(text_data, "").to_string();
        Ok(result_text)
    }
}

#[allow(clippy::default_constructed_unit_structs)]
impl NativeCrawler {
    pub fn new() -> Self {
        Self::default()
    }
}
