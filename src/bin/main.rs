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
use news_rss::server::{RssWorker, ServerApp};
use news_rss::{logger, server, ServiceConnect};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors, trace};
use news_rss::feeds::rss_feeds::config::RssConfig;
use news_rss::storage::LoadTopic;
use news_rss::storage::pgsql::PgsqlTopicStorage;

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

    #[allow(unused_variables)]
    let rss_config = [config.topics().rss()];
    #[cfg(feature = "storage-pgsql")]
    let rss_config = load_topics_from_pgsql(&config).await?;

    let rss_workers = rss_config
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

    let listener = TcpListener::bind(config.server().address()).await?;
    let server_app = ServerApp::new(rss_workers, publish, cache, crawler);
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let cors_layer = cors::CorsLayer::permissive();

    let app = server::init_server(server_app)
        .layer(trace_layer)
        .layer(cors_layer);

    axum::serve(listener, app).await?;

    Ok(())
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

#[cfg(feature = "storage-pgsql")]
pub async fn load_topics_from_pgsql(config: &ServiceConfig) -> Result<Vec<RssConfig>, anyhow::Error> {
    let rss_config = config.topics().rss();

    let pgsql_config = config.storage().pgsql();
    let storage = PgsqlTopicStorage::connect(pgsql_config).await?;
    let mut topics = storage
        .load_at_launch()
        .await?
        .into_iter()
        .map(|it| {
            RssConfig::builder()
                .source_name(it.name.to_owned())
                .target_url(it.link.to_owned())
                .max_retries(rss_config.max_retries())
                .timeout(rss_config.timeout())
                .interval_secs(rss_config.interval_secs())
                .build()
                .unwrap()
        })
        .map(|it| (it.target_url().to_owned(), it))
        .collect::<HashMap<String, RssConfig>>();

    topics.insert(rss_config.target_url().to_owned(), rss_config);
    let topics = topics.into_values().collect();
    Ok(topics)
}
