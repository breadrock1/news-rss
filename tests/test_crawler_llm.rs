#[cfg(feature = "crawler-llm")]
mod test_crawler_llm {
    use news_rss::config::ServiceConfig;
    use news_rss::crawler::llm::LlmCrawler;
    use news_rss::crawler::CrawlerService;
    use news_rss::{logger, ServiceConnect};

    const INPUT_HTML_URL: &str = "https://edition.cnn.com/2024/10/28/media/jeff-bezos-defends-washington-post-endorsement/index.html";
    const INPUT_HTML_DATA: &[u8] = include_bytes!("resources/cnn-news.html");

    #[tokio::test]
    async fn test_llm_crawler() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let crawler = LlmCrawler::connect(config.crawler().llm()).await?;

        let result = crawler.scrape_by_url(INPUT_HTML_URL).await?;

        tracing::info!("There is scraped text: {result}");

        Ok(())
    }

    #[tokio::test]
    async fn test_llm_crawler_from_file() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let crawler = LlmCrawler::connect(config.crawler().llm()).await?;

        let html_text = html2text::from_read(&INPUT_HTML_DATA[..], INPUT_HTML_DATA.len())?;
        let result = crawler.scrape(&html_text).await?;

        tracing::info!("There is scraped text: {result}");

        Ok(())
    }
}
