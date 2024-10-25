use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("failed to insert data into cache: {0}")]
    InsertFailed(String),
    #[error("failed to load data from cache: {0}")]
    LoadFailed(String),
    #[error("internal cache service error: {0}")]
    ServiceError(String),
}
