use anyhow::Error;
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
async fn main() -> Result<(), Error> {
    let config = ServiceConfig::new()?;
    logger::init_logger(config.logger())?;

    let rmq_config = config.publish().rmq();
    let rmq = RabbitPublisher::connect(rmq_config).await?;
    let rmq = Arc::new(rmq);

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
            RssFeeds::new(&rss_config, rmq.clone(), cache.clone(), crawler.clone()).ok()
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

fn extract_worker_result(result: Result<Result<(), Error>, JoinError>) {
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
