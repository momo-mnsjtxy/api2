use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, Method},
    response::Response,
};
use chrono::{Local, TimeZone};
use serde_json::{json, Map, Value};
use sqlx::{mysql::MySqlRow, Column, MySqlPool, Row};

use crate::{db, respond, state::AppState, util};

type Params = HashMap<String, String>;

const PAGE_SIZE: i64 = 20;

pub async fn index(Query(params): Query<Params>) -> Response {
    respond::int(
        output_type(&params),
        json!({"code": 400, "msg": "接口错误", "api": ""}),
    )
}

pub async fn netease(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    content_page(state, params, headers, method, "netease", "hot", false).await
}

pub async fn autograph(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    content_page(
        state,
        params,
        headers,
        method,
        "autograph",
        "autograph",
        false,
    )
    .await
}

pub async fn love(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    content_page(state, params, headers, method, "love", "love", true).await
}

pub async fn name(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    content_page(state, params, headers, method, "name", "name", false).await
}

pub async fn shuoshuo(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    content_page(
        state, params, headers, method, "shuoshuo", "shuoshuo", false,
    )
    .await
}

pub async fn word(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    content_page(state, params, headers, method, "word", "word", false).await
}

pub async fn log(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    if params.get("page").map(String::as_str) == Some("day") {
        let data = match log_stats(&state.pool).await {
            Ok(v) => v,
            Err(_) => json!({"code": 400, "msg": "内部错误"}),
        };
        return respond::int(output_type(&params), data);
    }

    let page = page_param(&params);
    let body = match paged_rows(&state.pool, "log", page, PAGE_SIZE).await {
        Ok((count, rows)) => {
            let data = rows
                .iter()
                .map(|row| {
                    let ts = get_i64(row, "time").unwrap_or_default();
                    json!({
                        "id": get_i64(row, "id").unwrap_or_default(),
                        "name": get_string(row, "name").unwrap_or_default(),
                        "ip": mask_ip(&get_string(row, "ip").unwrap_or_default()),
                        "time": format_ts(ts),
                        "request": get_string(row, "request").unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>();
            page_payload(count, page, data)
        }
        Err(_) => json!({"code": 400, "msg": "内部错误"}),
    };
    respond::int(output_type(&params), body)
}

pub async fn likes(State(state): State<AppState>, Query(params): Query<Params>) -> Response {
    let page = page_param(&params);
    let body = match paged_rows(&state.pool, "likes", page, PAGE_SIZE).await {
        Ok((count, rows)) => {
            let data = rows
                .iter()
                .map(|row| {
                    json!({
                        "id": get_i64(row, "id").unwrap_or_default(),
                        "cid": get_i64(row, "cid").unwrap_or_default(),
                        "keyword": get_string(row, "keyword").unwrap_or_default(),
                        "time": get_i64(row, "time").unwrap_or_default(),
                        "ip": mask_ip(&get_string(row, "ip").unwrap_or_default()),
                    })
                })
                .collect::<Vec<_>>();
            page_payload(count, page, data)
        }
        Err(_) => json!({"code": 400, "msg": "内部错误"}),
    };
    respond::int(output_type(&params), body)
}

pub async fn headimg(
    State(state): State<AppState>,
    Query(params): Query<Params>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    log_access(&state.pool, "headimg", &headers, &method).await;
    let table = params
        .get("kw")
        .filter(|kw| !kw.trim().is_empty())
        .map(String::as_str)
        .unwrap_or("女生头像");
    let page = page_param(&params);
    let body = match paged_rows(&state.pool, table, page, PAGE_SIZE).await {
        Ok((count, rows)) => {
            let data = rows
                .iter()
                .map(|row| {
                    json!({
                        "id": get_i64(row, "id").unwrap_or_default(),
                        "url": get_string(row, "url").unwrap_or_default(),
                    })
                })
                .collect::<Vec<_>>();
            page_payload(count, page, data)
        }
        Err(_) => json!({"code": 400, "msg": "内部错误"}),
    };
    respond::int(output_type(&params), body)
}

async fn content_page(
    state: AppState,
    params: Params,
    headers: HeaderMap,
    method: Method,
    log_kw: &'static str,
    table: &'static str,
    random_index: bool,
) -> Response {
    log_access(&state.pool, log_kw, &headers, &method).await;

    if params.get("page").map(String::as_str) == Some("love") {
        let ip = util::client_ip(&headers, None);
        let body = like_row(
            &state.pool,
            table,
            params.get("id").map(String::as_str),
            &ip,
        )
        .await;
        return respond::int(output_type(&params), body);
    }

    if random_index && params.get("page").map(String::as_str) == Some("index") {
        let body = match random_love_rows(&state.pool).await {
            Ok(rows) => json!({
                "code": 200,
                "Copyright": respond::copyright(),
                "data": rows_to_json(rows),
            }),
            Err(_) => json!({"code": 400, "msg": "内部错误"}),
        };
        return respond::int(output_type(&params), body);
    }

    let page = page_param(&params);
    let body = match paged_rows(&state.pool, table, page, PAGE_SIZE).await {
        Ok((count, rows)) => page_payload(count, page, rows_to_json(rows)),
        Err(_) => json!({"code": 400, "msg": "内部错误"}),
    };
    respond::int(output_type(&params), body)
}

async fn log_access(pool: &MySqlPool, keyword: &str, headers: &HeaderMap, method: &Method) {
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let ip = util::client_ip(headers, None);
    let _ = db::insert_log(pool, keyword, ua, method.as_str(), &ip).await;
}

async fn like_row(pool: &MySqlPool, table: &str, id: Option<&str>, ip: &str) -> Value {
    let id = match id.and_then(|raw| raw.parse::<i64>().ok()) {
        Some(id) if id > 0 => id,
        _ => return json!({"code": "400", "msg": "ID不存在"}),
    };
    let table_sql = match quote_table(table) {
        Some(table) => table,
        None => return json!({"code": "400", "msg": "ID不存在"}),
    };

    let find_sql = format!("SELECT id, love FROM {table_sql} WHERE id=? LIMIT 1");
    let existing = match sqlx::query(&find_sql).bind(id).fetch_optional(pool).await {
        Ok(row) => row,
        Err(_) => return json!({"code": "400", "msg": "点赞失败", "love": 0}),
    };
    let Some(existing) = existing else {
        return json!({"code": "400", "msg": "ID不存在"});
    };
    let old_love = get_i64(&existing, "love").unwrap_or_default();

    let liked = sqlx::query("SELECT id FROM likes WHERE cid=? AND ip=? AND keyword=? LIMIT 1")
        .bind(id)
        .bind(ip)
        .bind(table)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if liked {
        return json!({"code": "201", "msg": "已点赞过了", "love": old_love});
    }

    let inserted = sqlx::query("INSERT INTO likes (cid, keyword, time, ip) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(table)
        .bind(util::now_ts())
        .bind(ip)
        .execute(pool)
        .await
        .is_ok();
    let update_sql = format!("UPDATE {table_sql} SET love=COALESCE(love, 0)+1 WHERE id=?");
    let updated = sqlx::query(&update_sql)
        .bind(id)
        .execute(pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false);

    if inserted && updated {
        let new_love = sqlx::query(&find_sql)
            .bind(id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
            .and_then(|row| get_i64(&row, "love"))
            .unwrap_or(old_love + 1);
        json!({"code": "200", "msg": "点赞成功", "love": new_love})
    } else {
        json!({"code": "400", "msg": "点赞失败", "love": old_love})
    }
}

async fn random_love_rows(pool: &MySqlPool) -> sqlx::Result<Vec<MySqlRow>> {
    let count = count_table(pool, "love").await?.max(0);
    let page_count = ((count + 9) / 10).max(1);
    let page = rand::random::<i64>().rem_euclid(page_count) + 1;
    let offset = (page - 1) * 10;
    sqlx::query("SELECT * FROM `love` ORDER BY id DESC LIMIT 10 OFFSET ?")
        .bind(offset)
        .fetch_all(pool)
        .await
}

async fn log_stats(pool: &MySqlPool) -> sqlx::Result<Value> {
    let (today_start, today_end) = util::range_today();
    let (yesterday_start, yesterday_end) = util::range_yesterday();
    let (week_start, week_end) = util::range_this_week();
    let (month_start, month_end) = util::range_this_month();

    Ok(json!({
        "count": count_table(pool, "log").await?,
        "week": count_between(pool, "log", "time", week_start, week_end).await?,
        "month": count_between(pool, "log", "time", month_start, month_end).await?,
        "today": count_between(pool, "log", "time", today_start, today_end).await?,
        "yesterday": count_between(pool, "log", "time", yesterday_start, yesterday_end).await?,
        "day2": count_day_offset(pool, 2).await?,
        "day3": count_day_offset(pool, 3).await?,
        "day4": count_day_offset(pool, 4).await?,
        "day5": count_day_offset(pool, 5).await?,
        "day6": count_day_offset(pool, 6).await?,
    }))
}

async fn count_day_offset(pool: &MySqlPool, days_ago: i64) -> sqlx::Result<i64> {
    let (start, end) = util::day_range_offset(days_ago);
    count_between(pool, "log", "time", start, end).await
}

async fn count_between(
    pool: &MySqlPool,
    table: &str,
    field: &str,
    start: i64,
    end: i64,
) -> sqlx::Result<i64> {
    let sql = format!(
        "SELECT COUNT(*) AS c FROM {} WHERE `{}` BETWEEN ? AND ?",
        quote_table(table).unwrap_or_else(|| "`log`".to_string()),
        field.replace('`', "")
    );
    let row = sqlx::query(&sql)
        .bind(start)
        .bind(end)
        .fetch_one(pool)
        .await?;
    Ok(row.try_get::<i64, _>("c").unwrap_or(0))
}

async fn count_table(pool: &MySqlPool, table: &str) -> sqlx::Result<i64> {
    let sql = format!(
        "SELECT COUNT(*) AS c FROM {}",
        quote_table(table).unwrap_or_else(|| "`log`".to_string())
    );
    let row = sqlx::query(&sql).fetch_one(pool).await?;
    Ok(row.try_get::<i64, _>("c").unwrap_or(0))
}

async fn paged_rows(
    pool: &MySqlPool,
    table: &str,
    page: i64,
    page_size: i64,
) -> sqlx::Result<(i64, Vec<MySqlRow>)> {
    let table_sql = quote_table(table).ok_or(sqlx::Error::RowNotFound)?;
    let count = count_table(pool, table).await?;
    let offset = (page.max(1) - 1) * page_size;
    let sql = format!("SELECT * FROM {table_sql} ORDER BY id DESC LIMIT ? OFFSET ?");
    let rows = sqlx::query(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok((count, rows))
}

fn page_payload(count: i64, page: i64, data: Vec<Value>) -> Value {
    json!({
        "code": 200,
        "Copyright": respond::copyright(),
        "conf": {
            "count": count,
            "PageCount": ((count + PAGE_SIZE - 1) / PAGE_SIZE).max(0),
            "Page": page,
        },
        "data": data,
    })
}

fn rows_to_json(rows: Vec<MySqlRow>) -> Vec<Value> {
    rows.iter().map(row_to_json).collect()
}

fn row_to_json(row: &MySqlRow) -> Value {
    let mut map = Map::new();
    for column in row.columns() {
        let name = column.name();
        map.insert(name.to_string(), cell_to_json(row, name));
    }
    Value::Object(map)
}

fn cell_to_json(row: &MySqlRow, column: &str) -> Value {
    if let Ok(v) = row.try_get::<Option<i64>, _>(column) {
        return v.map_or(Value::Null, |v| json!(v));
    }
    if let Ok(v) = row.try_get::<Option<u64>, _>(column) {
        return v.map_or(Value::Null, |v| json!(v));
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(column) {
        return v.map_or(Value::Null, |v| json!(v));
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(column) {
        return v.map_or(Value::Null, Value::String);
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(column) {
        return v
            .map(|bytes| Value::String(String::from_utf8_lossy(&bytes).to_string()))
            .unwrap_or(Value::Null);
    }
    Value::Null
}

fn get_string(row: &MySqlRow, column: &str) -> Option<String> {
    row.try_get::<Option<String>, _>(column)
        .ok()
        .flatten()
        .or_else(|| get_i64(row, column).map(|v| v.to_string()))
}

fn get_i64(row: &MySqlRow, column: &str) -> Option<i64> {
    row.try_get::<Option<i64>, _>(column).ok().flatten()
}

fn page_param(params: &Params) -> i64 {
    params
        .get("page")
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1)
}

fn output_type(params: &Params) -> Option<&str> {
    params.get("type").map(String::as_str)
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

fn mask_ip(ip: &str) -> String {
    let parts = ip.split('.').collect::<Vec<_>>();
    if parts.len() == 4 && parts.iter().all(|part| part.parse::<u8>().is_ok()) {
        format!("{}.{}.{}.*", parts[0], parts[1], parts[2])
    } else {
        ip.to_string()
    }
}

fn format_ts(ts: i64) -> String {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}
