use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Extension, Router,
};
use clap::Parser;
use tokio::signal;
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, Level};

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

    #[clap(long, value_parser, default_value_t = false)]
    json_logs: bool,
}

type AppData = Arc<Args>;

#[tokio::main]
async fn main() {
    identicon_rs::Identicon::default().border();
    let args = Args::parse();

    // Set up logging
    if args.json_logs {
        tracing_subscriber::fmt().with_ansi(false).json().init();
    } else {
        tracing_subscriber::fmt::init();
    }

    // Validate args
    if args.scale < args.size {
        let err = errors::AppError::ScaleTooSmall {
            scale: args.scale,
            size: args.size,
        };
        error!("{}", err);
        panic!("{}", err.to_string());
    } else if args.scale > 1024 {
        let err = errors::AppError::ScaleTooLarge(args.scale);
        error!("{}", err);
        panic!("{}", err.to_string());
    }

    // Construct Middleware
    let app_data: AppData = Arc::new(args);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::default().level(Level::INFO))
        .on_response(DefaultOnResponse::default().level(Level::INFO))
        .on_request(DefaultOnRequest::default().level(Level::INFO));

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .load_shed()
        .concurrency_limit(1024)
        .timeout(Duration::from_secs(5))
        .layer(trace_layer)
        .layer(Extension(app_data));

    // Construct App
    let app = Router::new()
        .route("/identicon/:name", get(image::generate_image_path))
        .fallback(get_service(ServeDir::new("assets")).handle_error(handle_serve_dir_error))
        .layer(middleware_stack);

    // Start Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handle_serve_dir_error(_err: std::io::Error) -> impl IntoResponse {
    errors::new(StatusCode::INTERNAL_SERVER_ERROR, "could not read asset")
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    error!("handling error: {}", error);
    if error.is::<errors::AppError>() {
        match error.downcast::<errors::AppError>() {
            Ok(err) => err.into_response(),
            Err(_) => errors::new(StatusCode::INTERNAL_SERVER_ERROR, "internal server error"),
        }
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
