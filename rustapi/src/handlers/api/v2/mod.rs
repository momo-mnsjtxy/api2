#![allow(non_snake_case)]

pub mod content;
pub mod media;
pub mod misc;
pub mod music;
pub mod tools;
pub mod video;

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Local;
use serde_json::{json, Value};
use sqlx::{mysql::MySqlRow, Row};
use std::path::{Path, PathBuf};

use super::ApiCtx;
use crate::{db::insert_log, respond};

pub async fn dispatch(action: &str, ctx: ApiCtx) -> Response {
    match action {
        "netease" => content::netease(ctx).await,
        "love" => content::love(ctx).await,
        "Name" => content::Name(ctx).await,
        "autograph" => content::autograph(ctx).await,
        "shuoshuo" => content::shuoshuo(ctx).await,
        "word" => content::word(ctx).await,
        "dog" => content::dog(ctx).await,
        "HeadImg" => content::HeadImg(ctx).await,
        "today" => content::today(ctx).await,
        "du_word" => content::du_word(ctx).await,
        "xiaohua" => content::xiaohua(ctx).await,
        "chengyu" => content::chengyu(ctx).await,
        "colors" => content::colors(ctx).await,

        "MD5" => tools::MD5(ctx).await,
        "QrCode" => tools::QrCode(ctx).await,
        "QrReader" => tools::QrReader(ctx).await,
        "Mobile" => tools::Mobile(ctx).await,
        "whois" => tools::whois(ctx).await,
        "pinyin" => tools::pinyin(ctx).await,
        "fanyi" => tools::fanyi(ctx).await,
        "ping" => tools::ping(ctx).await,
        "robot" => tools::robot(ctx).await,
        "weather" => tools::weather(ctx).await,
        "WeatherInfo" => tools::WeatherInfo(ctx).await,
        "laji" => tools::laji(ctx).await,
        "address" => tools::address(ctx).await,
        "check" => tools::check(ctx).await,
        "check_domain" => tools::check_domain(ctx).await,
        "Url_Sec" => tools::Url_Sec(ctx).await,
        "HttpCode" => tools::HttpCode(ctx).await,
        "RandPass" => tools::RandPass(ctx).await,
        "Go" => tools::Go(ctx).await,
        "district" => tools::district(ctx).await,
        "IpSadd" => tools::IpSadd(ctx).await,
        "QQChat" => tools::QQChat(ctx).await,
        "QQInfo" => tools::QQInfo(ctx).await,
        "idcard" => tools::idcard(ctx).await,
        "telxj" => tools::telxj(ctx).await,
        "qqxj" => tools::qqxj(ctx).await,

        "UserInfo" => media::UserInfo(ctx).await,
        "Url" => media::Url(ctx).await,
        "qlogo" => media::qlogo(ctx).await,
        "Gravatar" => media::Gravatar(ctx).await,
        "Bing_img" => media::Bing_img(ctx).await,
        "image" => media::image(ctx).await,
        "IP" => media::IP(ctx).await,
        "IP_IMG" => media::IP_IMG(ctx).await,
        "Email" => media::Email(ctx).await,
        "ICP" => media::ICP(ctx).await,
        "DM_IMG" => media::DM_IMG(ctx).await,
        "C_IMG" => media::C_IMG(ctx).await,
        "audio" => media::audio(ctx).await,
        "qunlogo" => media::qunlogo(ctx).await,
        "upload" => media::upload(ctx).await,
        "Baidu_Upload" => media::Baidu_Upload(ctx).await,
        "Sogou_Upload" => media::Sogou_Upload(ctx).await,
        "proxy" => media::proxy(ctx).await,

        "Music_163" => music::Music_163(ctx).await,
        "Music" => music::Music(ctx).await,
        "Music_hot" => music::Music_hot(ctx).await,
        "Music_List_163" => music::Music_List_163(ctx).await,
        "Kugou" => music::Kugou(ctx).await,
        "qq" => music::qq(ctx).await,
        "kuwo" => music::kuwo(ctx).await,
        "xiami" => music::xiami(ctx).await,
        "migu" => music::migu(ctx).await,
        "kg" => music::kg(ctx).await,
        "MusicUrl" => music::MusicUrl(ctx).await,

        "douyin" => video::douyin(ctx).await,
        "kuaishou" => video::kuaishou(ctx).await,
        "weishi" => video::weishi(ctx).await,
        "pipixia" => video::pipixia(ctx).await,
        "miaopai" => video::miaopai(ctx).await,
        _ => misc::index(ctx).await,
    }
}

