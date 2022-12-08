use std::sync::Arc;

use axum::extract::FromRef;

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub config: SharedConfig,
}

impl AppState {
    pub fn load() -> Self {
        // load app state
        let config = Config {};
        AppState {
            config: Arc::new(config),
        }
    }
}

pub type SharedConfig = Arc<Config>;

#[derive(Clone, Debug)]
pub struct Config {}
