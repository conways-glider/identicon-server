use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer, http::StatusCode, response::{IntoResponse, Html}, routing::get,
    Extension, Router,
};
use clap::Parser;
use tokio::signal;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod errors;
mod image;

const INDEX: &'static str = include_str!("../assets/index.html");

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

type AppData = Arc<Args>;

#[tokio::main]
async fn main() {
    identicon_rs::Identicon::default().border();
    let args = Args::parse();
    tracing_subscriber::fmt::init();

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

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .load_shed()
        .concurrency_limit(1024)
        .timeout(Duration::from_secs(5))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_data));

    // Construct App
    let app = Router::new()
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/:name", get(image::generate_image_path))
        .layer(middleware_stack);

    // Start Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
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

async fn index() -> Html<&'static str> {
    Html(INDEX)
}
