pub mod config;
pub mod models;
#[cfg(feature = "publish-offline")]
pub mod pgsql;
pub mod rabbit;

use crate::publish::models::PublishNews;
use std::fmt::Debug;

#[async_trait::async_trait]
pub trait Publisher {
    type Error: Debug;

    async fn publish(&self, msg_body: &PublishNews) -> Result<(), Self::Error>;
}
