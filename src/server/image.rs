use axum::{extract::Path, response::Html, routing::get, Router};
use tracing::{info, instrument};

use crate::config::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/identicon/:path", get(identicon))
}

#[instrument]
async fn identicon(Path(path): Path<String>) -> Html<&'static str> {
    info!("generating identicon");
    Html("<h1>TODO</h1>")
}
