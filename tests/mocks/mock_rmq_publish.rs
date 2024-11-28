use news_rss::publish::models::PublishNews;
use news_rss::publish::rabbit::config::RabbitConfig;
use news_rss::publish::Publisher;
use news_rss::ServiceConnect;

#[derive(Clone)]
pub struct MockRabbitPublisher;

#[async_trait::async_trait]
impl ServiceConnect for MockRabbitPublisher {
    type Config = RabbitConfig;
    type Error = anyhow::Error;
    type Client = Self;

    async fn connect(_config: &Self::Config) -> Result<Self::Client, Self::Error> {
        Ok(MockRabbitPublisher {})
    }
}

#[async_trait::async_trait]
impl Publisher for MockRabbitPublisher {
    type Error = ();

    async fn publish(&self, msg_body: &PublishNews) -> Result<(), Self::Error> {
        let id = msg_body.id();
        tracing::info!(article = id, "rabbit confirm msg successful");
        Ok(())
    }
}
