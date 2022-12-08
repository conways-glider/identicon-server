use axum::{response::Html, routing::get, Router};
use tracing::{info, instrument};

use crate::config::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(handler))
}

#[instrument]
async fn handler() -> Html<&'static str> {
    info!("running root");
    Html("<h1>Hello, World!</h1>")
}
