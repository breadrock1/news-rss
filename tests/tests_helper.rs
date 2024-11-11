use lapin::message::DeliveryResult;
use lapin::options::{BasicAckOptions, BasicConsumeOptions};
use lapin::options::{ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use lapin::{Connection, ConnectionProperties, ExchangeKind};

#[cfg(feature = "cache-redis")]
use news_rss::cache::redis::RedisClient;

#[cfg(feature = "crawler-llm")]
use news_rss::crawler::llm::LlmCrawler;

#[cfg(feature = "publish-offline")]
use news_rss::publish::pgsql::PgsqlPublisher;

#[cfg(feature = "storage-pgsql")]
use news_rss::storage::pgsql::PgsqlTopicStorage;

use news_rss::cache::local::LocalCache;
use news_rss::config::ServiceConfig;
use news_rss::crawler::native::NativeCrawler;
use news_rss::publish::rabbit::config::RabbitConfig;
use news_rss::publish::rabbit::RabbitPublisher;
use news_rss::ServiceConnect;
use std::sync::Arc;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[allow(dead_code)]
const TEST_AMQP_CONSUMER_TAG: &str = "test-news-rss-consumer";
pub const TEST_LLM_URL: &str = "/v1/chat/completions";
pub const TEST_NEWS_URL: &str = "/news/index.html";
pub const INPUT_LLM_RESP_JSON_DATA: &str = include_str!("resources/llm-chat-response.json");

pub async fn build_mock_server() -> MockServer {
    let mock_server = MockServer::start().await;
    create_llm_completion_route(&mock_server, TEST_LLM_URL, "POST").await;
    mock_server
}

pub async fn create_llm_completion_route(mock: &MockServer, url: &str, http_method: &str) {
    let resp_template = ResponseTemplate::new(200).set_body_string(INPUT_LLM_RESP_JSON_DATA);

    Mock::given(method(http_method))
        .and(path(url))
        .respond_with(resp_template)
        .mount(mock)
        .await;
}

#[allow(dead_code)]
#[allow(unused_assignments)]
#[allow(unused_variables)]
pub async fn rabbit_consumer(config: &RabbitConfig) -> Result<(), anyhow::Error> {
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

    let queue_decl_opts = QueueDeclareOptions {
        durable: true,
        ..Default::default()
    };

    channel
        .queue_declare(config.stream_name(), queue_decl_opts, FieldTable::default())
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
        .await?;

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

#[allow(dead_code)]
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

#[cfg(feature = "storage-pgsql")]
pub async fn build_pgsql_storage(
    config: &ServiceConfig,
) -> Result<Arc<PgsqlTopicStorage>, anyhow::Error> {
    let pgsql_config = config.storage().pgsql();
    let pgsql = PgsqlTopicStorage::connect(pgsql_config).await?;
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
