use thiserror::Error;

#[derive(Debug, Error)]
pub enum RabbitPublishError {
    #[error("client rabbitmq error: {0}")]
    Client(String),
    #[error("producer rabbitmq error: {0}")]
    Produce(String),
    #[error("failed to publish msg: {0}")]
    Publish(String),
    #[error("failed to (de)serialize object: {0}")]
    SerdeError(String),
}

impl From<lapin::Error> for RabbitPublishError {
    fn from(err: lapin::Error) -> Self {
        RabbitPublishError::Client(err.to_string())
    }
}

impl From<serde_json::Error> for RabbitPublishError {
    fn from(err: serde_json::Error) -> Self {
        RabbitPublishError::SerdeError(err.to_string())
    }
}
