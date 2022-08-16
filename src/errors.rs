use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use identicon_rs::error::IdenticonError;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum AppError {
    /// Something went wrong when calling the identicon-rs library
    #[error("identicon-rs error")]
    Identicon(#[from] IdenticonError),

    #[error("scale must be equal to or less than 1024: {0}")]
    ScaleTooLarge(u32),
    #[error("scale must be greater than the size: {scale} < {size}")]
    ScaleTooSmall { scale: u32, size: u32 },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Identicon(error) => match error {
                IdenticonError::ScaleTooSmallError(_) => {
                    (StatusCode::BAD_REQUEST, error.to_string())
                }
                IdenticonError::SizeTooLargeError(_) => {
                    (StatusCode::BAD_REQUEST, error.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            },
            AppError::ScaleTooLarge(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::ScaleTooSmall { scale: _, size: _ } => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
        };

        new(status, error_message.as_str())
    }
}

pub(crate) fn new(status_code: StatusCode, error_message: &str) -> Response {
    let body = Json(json!({
        "error": error_message,
    }));
    (status_code, body).into_response()
}
