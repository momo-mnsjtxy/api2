use crate::cache::AppCache;
use crate::config::AppConfig;
use reqwest::Client;
use sqlx::MySqlPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub http: Client,
    pub config: Arc<AppConfig>,
    pub cache: AppCache,
}

impl AppState {
    pub fn new(pool: MySqlPool, config: AppConfig, cache: AppCache) -> Self {
        let http = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; api2-rust/0.1)")
            .build()
            .expect("http client");
        Self {
            pool,
            http,
            config: Arc::new(config),
            cache,
        }
    }
}
