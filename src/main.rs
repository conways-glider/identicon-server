mod config;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    start_logger();
    server::start_server().await
}

fn start_logger() {
    tracing_subscriber::fmt::init();
}
