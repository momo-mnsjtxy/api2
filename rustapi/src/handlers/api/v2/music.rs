#![allow(non_snake_case)]

use axum::response::Response;
use serde_json::{json, Value};

use super::{cache_key, err, http_json, int, ok, read_public_text, vclone, vstr, ApiCtx};
use crate::{respond, services};

pub async fn Music_163(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Music_163").await;
    let Some(id) = ctx.param("id") else {
        return err(&ctx, 400, "参数错误");
    };
    match services::music::m_163(&ctx.state.http, id).await {
        Some(v) => {
            let data = normalize_163(id, v);
            if ctx.param("type") == Some("lrc") {
                super::text_response("text/html; charset=utf-8", vstr(&data, "/lrc"))
            } else {
                ok(&ctx, data)
            }
        }
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn Music(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "music").await;
    let input = ctx.param_or("input", "").trim();
    let class = ctx.param_or("class", "name");
    let site = ctx.param_or("site", "");
    let page = ctx.param_or("page", "0");
    let value = services::music::music_search(&ctx.state.http, input, site, page).await;
    let code = vstr(&value, "/code");
    let data = value.get("data").cloned().unwrap_or(Value::Null);
    let wrapped = if data.is_array() {
        json!({"code": if code.is_empty() { "success" } else { code.as_str() }, "msg": "搜索成功", "data": data})
    } else {
        json!({"code": if code.is_empty() { "error" } else { code.as_str() }, "msg": vstr(&value, "/error")})
    };
    ok(&ctx, wrapped)
}

pub async fn Music_hot(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Music_hot").await;
    match read_public_text(&ctx, "hot.json")
        .await
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
    {
        Some(v) => ok(&ctx, v),
        None => err(&ctx, 400, "数据文件不存在"),
    }
}

pub async fn Music_List_163(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Music_List").await;
    let Some(id) = ctx.param("id") else {
        return err(&ctx, 400, "参数错误");
    };
    let key = cache_key("Music_List_ID_163", id);
    if let Some(cached) = ctx.state.cache.get(&key).await {
        if let Ok(v) = serde_json::from_str::<Value>(&cached) {
            return ok(&ctx, v);
        }
    }
    let url = format!("https://music.163.com/api/playlist/detail?id={id}");
    match http_json(&ctx, &url).await {
        Some(v) => {
            let data = v.pointer("/result/tracks").cloned().unwrap_or(v);
            let _ = ctx.state.cache.set(&key, data.to_string(), 43_200).await;
            ok(&ctx, data)
        }
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn Kugou(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Kugou").await;
    provider_by_id(ctx, "kugou", "找不到可用的播放地址").await
}

pub async fn qq(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "qq").await;
    provider_by_id(ctx, "qq", "找不到可用的播放地址").await
}

pub async fn kuwo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "kuwo").await;
    provider_by_id(ctx, "kuwo", "找不到可用的播放地址").await
}

pub async fn xiami(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "xiami").await;
    provider_by_id(ctx, "xiami", "找不到可用的播放地址").await
}

pub async fn migu(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "migu").await;
    provider_by_id(ctx, "migu", "找不到可用的播放地址").await
}

pub async fn kg(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "kg").await;
    provider_by_id(ctx, "kg", "找不到可用的播放地址").await
}

pub async fn MusicUrl(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "MusicUrl").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let value = services::music::music_search(&ctx.state.http, url, "_", "1").await;
    provider_response(&ctx, value, "找不到可用的播放地址(推荐POST请求)")
}

async fn provider_by_id(ctx: ApiCtx, provider: &str, fail_msg: &str) -> Response {
    let Some(id) = ctx.param("id") else {
        return err(&ctx, 400, "参数错误");
    };
    let value = services::music::music_search(&ctx.state.http, id, provider, "1").await;
    provider_response(&ctx, value, fail_msg)
}

fn provider_response(ctx: &ApiCtx, value: Value, fail_msg: &str) -> Response {
    if vstr(&value, "/code") == "success" {
        let data = value
            .pointer("/data/0")
            .cloned()
            .or_else(|| value.get("data").cloned())
            .unwrap_or(Value::Null);
        ok(ctx, data)
    } else {
        ok(ctx, json!({"code": "400", "msg": fail_msg}))
    }
}

fn normalize_163(id: &str, raw: Value) -> Value {
    let author = vstr(&raw, "/artists");
    let lrc = {
        let lrc = choose(&raw, &["/lrc"]);
        if lrc.is_empty() {
            "此音乐暂无歌词".to_string()
        } else {
            lrc
        }
    };
    json!({
        "song_id": id,
        "name": choose(&raw, &["/name", "/song_name"]),
        "author": author,
        "song_zj": choose(&raw, &["/song_zj"]),
        "url": choose(&raw, &["/url"]),
        "cover": choose(&raw, &["/cover", "/pic"]),
        "artists_img1v1Url": choose(&raw, &["/artists_img1v1Url"]),
        "lrc": lrc,
    })
}

fn choose(raw: &Value, paths: &[&str]) -> String {
    paths
        .iter()
        .map(|p| vstr(raw, p))
        .find(|s| !s.is_empty())
        .unwrap_or_default()
}
