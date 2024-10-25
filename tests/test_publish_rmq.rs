use lapin::message::DeliveryResult;
use lapin::options::ExchangeDeclareOptions;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::options::{QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties, ExchangeKind};
use news_rss::cache::local::LocalCache;
use news_rss::config::ServiceConfig;
use news_rss::crawler::native::NativeCrawler;
use news_rss::feeds::rss_feeds::RssFeeds;
use news_rss::feeds::FetchTopic;
use news_rss::publish::rabbit::config::RabbitConfig;
use news_rss::publish::rabbit::RabbitPublisher;
use news_rss::{logger, ServiceConnect};
use std::sync::Arc;
use std::time::Duration;

const TEST_AMQP_CONSUMER_TAG: &str = "test-news-rss-consumer";
const TEST_TIME_EXECUTION: u64 = 5;

#[tokio::test]
async fn test_rss_feeds() -> Result<(), anyhow::Error> {
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
        let _ = worker.await;
    }

    let _ = rabbit_consumer(config.publish().rmq()).await?;

    tokio::time::sleep(Duration::from_secs(TEST_TIME_EXECUTION)).await;

    Ok(())
}

async fn rabbit_consumer(config: &RabbitConfig) -> Result<(), anyhow::Error> {
    let conn_props = ConnectionProperties::default();
    let connection = Connection::connect(config.address(), conn_props).await?;
    let channel = connection.create_channel().await?;

    let exchange_opts = ExchangeDeclareOptions {
        nowait: true,
        ..Default::default()
    };
    channel
        .exchange_declare(
            config.exchange(),
            ExchangeKind::Direct,
            exchange_opts,
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_declare(
            config.stream_name(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

    channel
        .queue_bind(
            config.stream_name(),
            config.exchange(),
            config.routing_key(),
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;

    let consumer = channel
        .basic_consume(
            config.stream_name(),
            TEST_AMQP_CONSUMER_TAG,
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    let mut counter = 0;
    consumer.set_delegate(move |delivery: DeliveryResult| async move {
        let delivery = match delivery {
            Ok(Some(delivery)) => {
                tracing::info!("delivered {delivery:?}");
                delivery
            }
            Ok(None) => return,
            Err(err) => {
                tracing::error!("failed to consume queue message {err:#?}");
                return;
            }
        };

        counter += 1;

        delivery
            .ack(BasicAckOptions::default())
            .await
            .expect("Failed to ack send_webhook_event message");
    });

    counter += 1;
    tracing::info!("amount consumer messages: {counter}");

    Ok(())
}
