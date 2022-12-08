use anyhow::Context;
use axum::Router;
use tracing::info;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

mod root;
mod signal;

pub async fn start_server() -> anyhow::Result<()> {
    // build our application with a route
    let app = api_router();

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!(addr = ?addr, "starting server");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(signal::shutdown_signal())
        .await
        .context("error running server")
}

fn api_router() -> Router {
    // This is the order that the modules were authored in.
    Router::new()
        .merge(root::router())
        // Enables logging. Use `RUST_LOG=tower_http=debug`
        .layer(TraceLayer::new_for_http())
}
