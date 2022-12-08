use std::sync::Arc;

use config::{AppState, Config};

mod config;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    start_logger();

    // load app state
    let config = Config {};
    let state = AppState {
        config: Arc::new(config),
    };

    server::start_server(state).await
}

fn start_logger() {
    tracing_subscriber::fmt::init();
}
