pub mod config;
mod errors;

use crate::publish::models::PublishNews;
use crate::publish::rabbit::config::RabbitConfig;
use crate::publish::rabbit::errors::RabbitPublishError;
use crate::publish::Publisher;
use crate::ServiceConnect;

use lapin::options::{BasicPublishOptions, ExchangeDeclareOptions};
use lapin::options::{QueueBindOptions, QueueDeclareOptions};
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
        let bytes = serde_json::to_vec(&news)?;
        let pub_opts = BasicPublishOptions {
            mandatory: true,
            immediate: false,
        };
        let pub_props = BasicProperties::default();

        let confirm = self
            .channel
            .basic_publish(
                self.config.exchange(),
                self.config.routing_key(),
                pub_opts,
                bytes.as_slice(),
                pub_props,
            )
            .await?
            .await?;

        tracing::info!("rabbit confirm is: {confirm:?}");

        Ok(())
    }
}
