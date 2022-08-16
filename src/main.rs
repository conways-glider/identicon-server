use std::{net::SocketAddr, time::Duration};

use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::IntoResponse, routing::get,
    Extension, Router,
};
use clap::Parser;
use tokio::signal;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::info;

mod errors;
mod image;

#[derive(Parser, Debug, Clone, Copy)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(long, value_parser, default_value_t = identicon_rs::Identicon::default().border())]
    border: u32,

    #[clap(long, value_parser, default_value_t = identicon_rs::Identicon::default().size())]
    size: u32,

    #[clap(long, value_parser, default_value_t = identicon_rs::Identicon::default().scale())]
    scale: u32,

    #[clap(long, value_parser, default_value_t = identicon_rs::Identicon::default().mirrored())]
    mirrored: bool,
}

#[tokio::main]
async fn main() {
    identicon_rs::Identicon::default().border();
    let args = Args::parse();

    if args.scale < args.size {
        panic!("scale must be equal to or larger than size");
    } else if args.scale > 1024 {
        panic!("scale must be equal to or less than 1024");
    }

    tracing_subscriber::fmt::init();

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .load_shed()
        .concurrency_limit(1024)
        .timeout(Duration::from_secs(5))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(args));

    let app = Router::new()
        .route("/", get(root))
        .route("/:name", get(image::generate_image_path))
        .layer(middleware_stack);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<errors::AppError>() {
        errors::new(StatusCode::IM_A_TEAPOT, "TEAPOT")
    } else if error.is::<tower::timeout::error::Elapsed>() {
        // return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out")).into_response();
        errors::new(StatusCode::REQUEST_TIMEOUT, "request timed out")
    } else if error.is::<tower::load_shed::error::Overloaded>() {
        errors::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service is overloaded, try again later",
        )
    } else {
        errors::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            // &format!("unhandled internal error: {0}", error),
            "unhandled internal error",
        )
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

async fn root() -> &'static str {
    todo!()
}
