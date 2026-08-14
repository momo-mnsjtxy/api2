use axum::response::Response;
use serde_json::json;

use super::{int, ApiCtx};

pub async fn index(ctx: ApiCtx) -> Response {
    int(&ctx, json!({"code": 400, "msg": "接口错误", "api": ""}))
}
