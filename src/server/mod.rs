use anyhow::Context;
use axum::{error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, Router};
use std::{borrow::Cow, net::SocketAddr, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};

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
    // construct middleware
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::default().level(Level::INFO))
        .on_response(DefaultOnResponse::default().level(Level::INFO))
        .on_request(DefaultOnRequest::default().level(Level::INFO));

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .load_shed()
        .concurrency_limit(1024)
        .timeout(Duration::from_secs(10))
        .layer(trace_layer);

    Router::new()
        .merge(root::router())
        // Enables logging. Use `RUST_LOG=tower_http=debug`
        .layer(middleware_stack)
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}
