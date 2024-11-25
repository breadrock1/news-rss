mod mocks;
mod tests_helper;

use mocks::mock_rmq_publish::MockRabbitPublisher;
use news_rss::config::ServiceConfig;
use news_rss::feeds::rss_feeds::config::RssConfig;
use news_rss::feeds::rss_feeds::RssFeeds;
use news_rss::feeds::FetchTopic;
use news_rss::server::RssWorker;
use news_rss::{logger, ServiceConnect};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

const TEST_TIME_EXECUTION: u64 = 5;
const TEST_RMQ_QUEUE_NAME: &str = "news-rss";
const TEST_SOURCE_NAME: &str = "NDTV World News";
const TEST_TARGET_URL: &str = "https://feeds.feedburner.com/ndtvnews-world-news";

#[tokio::test]
async fn test_rss_feeds() -> Result<(), anyhow::Error> {
    let config = ServiceConfig::new()?;
    logger::init_logger(config.logger())?;

    let publish = MockRabbitPublisher::connect(config.publish().rmq()).await?;
    #[allow(unused_variables)]
    let publish = Arc::new(publish);

    #[cfg(feature = "publish-offline")]
    let publish = tests_helper::build_pgsql_publish(&config).await?;

    #[allow(unused_variables)]
    let cache = tests_helper::build_local_cache(&config).await?;
    #[cfg(feature = "cache-redis")]
    let cache = tests_helper::build_redis_cache(&config).await?;

    #[allow(unused_variables)]
    let crawler = tests_helper::build_native_crawler(&config).await?;
    #[cfg(feature = "crawler-llm")]
    let crawler = tests_helper::build_llm_crawler(&config).await?;

    let rss_config = vec![RssConfig::builder()
        .source_name(TEST_SOURCE_NAME.to_owned())
        .target_url(TEST_TARGET_URL.to_owned())
        .max_retries(3)
        .timeout(10)
        .interval_secs(5)
        .build()?];

    let _ = rss_config
        .into_iter()
        .filter_map(|it| RssFeeds::new(it, publish.clone(), cache.clone(), crawler.clone()).ok())
        .map(|it| {
            let config = it.config();

            let url = config.target_url();
            let it_cln = it.clone();
            let worker = tokio::spawn(async move { it_cln.launch_fetching().await });

            let rss_worker = RssWorker::new(Arc::new(config.clone()), worker);
            (url.to_owned(), rss_worker)
        })
        .collect::<HashMap<String, RssWorker>>();

    #[cfg(feature = "test-publish-rabbit")]
    let _ = tests_helper::rabbit_consumer(TEST_RMQ_QUEUE_NAME, config.publish().rmq()).await?;

    tokio::time::sleep(Duration::from_secs(TEST_TIME_EXECUTION)).await;

    Ok(())
}
