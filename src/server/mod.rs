use anyhow::Context;
use axum::{error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, Router};
use axum_extra::routing::SpaRouter;
use std::{borrow::Cow, net::SocketAddr, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{debug, info, instrument, Level};

use crate::config::AppState;

mod image;
mod signal;

pub async fn start_server(state: AppState) -> anyhow::Result<()> {
    let port = state.config.port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // build our application with a route
    let app = api_router(state);

    info!(addr = ?addr, "starting server");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(signal::shutdown_signal())
        .await
        .context("error running server")
}

#[instrument]
fn api_router(state: AppState) -> Router {
    // Enables logging. Use `RUST_LOG=tower_http=debug`
    debug!("constructing trace layer");
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::default().level(Level::INFO))
        .on_response(DefaultOnResponse::default().level(Level::INFO))
        .on_request(DefaultOnRequest::default().level(Level::INFO));

    // construct middleware
    debug!("constructing middleware");
    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .load_shed()
        .concurrency_limit(1024)
        .timeout(Duration::from_secs(10))
        .layer(trace_layer);

    // set up router
    info!("constructing router");
    Router::new()
        .merge(SpaRouter::new("/assets", "assets").index_file("index.html"))
        .merge(image::router())
        .layer(middleware_stack)
        .with_state(state)
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
