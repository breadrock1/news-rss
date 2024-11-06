use thiserror::Error;

#[derive(Debug, Error)]
pub enum RabbitPublishError {
    #[error("client rabbitmq error: {0}")]
    Client(#[from] lapin::Error),
    #[error("producer rabbitmq error: {0}")]
    Produce(String),
    #[error("failed to publish msg: {0}")]
    Publish(String),
    #[error("failed to (de)serialize object: {0}")]
    SerdeError(#[from] serde_json::Error),
}
