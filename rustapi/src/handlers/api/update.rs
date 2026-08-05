use std::{collections::HashMap, path::PathBuf};

use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Local;
use serde_json::{json, Value};
use sqlx::{MySqlPool, Row};

use crate::{respond, services, state::AppState};

type Params = HashMap<String, String>;

pub async fn index() -> Response {
    respond::json_response(json!({"code": 400, "msg": "接口错误", "api": ""}))
}

pub async fn music_hot(State(state): State<AppState>) -> Response {
    let Some(list) = fetch_json(
        &state,
        "https://www.gqink.cn/usr/themes/handsome/libs/Get.php?type=collect&media=netease&id=3778678",
    )
    .await
    .and_then(|v| v.as_array().cloned())
    else {
        return text("更新失败");
    };

    let mut data = Vec::new();
    for item in list {
        let song_id = value_string(item.get("song_id")).unwrap_or_default();
        let lrc = services::music::m_163(&state.http, &song_id)
            .await
            .and_then(|v| v.get("lrc").and_then(Value::as_str).map(ToOwned::to_owned))
            .unwrap_or_default();
        data.push(json!({
            "name": value_string(item.get("name")).unwrap_or_default(),
            "url": format!("https://music.163.com/song/media/outer/url?id={song_id}"),
            "cover": value_string(item.get("cover")).unwrap_or_default(),
            "author": value_string(item.get("author")).unwrap_or_default(),
            "lrc": lrc,
        }));
    }

    let payload = match serde_json::to_string(&data) {
        Ok(payload) => payload,
        Err(_) => return text("更新失败"),
    };
    let path = PathBuf::from(&state.config.public_dir).join("hot.json");
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    match tokio::fs::write(path, payload.as_bytes()).await {
        Ok(_) => text(format!("{}更新成功", payload.len())),
        Err(_) => text("更新失败"),
    }
}

pub async fn word(State(state): State<AppState>) -> Response {
    let Some(hitokoto) = fetch_json(&state, "https://v1.hitokoto.cn/")
        .await
        .and_then(|v| value_string(v.get("hitokoto")))
    else {
        return text("内部错误");
    };
    text(
        insert_unique(
            &state.pool,
            "word",
            "text",
            &hitokoto,
            &[("love", json!(520))],
        )
        .await,
    )
}

pub async fn wordtime(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    let day = params
        .get("time")
        .filter(|v| !v.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string());
    let url = format!("http://www.dutangapp.cn/u/toxic?date={day}");
    let Some(items) = fetch_json(&state, &url)
        .await
        .and_then(|v| v.get("data").and_then(Value::as_array).cloned())
    else {
        return text("内部错误");
    };

    let mut body = String::new();
    for item in items {
        let line = value_string(item.get("data")).unwrap_or_default();
        if line.is_empty() {
            continue;
        }
        body.push_str(
            &insert_unique(&state.pool, "word", "text", &line, &[("love", json!(520))]).await,
        );
        body.push_str("<br/>");
    }
    text(if body.is_empty() {
        "内部错误".into()
    } else {
        body
    })
}

pub async fn netease(State(state): State<AppState>) -> Response {
    let Some(data) = fetch_json(&state, "https://api.uomg.com/api/comments.163")
        .await
        .and_then(|v| v.get("data").cloned())
    else {
        return text("内部错误");
    };
    text(insert_hot_comment(&state.pool, &data).await)
}

pub async fn headimg(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    let kw = params.get("kw").cloned().unwrap_or_default();
    let kwn = params.get("kwn").cloned().unwrap_or_default();
    let start = params
        .get("start")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let next = start + 30;
    let url = format!(
        "http://api4.qzone.cc/feed/default/get-new-feed-by-feed-tag?feed_tag_id={kwn}&start={start}&len=30"
    );
    let inserted = match fetch_json(&state, &url).await {
        Some(v) => {
            let mut inserted = 0;
            if let Some(items) = v.get("data").and_then(Value::as_array) {
                for item in items {
                    if let Some(images) = item.get("image").and_then(Value::as_array) {
                        for image in images {
                            if let Some(url) = value_string(image.get("image_url")) {
                                if insert_unique(&state.pool, &kw, "url", &url, &[]).await
                                    == "更新成功"
                                {
                                    inserted += 1;
                                }
                            }
                        }
                    }
                }
                format!("写入{inserted}条成功")
            } else {
                "内部错误".to_string()
            }
        }
        None => "内部错误".to_string(),
    };
    text(format!(
        "<title>{inserted}</title>{inserted}<script>window.onload=function(){{window.location.replace('https://api.gqink.cn/api/update/headimg/kw/{kw}/kwn/{kwn}/start/{next}/');}}</script>"
    ))
}

pub async fn words(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    qzone_words(state, params, 1070092, "word", "text", None, "words").await
}

pub async fn qianming(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    qzone_words(
        state,
        params,
        1019,
        "autograph",
        "word",
        Some(("keyword", json!("伤感"))),
        "qianming",
    )
    .await
}

pub async fn wangming(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    qzone_words(
        state,
        params,
        1025,
        "name",
        "word",
        Some(("keyword", json!("女生"))),
        "wangming",
    )
    .await
}

pub async fn net(State(state): State<AppState>) -> Response {
    // The old action depended on a PHP scraper class. Keep the update surface alive
    // using the same hot-comment schema via the public comments endpoint.
    netease(State(state)).await
}

