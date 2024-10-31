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
    
    #[error("worker {0} already launched")]
    AlreadyLaunched(String),
    
    #[error("internal service error: {0}")]
    InternalError(String),
    
    #[error("service unavailable")]
    ServiceUnavailable,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    message: String,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status = StatusCode::INTERNAL_SERVER_ERROR;
        let message = match self {
            ServerError::NotFound(msg) => msg,
            _ => "runtime error".to_string(),
        };

        let mut resp = Json(ErrorResponse { message }).into_response();
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
