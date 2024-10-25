#[cfg(feature = "crawler-llm")]
mod test_crawler_llm {
    use html2text::from_read;
    use news_rss::config::ServiceConfig;
    use news_rss::crawler::llm::LlmCrawler;
    use news_rss::crawler::CrawlerService;
    use news_rss::{logger, ServiceConnect};
    use regex::Regex;
    use std::sync::Arc;

    const INPUT_HTML_FILE: &[u8] = include_bytes!("resources/cnn-news.html");

    #[tokio::test]
    async fn test_llm_crawler() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let crawler = LlmCrawler::connect(config.crawler().llm()).await?;
        let crawler = Arc::new(crawler);

        let html_text = from_read(&INPUT_HTML_FILE[..], INPUT_HTML_FILE.len())?;
        let result = crawler.scrape_text(&html_text).await?;

        println!("{result}");

        Ok(())
    }

    #[tokio::test]
    async fn test_scrape_text() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let crawler = LlmCrawler::connect(config.crawler().llm()).await?;
        let response = crawler.scrape_text("<p>Hello world<p>").await?;

        let matched = Regex::new(r#"Hello world"#)?
            .find(&response)
            .map(|it| it.len())
            .unwrap_or(0);

        assert!(matched > 0);

        Ok(())
    }
}
