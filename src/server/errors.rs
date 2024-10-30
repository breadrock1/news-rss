use axum::{
    response::{IntoResponse, Response},
    Json,
};
use reqwest::StatusCode;
use serde::Serialize;
use thiserror::Error;

pub type ServerResult<T> = Result<T, ServerError>;

#[derive(Debug, Error, Serialize)]
pub enum ServerError {
    #[error("not found error: {0}")]
    NotFound(String),
    #[error("worker {0} already launched")]
    AlreadyLaunched(String),
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

#[derive(Serialize)]
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
