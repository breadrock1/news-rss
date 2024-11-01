use crate::server::swagger::SwaggerExamples;

use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Error, Serialize, ToSchema)]
pub enum ServerError {
    #[error("not found error: {0}")]
    NotFound(String),

    #[error("worker {0} is launched")]
    Launched(String),

    #[error("internal service error: {0}")]
    InternalError(String),

    #[error("service unavailable")]
    ServiceUnavailable,
}

impl ServerError {
    pub fn status_code(&self) -> (&str, StatusCode) {
        match self {
            ServerError::NotFound(msg) => (msg, StatusCode::NOT_FOUND),
            ServerError::Launched(msg) => (msg, StatusCode::CONFLICT),
            ServerError::InternalError(msg) => (msg, StatusCode::INTERNAL_SERVER_ERROR),
            ServerError::ServiceUnavailable => {
                ("service unavailable", StatusCode::SERVICE_UNAVAILABLE)
            }
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (msg, status) = self.status_code();
        let mut resp = Json(ErrorResponse {
            message: msg.to_string(),
        })
        .into_response();

        *resp.status_mut() = status;
        resp
    }
}

impl SwaggerExamples for ServerError {
    type Example = Self;

    fn example(value: Option<String>) -> Self::Example {
        match value {
            None => ServerError::ServiceUnavailable,
            Some(msg) => ServerError::InternalError(msg),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct Success {
    status: u16,
    message: String,
}

impl Default for Success {
    fn default() -> Self {
        Success {
            status: 200,
            message: "Ok".to_string(),
        }
    }
}

impl SwaggerExamples for Success {
    type Example = Self;

    fn example(_value: Option<String>) -> Self::Example {
        Success::default()
    }
}
