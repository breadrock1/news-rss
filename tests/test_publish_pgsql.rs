#[cfg(feature = "publish-offline")]
mod test_publish_pgsql {
    use news_rss::cache::local::LocalCache;
    use news_rss::config::ServiceConfig;
    use news_rss::crawler::native::NativeCrawler;
    use news_rss::feeds::rss_feeds::RssFeeds;
    use news_rss::feeds::FetchTopic;
    use news_rss::publish::pgsql::PgsqlPublisher;
    use news_rss::{logger, ServiceConnect};
    use std::sync::Arc;
    use std::time::Duration;

    const TEST_TIME_EXECUTION: u64 = 15;

    #[tokio::test]
    async fn test_publish_pgsql() -> Result<(), anyhow::Error> {
        let config = ServiceConfig::new()?;
        logger::init_logger(config.logger())?;

        let mongo_config = config.publish().pgsql();
        let mongo = PgsqlPublisher::connect(mongo_config).await?;
        let mongo = Arc::new(mongo);

        let cache_config = config.cache().local();
        let cache = LocalCache::connect(cache_config).await?;
        let cache = Arc::new(cache);

        let crawler = NativeCrawler::new();
        let crawler = Arc::new(crawler);

        let topics_config = config.topics();
        let rss_config = topics_config.rss();

        let rss_feeds = rss_config
            .target_url()
            .split(',')
            .filter_map(|it| {
                let mut rss_config = topics_config.rss();
                rss_config.set_target_url(it.to_string());
                RssFeeds::new(&rss_config, mongo.clone(), cache.clone(), crawler.clone()).ok()
            })
            .collect::<Vec<RssFeeds<_, _, _>>>();

        let _ = rss_feeds
            .into_iter()
            .map(|it| tokio::spawn(async move { it.launch_fetching().await }))
            .collect::<Vec<_>>();

        tokio::time::sleep(Duration::from_secs(TEST_TIME_EXECUTION)).await;

        Ok(())
    }
}
