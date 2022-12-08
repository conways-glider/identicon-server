use axum::{response::Html, routing::get, Router};
use tracing::instrument;

use crate::config::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}

#[instrument]
async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
