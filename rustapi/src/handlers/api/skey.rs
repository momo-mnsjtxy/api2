use std::collections::HashMap;

use axum::{extract::Query, response::Response};
use serde_json::json;

use crate::{respond, services::qqskey::QqLogin};

type Params = HashMap<String, String>;

pub async fn index(Query(params): Query<Params>) -> Response {
    let do_cmd = params.get("do").map(String::as_str).unwrap_or_default();
    if do_cmd.is_empty() {
        return respond::int(
            params.get("type").map(String::as_str),
            json!({"code": 400, "msg": "unknown do"}),
        );
    }

    let pairs = params
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<_>>();
    let data = QqLogin::new().dispatch(do_cmd, &pairs);
    respond::int(params.get("type").map(String::as_str), data)
}
