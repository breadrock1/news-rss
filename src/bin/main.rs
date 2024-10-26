#[cfg(feature = "cache-redis")]
use news_rss::cache::redis::RedisClient;

#[cfg(feature = "crawler-llm")]
use news_rss::crawler::llm::LlmCrawler;

#[cfg(feature = "publish-offline")]
use news_rss::publish::pgsql::PgsqlPublisher;

use news_rss::cache::local::LocalCache;
use news_rss::config::ServiceConfig;
use news_rss::crawler::native::NativeCrawler;
use news_rss::feeds::rss_feeds::RssFeeds;
use news_rss::feeds::FetchTopic;
use news_rss::publish::rabbit::RabbitPublisher;
use news_rss::{logger, ServiceConnect};
use std::sync::Arc;
use tokio::task::JoinError;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let config = ServiceConfig::new()?;
    logger::init_logger(config.logger())?;

    #[allow(unused_variables)]
    let publish = build_rmq_publish(&config).await?;
    #[cfg(feature = "publish-offline")]
    let publish = build_pgsql_publish(&config).await?;

    #[allow(unused_variables)]
    let cache = build_local_cache(&config).await?;
    #[cfg(feature = "cache-redis")]
    let cache = build_redis_cache(&config).await?;

    #[allow(unused_variables)]
    let crawler = build_native_crawler(&config).await?;
    #[cfg(feature = "crawler-llm")]
    let crawler = build_llm_crawler(&config).await?;

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

    let rss_workers = rss_feeds
        .into_iter()
        .map(|it| tokio::spawn(async move { it.launch_fetching().await }))
        .collect::<Vec<_>>();

    for worker in rss_workers {
        let res = worker.await;
        extract_worker_result(res);
    }

    Ok(())
}

fn extract_worker_result(result: Result<Result<(), anyhow::Error>, JoinError>) {
    let join_result = match result {
        Ok(res) => res,
        Err(err) => {
            tracing::error!("failed to joint worker: {err:#?}");
            return;
        }
    };

    if let Err(err) = join_result {
        tracing::error!("internal worker error: {err:#?}");
        return;
    }

    tracing::info!("worker has been terminated successful");
}

pub async fn build_local_cache(config: &ServiceConfig) -> Result<Arc<LocalCache>, anyhow::Error> {
    let cache_config = config.cache().local();
    let cache = LocalCache::connect(cache_config).await?;
    let cache = Arc::new(cache);
    Ok(cache)
}

#[cfg(feature = "cache-redis")]
pub async fn build_redis_cache(config: &ServiceConfig) -> Result<Arc<RedisClient>, anyhow::Error> {
    let redis_config = config.cache().redis();
    let cache = RedisClient::connect(redis_config).await?;
    let cache = Arc::new(cache);
    Ok(cache)
}

pub async fn build_rmq_publish(
    config: &ServiceConfig,
) -> Result<Arc<RabbitPublisher>, anyhow::Error> {
    let rmq_config = config.publish().rmq();
    let rmq = RabbitPublisher::connect(rmq_config).await?;
    let rmq = Arc::new(rmq);
    Ok(rmq)
}

#[cfg(feature = "publish-offline")]
pub async fn build_pgsql_publish(
    config: &ServiceConfig,
) -> Result<Arc<PgsqlPublisher>, anyhow::Error> {
    let pgsql_config = config.publish().pgsql();
    let pgsql = PgsqlPublisher::connect(pgsql_config).await?;
    let pgsql = Arc::new(pgsql);
    Ok(pgsql)
}

pub async fn build_native_crawler(
    _config: &ServiceConfig,
) -> Result<Arc<NativeCrawler>, anyhow::Error> {
    let crawler = NativeCrawler::new();
    let crawler = Arc::new(crawler);
    Ok(crawler)
}

#[cfg(feature = "crawler-llm")]
pub async fn build_llm_crawler(config: &ServiceConfig) -> Result<Arc<LlmCrawler>, anyhow::Error> {
    let crawler_config = config.crawler().llm();
    let crawler = LlmCrawler::connect(crawler_config).await?;
    let crawler = Arc::new(crawler);
    Ok(crawler)
}
