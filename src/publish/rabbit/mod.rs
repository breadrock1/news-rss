pub mod config;
mod errors;

use crate::publish::models::PublishNews;
use crate::publish::rabbit::config::RabbitConfig;
use crate::publish::rabbit::errors::RabbitPublishError;
use crate::publish::Publisher;
use crate::ServiceConnect;

use lapin::options::{BasicPublishOptions, ExchangeDeclareOptions};
use lapin::types::FieldTable;
use lapin::{BasicProperties, ConnectionProperties, ExchangeKind};
use lapin::{Channel, Connection};
use std::sync::Arc;

#[derive(Clone)]
pub struct RabbitPublisher {
    config: Arc<RabbitConfig>,
    channel: Arc<Channel>,
}

impl RabbitPublisher {
    pub fn channel(&self) -> Arc<Channel> {
        self.channel.clone()
    }
}

#[async_trait::async_trait]
impl ServiceConnect for RabbitPublisher {
    type Config = RabbitConfig;
    type Error = RabbitPublishError;
    type Client = Self;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error> {
        let conn_props = ConnectionProperties::default();
        let connection = Connection::connect(config.address(), conn_props).await?;
        let channel = connection.create_channel().await?;

        let exchange_opts = ExchangeDeclareOptions {
            nowait: true,
            durable: false,
            ..Default::default()
        };

        channel
            .exchange_declare(
                config.exchange(),
                ExchangeKind::Fanout,
                exchange_opts,
                FieldTable::default(),
            )
            .await?;

        let client = RabbitPublisher {
            config: Arc::new(config.to_owned()),
            channel: Arc::new(channel),
        };

        Ok(client)
    }
}

#[async_trait::async_trait]
impl Publisher for RabbitPublisher {
    type Error = RabbitPublishError;

    async fn publish(&self, news: &PublishNews) -> Result<(), Self::Error> {
        let exchange = self.config.exchange();
        let routing = self.config.routing_key();
        let bytes = serde_json::to_vec(&news)?;
        let pub_opts = BasicPublishOptions {
            mandatory: true,
            immediate: false,
        };

        let confirm = self
            .channel
            .basic_publish(
                exchange,
                routing,
                pub_opts,
                bytes.as_slice(),
                BasicProperties::default(),
            )
            .await?
            .await?;

        tracing::info!(
            exchange = exchange,
            routing_key = routing,
            "rabbit confirm: {confirm:?}"
        );

        Ok(())
    }
}
