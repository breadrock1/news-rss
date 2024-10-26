mod mocks;
mod tests_helper;

use mocks::mock_rmq_publish::MockRabbitPublisher;
use news_rss::config::ServiceConfig;
use news_rss::feeds::rss_feeds::RssFeeds;
use news_rss::feeds::FetchTopic;
use news_rss::{logger, ServiceConnect};
use std::sync::Arc;
use std::time::Duration;

const TEST_TIME_EXECUTION: u64 = 5;

#[tokio::test]
async fn test_rss_feeds() -> Result<(), anyhow::Error> {
    let config = ServiceConfig::new()?;
    logger::init_logger(config.logger())?;

    let publish = MockRabbitPublisher::connect(config.publish().rmq()).await?;
    let publish = Arc::new(publish);

    #[cfg(feature = "publish-offline")]
    let publish = tests_helper::build_pgsql_publish(&config).await?;

    let cache = tests_helper::build_local_cache(&config).await?;
    #[cfg(feature = "cache-redis")]
    let cache = tests_helper::build_redis_cache(&config).await?;

    let crawler = tests_helper::build_native_crawler(&config).await?;
    #[cfg(feature = "crawler-llm")]
    let crawler = tests_helper::build_llm_crawler(&config).await?;

    let topics_config = config.topics();
    let rss_config = topics_config.rss();

    let rss_feeds = rss_config
        .target_url()
        .split(',')
        .filter_map(|it| {
            let mut rss_config = topics_config.rss();
            rss_config.set_target_url(it.to_string());
            RssFeeds::new(&rss_config, publish.clone(), cache.clone(), crawler.clone()).ok()
        })
        .collect::<Vec<RssFeeds<_, _, _>>>();

    let _ = rss_feeds
        .into_iter()
        .map(|it| tokio::spawn(async move { it.launch_fetching().await }))
        .collect::<Vec<_>>();

    // let _ = tests_helper::rabbit_consumer(config.publish().rmq()).await?;

    tokio::time::sleep(Duration::from_secs(TEST_TIME_EXECUTION)).await;

    Ok(())
}
