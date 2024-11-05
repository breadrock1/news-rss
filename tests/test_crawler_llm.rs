mod tests_helper;

#[cfg(feature = "crawler-llm")]
mod test_crawler_llm {
    use crate::tests_helper;

    use news_rss::config::ServiceConfig;
    use news_rss::crawler::llm::config::LlmConfig;
    use news_rss::crawler::llm::LlmCrawler;
    use news_rss::crawler::CrawlerService;
    use news_rss::{logger, ServiceConnect};

    const ASSERT_CRAWLER_DATA: &str = include_str!("resources/llm-chat-response-assert.txt");

    #[tokio::test]
    async fn test_llm_crawler() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let mock = tests_helper::build_mock_server().await;
        let llm_address = format!("http://{}/v1", mock.address());
        tracing::info!("llm address is {}", &llm_address);
        let llm_config = LlmConfig::builder()
            .api_key("sk-no-key-required".to_string())
            .base_url(llm_address)
            .build()?;

        let crawler = LlmCrawler::connect(&llm_config).await?;
        let result = crawler.scrape("").await?;

        tracing::info!("there is scraped text: {result}");
        assert_eq!(ASSERT_CRAWLER_DATA, result);

        Ok(())
    }
}