pub async fn dog(State(state): State<AppState>) -> Response {
    let body = match fetch_json(&state, "https://v1.alapi.cn/api/dog?format=json").await {
        Some(v) if v.get("code").and_then(Value::as_i64) == Some(200) => {
            let text = v
                .pointer("/data/content")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            if text.is_empty() {
                "内部错误".to_string()
            } else {
                match insert_unique(&state.pool, "dog", "text", &text, &[])
                    .await
                    .as_str()
                {
                    "更新成功" => "写入成功".to_string(),
                    "数据重复" => "重复".to_string(),
                    _ => "失败".to_string(),
                }
            }
        }
        _ => "内部错误".to_string(),
    };
    text(format!(
        "{body}<script>window.onload=function(){{window.location.replace('https://api.gqink.cn/api/update/dog/');}}</script>"
    ))
}

async fn qzone_words(
    state: AppState,
    params: Params,
    feed_tag_id: i64,
    table: &'static str,
    column: &'static str,
    extra: Option<(&'static str, Value)>,
    action: &'static str,
) -> Response {
    let start = params
        .get("start")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let next = start + 100;
    let url = format!(
        "http://api4.qzone.cc/feed/default/get-new-feed-by-feed-tag?feed_tag_id={feed_tag_id}&start={start}&len=100"
    );
    let inserted = match fetch_json(&state, &url).await {
        Some(v) => {
            let mut inserted = 0;
            if let Some(items) = v.get("data").and_then(Value::as_array) {
                for item in items {
                    let Some(line) = value_string(item.get("text")) else {
                        continue;
                    };
                    let extras = extra
                        .as_ref()
                        .map(|(k, v)| vec![(*k, v.clone())])
                        .unwrap_or_default();
                    let extras = extras
                        .iter()
                        .map(|(k, v)| (*k, v.clone()))
                        .collect::<Vec<_>>();
                    if insert_unique_owned(&state.pool, table, column, &line, &extras).await
                        == "更新成功"
                    {
                        inserted += 1;
                    }
                }
                format!("写入{inserted}条成功")
            } else {
                "内部错误".to_string()
            }
        }
        None => "内部错误".to_string(),
    };
    text(format!(
        "<title>{inserted}</title>{inserted}<script>window.onload=function(){{window.location.replace('https://api.gqink.cn/api/update/{action}/start/{next}/');}}</script>"
    ))
}

async fn insert_hot_comment(pool: &MySqlPool, data: &Value) -> String {
    let comment = value_string(data.get("content")).unwrap_or_default();
    if comment.is_empty() {
        return "内部错误".into();
    }
    let exists = sqlx::query("SELECT id FROM `hot` WHERE comment_content=? LIMIT 1")
        .bind(&comment)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        return "数据重复".into();
    }
    let inserted = sqlx::query(
        "INSERT INTO `hot` (name, images, author, mp3_url, comment_nickname, comment_content, love) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(value_string(data.get("name")).unwrap_or_default())
    .bind(value_string(data.get("picurl")).unwrap_or_default())
    .bind(value_string(data.get("artistsname")).unwrap_or_default())
    .bind(value_string(data.get("url")).unwrap_or_default())
    .bind(value_string(data.get("nickname")).unwrap_or_default())
    .bind(comment)
    .bind(520_i64)
    .execute(pool)
    .await
    .is_ok();
    if inserted {
        "更新成功".into()
    } else {
        "更新失败".into()
    }
}

async fn insert_unique(
    pool: &MySqlPool,
    table: &str,
    unique_column: &str,
    unique_value: &str,
    extra: &[(&str, Value)],
) -> String {
    let owned = extra
        .iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect::<Vec<_>>();
    insert_unique_owned(pool, table, unique_column, unique_value, &owned).await
}

async fn insert_unique_owned(
    pool: &MySqlPool,
    table: &str,
    unique_column: &str,
    unique_value: &str,
    extra: &[(&str, Value)],
) -> String {
    let Some(table_sql) = quote_table(table) else {
        return "内部错误".into();
    };
    let column_sql = quote_ident(unique_column);
    let exists_sql = format!("SELECT id FROM {table_sql} WHERE {column_sql}=? LIMIT 1");
    let exists = sqlx::query(&exists_sql)
        .bind(unique_value)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if exists {
        return "数据重复".into();
    }

    let mut columns = vec![column_sql];
    let mut placeholders = vec!["?".to_string()];
    for (column, _) in extra {
        columns.push(quote_ident(column));
        placeholders.push("?".to_string());
    }
    let insert_sql = format!(
        "INSERT INTO {table_sql} ({}) VALUES ({})",
        columns.join(","),
        placeholders.join(",")
    );
    let mut query = sqlx::query(&insert_sql).bind(unique_value.to_string());
    for (_, value) in extra {
        query = bind_json_value(query, value);
    }
    if query.execute(pool).await.is_ok() {
        "更新成功".into()
    } else {
        "更新失败".into()
    }
}

fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    value: &Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    if let Some(v) = value.as_i64() {
        query.bind(v)
    } else if let Some(v) = value.as_u64() {
        query.bind(v)
    } else if let Some(v) = value.as_f64() {
        query.bind(v)
    } else if let Some(v) = value.as_bool() {
        query.bind(v)
    } else {
        query.bind(value.as_str().unwrap_or_default().to_string())
    }
}

async fn fetch_json(state: &AppState, url: &str) -> Option<Value> {
    let text = state.http.get(url).send().await.ok()?.text().await.ok()?;
    serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()
}

fn value_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(v) => Some(if *v { "1" } else { "0" }.to_string()),
        _ => None,
    }
}

fn quote_table(table: &str) -> Option<String> {
    let table = table.trim();
    if table.is_empty()
        || table
            .chars()
            .any(|c| c == '`' || c == ';' || c.is_control())
    {
        None
    } else {
        Some(format!("`{table}`"))
    }
}

fn quote_ident(column: &str) -> String {
    format!("`{}`", column.replace('`', ""))
}

fn text(body: impl Into<String>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        body.into(),
    )
        .into_response()
}
