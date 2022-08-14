use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use identicon_rs::error::IdenticonError;
use serde_json::json;

pub(crate) enum AppError {
    /// Something went wrong when calling the user repo.
    Identicon(IdenticonError),
}

/// This makes it possible to use `?` to automatically convert a `UserRepoError`
/// into an `AppError`.
impl From<IdenticonError> for AppError {
    fn from(inner: IdenticonError) -> Self {
        AppError::Identicon(inner)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Identicon(error) => (StatusCode::NOT_FOUND, error.to_string()),
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
