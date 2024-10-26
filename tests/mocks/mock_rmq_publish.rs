use news_rss::publish::models::PublishNews;
use news_rss::publish::rabbit::config::RabbitConfig;
use news_rss::publish::Publisher;
use news_rss::ServiceConnect;

pub struct MockRabbitPublisher {}

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
        tracing::info!("rabbit confirm message successful: {}", msg_body.id());
        Ok(())
    }
}
