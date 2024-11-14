pub mod config;
pub mod pgsql;

#[async_trait::async_trait]
pub trait LoadTopic {
    type Error;
    type Topic;
    type TopicId;

    async fn load_all(&self) -> Result<Vec<Self::Topic>, Self::Error>;
    async fn load_at_launch(&self) -> Result<Vec<Self::Topic>, Self::Error>;
    async fn search_source(&self, query: &str) -> Result<Vec<Self::Topic>, Self::Error>;
    async fn add_source(&self, topic: &Self::Topic) -> Result<(), Self::Error>;
    async fn remove_source(&self, id: Self::TopicId) -> Result<(), Self::Error>;
    async fn update_source(&self, topic: &Self::Topic) -> Result<(), Self::Error>;
}
