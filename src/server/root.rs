use axum::{response::Html, routing::get, Router};
use tracing::instrument;

pub fn router() -> Router {
    // By having each module responsible for setting up its own routing,
    // it makes the root module a lot cleaner.
    Router::new().route("/", get(handler))
}

#[instrument]
async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
