use thiserror::Error;

#[derive(Debug, Error)]
pub enum RssError {
    #[error("request timeout: {0}")]
    RequestTimeout(String),
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("service processing error: {0}")]
    ServiceError(String),
    #[error("failed to de/serialize: {0}")]
    SerdeError(String),
    #[error("rss crate error: {0}")]
    RssService(#[from] rss::Error),
    #[error("retry crate error: {0}")]
    RetryFailed(String),
}

impl From<reqwest::Error> for RssError {
    fn from(err: reqwest::Error) -> Self {
        let Some(status) = err.status() else {
            return RssError::ServiceError(err.to_string());
        };

        match status.as_u16() {
            503 => RssError::ServiceUnavailable(err.to_string()),
            408 => RssError::RequestTimeout(err.to_string()),
            _ => RssError::ServiceError(err.to_string()),
        }
    }
}

impl From<reqwest_middleware::Error> for RssError {
    fn from(err: reqwest_middleware::Error) -> Self {
        let Some(status) = err.status() else {
            return RssError::ServiceError(err.to_string());
        };

        match status.as_u16() {
            503 => RssError::ServiceUnavailable(err.to_string()),
            408 => RssError::RequestTimeout(err.to_string()),
            _ => RssError::ServiceError(err.to_string()),
        }
    }
}
