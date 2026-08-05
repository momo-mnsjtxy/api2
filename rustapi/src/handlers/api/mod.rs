pub mod index;
pub mod page;
pub mod skey;
pub mod update;
pub mod v2;

use axum::http::HeaderMap;
use std::collections::HashMap;

use crate::state::AppState;

#[derive(Clone)]
pub struct ApiCtx {
    pub state: AppState,
    pub params: HashMap<String, String>,
    pub method: String,
    pub ua: String,
    pub ip: String,
    pub headers: HeaderMap,
    pub file: Option<(String, Vec<u8>)>,
}

impl ApiCtx {
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params
            .get(key)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
    }

    pub fn param_raw(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|s| s.as_str())
    }

    pub fn param_or<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.param(key).unwrap_or(default)
    }

    pub fn param_i64(&self, key: &str, default: i64) -> i64 {
        self.param(key)
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(default)
    }

    pub fn output_type(&self) -> Option<&str> {
        self.params.get("type").map(|s| s.as_str())
    }
}
