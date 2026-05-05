use super::config::ServerConfig;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub(super) struct AppState {
    pub(super) conversions: Arc<Semaphore>,
    pub(super) config: ServerConfig,
}

impl AppState {
    pub(super) fn new(config: ServerConfig) -> Self {
        Self {
            conversions: Arc::new(Semaphore::new(config.max_concurrency)),
            config,
        }
    }
}
