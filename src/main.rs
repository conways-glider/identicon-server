use std::net::SocketAddr;

use axum::{routing::get, Extension, Router};
use clap::Parser;
use tokio::signal;
use tracing::info;

mod error;
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

    tracing_subscriber::fmt::init();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/:name", get(image::generate_image_path))
        .route("/", get(root))
        .layer(Extension(args));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
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
