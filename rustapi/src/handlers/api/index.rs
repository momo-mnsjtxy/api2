use axum::response::Response;
use serde_json::json;

use crate::respond;

pub async fn index() -> Response {
    respond::json_response(json!({"code": 400, "msg": "接口错误", "api": ""}))
}
