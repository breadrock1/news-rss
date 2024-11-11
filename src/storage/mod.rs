pub mod config;

#[cfg(feature = "storage-pgsql")]
pub mod pgsql;

#[async_trait::async_trait]
pub trait LoadTopic {
    type Error;
    type Topic;

    async fn load_all(&self) -> Result<Vec<Self::Topic>, Self::Error>;
    async fn load_at_launch(&self) -> Result<Vec<Self::Topic>, Self::Error>;
}
