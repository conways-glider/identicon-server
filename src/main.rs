use std::net::SocketAddr;

use axum::{routing::get, Extension, Router};
use clap::Parser;

mod error;
mod image;

#[derive(Parser, Debug, Clone)]
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
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/:name", get(image::generate_image))
        .route("/", get(root))
        .layer(Extension(args));
    // `POST /users` goes to `create_user`
    // .route("/users", post(create_user));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "Hello, World!"
}
