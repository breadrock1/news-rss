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
use news_rss::server::ServerApp;
use news_rss::{logger, server, ServiceConnect};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace;

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

    // TODO: Implement loading workers from db
    // let topics_config = config.topics();
    // let rss_config = topics_config.rss();
    //
    // let rss_feeds = rss_config
    //     .target_url()
    //     .split(',')
    //     .filter_map(|it| {
    //         let mut rss_config = topics_config.rss();
    //         rss_config.set_target_url(it.to_string());
    //         RssFeeds::new(&rss_config, publish.clone(), cache.clone(), crawler.clone()).ok()
    //     })
    //     .map(|it| (it.get_source(), it))
    //     .collect::<HashMap<String, RssFeeds<_, _, _>>>();

    let rss_feeds: HashMap<String, RssFeeds<RabbitPublisher, LocalCache, NativeCrawler>> =
        HashMap::default();

    let rss_workers = rss_feeds
        .into_iter()
        .map(|it| {
            let source = it.0;
            let worker = it.1;
            let task = tokio::spawn(async move { worker.launch_fetching().await });
            (source, task)
        })
        .collect::<HashMap<String, _>>();

    let listener = TcpListener::bind(config.server().address()).await?;
    let server_app = ServerApp::new(rss_workers, publish.clone(), cache.clone(), crawler.clone());

    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let app = server::init_server(server_app).layer(trace_layer);

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
