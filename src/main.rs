use config::AppState;

mod config;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    start_logger();

    let state = AppState::load();

    server::start_server(state).await
}

fn start_logger() {
    tracing_subscriber::fmt::init();
}
