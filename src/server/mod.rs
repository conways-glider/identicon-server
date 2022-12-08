use axum::{routing::get, Router};
use std::net::SocketAddr;

mod hello;
mod signal;

pub async fn start_server() {
    // build our application with a route
    let app = Router::new().route("/", get(hello::handler));

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(signal::shutdown_signal())
        .await
        .unwrap();
}