pub(crate) async fn log_action(ctx: &ApiCtx, name: &str) {
    let _ = insert_log(&ctx.state.pool, name, &ctx.ua, &ctx.method, &ctx.ip).await;
}

pub(crate) fn int(ctx: &ApiCtx, data: Value) -> Response {
    respond::int(ctx.output_type(), data)
}

pub(crate) fn ok(ctx: &ApiCtx, data: Value) -> Response {
    int(ctx, respond::ok_data(data))
}

pub(crate) fn err(ctx: &ApiCtx, code: i64, msg: &str) -> Response {
    int(ctx, respond::err_msg(code, msg))
}

pub(crate) fn err_value(code: i64, msg: &str) -> Value {
    respond::err_msg(code, msg)
}

pub(crate) fn json_ok(data: Value) -> Value {
    respond::ok_data(data)
}

pub(crate) fn json_with_msg(data: Value, msg: &str) -> Value {
    json!({
        "code": 200,
        "msg": msg,
        "Copyright": respond::copyright(),
        "data": data,
    })
}

pub(crate) fn bytes_response(content_type: &'static str, bytes: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        bytes,
    )
        .into_response()
}

pub(crate) fn text_response(content_type: &'static str, body: impl Into<String>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        body.into(),
    )
        .into_response()
}

pub(crate) fn redirect(location: &str) -> Response {
    let mut res = StatusCode::FOUND.into_response();
    if let Ok(value) = HeaderValue::from_str(location) {
        res.headers_mut().insert(header::LOCATION, value);
    }
    res
}

pub(crate) async fn proxy_bytes(ctx: &ApiCtx, url: &str, content_type: &'static str) -> Response {
    match http_bytes(ctx, url).await {
        Some(bytes) => bytes_response(content_type, bytes),
        None => err(ctx, 400, "请求失败"),
    }
}

pub(crate) async fn http_json(ctx: &ApiCtx, url: &str) -> Option<Value> {
    for _ in 0..10 {
        let resp = ctx.state.http.get(url).send().await.ok()?;
        let text = resp.text().await.ok()?;
        if let Some(v) = parse_json_loose(&text) {
            return Some(v);
        }
    }
    None
}

pub(crate) async fn http_post_json(ctx: &ApiCtx, url: &str) -> Option<Value> {
    for _ in 0..10 {
        let resp = ctx.state.http.post(url).send().await.ok()?;
        let text = resp.text().await.ok()?;
        if let Some(v) = parse_json_loose(&text) {
            return Some(v);
        }
    }
    None
}

pub(crate) async fn http_text(ctx: &ApiCtx, url: &str) -> Option<String> {
    ctx.state.http.get(url).send().await.ok()?.text().await.ok()
}

pub(crate) async fn http_post_text(ctx: &ApiCtx, url: &str) -> Option<String> {
    ctx.state
        .http
        .post(url)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()
}

pub(crate) async fn http_bytes(ctx: &ApiCtx, url: &str) -> Option<Vec<u8>> {
    ctx.state
        .http
        .get(url)
        .send()
        .await
        .ok()?
        .bytes()
        .await
        .ok()
        .map(|b| b.to_vec())
}

pub(crate) async fn head_status(ctx: &ApiCtx, url: &str) -> Option<u16> {
    let resp = ctx.state.http.head(url).send().await.ok()?;
    Some(resp.status().as_u16())
}

pub(crate) fn parse_json_loose(text: &str) -> Option<Value> {
    let trimmed = text.trim_start_matches('\u{feff}').trim();
    serde_json::from_str(trimmed)
        .ok()
        .or_else(|| serde_json::from_str(strip_jsonp(trimmed).as_str()).ok())
}

pub(crate) fn strip_jsonp(text: &str) -> String {
    let trimmed = text.trim();
    if (trimmed.starts_with('{') || trimmed.starts_with('[')) && !trimmed.ends_with(");") {
        return trimmed.to_string();
    }
    match (trimmed.find('('), trimmed.rfind(')')) {
        (Some(start), Some(end)) if end > start => trimmed[start + 1..end].to_string(),
        _ => trimmed.to_string(),
    }
}

