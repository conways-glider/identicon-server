use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: SharedConfig,
}

type SharedConfig = Arc<Config>;

pub struct Config {}
