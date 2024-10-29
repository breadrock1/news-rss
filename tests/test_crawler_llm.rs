#[cfg(feature = "crawler-llm")]
mod test_crawler_llm {
    use news_rss::config::ServiceConfig;
    use news_rss::crawler::llm::LlmCrawler;
    use news_rss::crawler::CrawlerService;
    use news_rss::{logger, ServiceConnect};

    // const INPUT_HTML_FILE: &[u8] = include_bytes!("resources/ndtv-uk-news.html");
    // const INPUT_HTML_FILE: &[u8] = include_bytes!("resources/cnn-news.html");
    const INPUT_HTML_URL: &str = "https://edition.cnn.com/2024/10/28/media/jeff-bezos-defends-washington-post-endorsement/index.html";
    const XML_LLM_RESPONSE_FILE_STR: &str = include_str!("resources/3279d5de-faa4-4d36-a0d7-bfebd76a7e35");

    #[tokio::test]
    async fn test_llm_crawler() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let crawler = LlmCrawler::connect(config.crawler().llm()).await?;

        // let html_text = from_read(&INPUT_HTML_FILE[..], INPUT_HTML_FILE.len())?;
        // let result = crawler.scrape(&html_text).await?;

        let result = crawler.scrape_by_url(INPUT_HTML_URL).await?;

        tracing::info!("There is scraped text: {result}");

        Ok(())
    }

    #[tokio::test]
    async fn test_extract_art_text() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let result = LlmCrawler::extract_semantic_blocks(XML_LLM_RESPONSE_FILE_STR)?;
        tracing::info!("{result}");
        Ok(())
    }
}
