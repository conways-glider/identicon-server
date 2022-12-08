use std::{fmt, sync::Arc};

use axum::extract::FromRef;

#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub config: SharedConfig,
}

impl AppState {
    pub fn load() -> Self {
        // load app state
        let config = Config { port: 3000 };
        AppState {
            config: Arc::new(config),
        }
    }
}

pub type SharedConfig = Arc<Config>;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
}

/// Formats the value using the given formatter by hand to prevent leaking secrets.
///
/// [Example Implementation from stdlib](https://doc.rust-lang.org/std/fmt/struct.Formatter.html#method.debug_struct)
impl fmt::Debug for Config {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Config")
            .field("port", &self.port)
            .finish()
    }
}