pub(crate) fn vstr(v: &Value, pointer: &str) -> String {
    match v.pointer(pointer) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

pub(crate) fn vclone(v: &Value, pointer: &str) -> Value {
    v.pointer(pointer).cloned().unwrap_or(Value::Null)
}

pub(crate) async fn random_row(ctx: &ApiCtx, table: &str) -> Option<MySqlRow> {
    let table = table.replace('`', "``");
    let sql = format!("SELECT * FROM `{table}` ORDER BY RAND() LIMIT 1");
    sqlx::query(&sql)
        .fetch_optional(&ctx.state.pool)
        .await
        .ok()
        .flatten()
}

pub(crate) fn row_json(row: &MySqlRow, col: &str) -> Value {
    if let Ok(v) = row.try_get::<i64, _>(col) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<u64, _>(col) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(col) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<String, _>(col) {
        return json!(v);
    }
    Value::Null
}

pub(crate) fn row_string(row: &MySqlRow, col: &str) -> String {
    match row_json(row, col) {
        Value::String(s) => s,
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

pub(crate) async fn random_text_table(
    ctx: &ApiCtx,
    table: &str,
    text_col: &str,
    with_keyword: bool,
) -> Response {
    match random_row(ctx, table).await {
        Some(row) => {
            let mut data = serde_json::Map::new();
            data.insert("id".into(), row_json(&row, "id"));
            data.insert("text".into(), row_json(&row, text_col));
            if with_keyword {
                data.insert("keyword".into(), row_json(&row, "keyword"));
                data.insert("love".into(), row_json(&row, "love"));
            } else if row_string(&row, "love").is_empty() {
                data.insert("love".into(), Value::Null);
            } else {
                data.insert("love".into(), row_json(&row, "love"));
            }
            ok(ctx, Value::Object(data))
        }
        None => err(ctx, 400, "未查询到数据"),
    }
}

pub(crate) fn public_paths(ctx: &ApiCtx, name: &str) -> Vec<PathBuf> {
    vec![
        Path::new(&ctx.state.config.public_dir).join(name),
        Path::new("/workspace/rustapi/public").join(name),
        Path::new("/workspace/oldapi/public").join(name),
    ]
}

pub(crate) async fn read_public_file(ctx: &ApiCtx, name: &str) -> Option<Vec<u8>> {
    for path in public_paths(ctx, name) {
        if let Ok(bytes) = tokio::fs::read(path).await {
            return Some(bytes);
        }
    }
    None
}

pub(crate) async fn read_public_text(ctx: &ApiCtx, name: &str) -> Option<String> {
    for path in public_paths(ctx, name) {
        if let Ok(text) = tokio::fs::read_to_string(path).await {
            return Some(text);
        }
    }
    None
}

pub(crate) fn file_ext(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
}

pub(crate) fn is_image_file(name: &str, bytes: &[u8], max: usize) -> bool {
    matches!(file_ext(name).as_str(), "gif" | "jpeg" | "jpg" | "png") && bytes.len() < max
}

pub(crate) fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let rest = &text[from..];
    let to = rest.find(end)?;
    Some(&rest[..to])
}

pub(crate) fn collapse_html(s: &str) -> String {
    s.replace("\r\n", "")
        .replace('\n', "")
        .replace('\t', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn clean_video_title(title: &str) -> String {
    title
        .split_whitespace()
        .filter(|part| !part.starts_with('@') && !part.starts_with('#'))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn cache_key(prefix: &str, value: &str) -> String {
    format!("{prefix}{value}")
}

pub(crate) fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub(crate) fn today_key(prefix: &str) -> String {
    format!("{prefix}{}", Local::now().format("%Y-%m-%d"))
}

pub(crate) fn rand_hex_color() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!(
        "{:02X}{:02X}{:02X}",
        rng.gen_range(0..=255),
        rng.gen_range(0..=255),
        rng.gen_range(0..=255)
    )
}

pub(crate) fn valid_http_url(url: &str) -> bool {
    matches!(url::Url::parse(url), Ok(u) if u.scheme() == "http" || u.scheme() == "https")
}
