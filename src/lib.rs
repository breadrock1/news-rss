pub mod cache;
pub mod config;
pub mod crawler;
pub mod feeds;
pub mod logger;
pub mod publish;
pub mod server;

#[async_trait::async_trait]
pub trait ServiceConnect {
    type Config;
    type Error;
    type Client;

    async fn connect(config: &Self::Config) -> Result<Self::Client, Self::Error>;
}
