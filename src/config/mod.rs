use std::sync::Arc;

#[derive(Clone)]
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

type SharedConfig = Arc<Config>;

pub struct Config {}
