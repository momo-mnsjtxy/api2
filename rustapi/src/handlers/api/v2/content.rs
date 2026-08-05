#![allow(non_snake_case)]

use axum::response::Response;
use rand::seq::SliceRandom;
use serde_json::{json, Value};

use super::{
    between, cache_key, err, http_json, http_post_text, http_text, int, json_with_msg, ok,
    proxy_bytes, rand_hex_color, random_row, random_text_table, read_public_text, redirect,
    row_json, row_string, today_key, vclone, ApiCtx,
};
use crate::respond;

pub async fn netease(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "netease").await;
    match random_row(&ctx, "hot").await {
        Some(row) => {
            let mp3 = row_string(&row, "mp3_url").replace(".mp3", "");
            let song_id = mp3.rsplit('=').next().unwrap_or("").to_string();
            ok(
                &ctx,
                json!({
                    "id": row_json(&row, "id"),
                    "song_id": song_id,
                    "name": row_json(&row, "name"),
                    "images": row_json(&row, "images"),
                    "author": row_json(&row, "author"),
                    "mp3_url": mp3,
                    "comment_nickname": row_json(&row, "comment_nickname"),
                    "comment_content": row_json(&row, "comment_content"),
                    "love": row_json(&row, "love"),
                }),
            )
        }
        None => err(&ctx, 400, "未查询到数据"),
    }
}

pub async fn love(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "love").await;
    random_text_table(&ctx, "love", "word", true).await
}

pub async fn Name(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Name").await;
    random_text_table(&ctx, "name", "word", true).await
}

pub async fn autograph(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "autograph").await;
    random_text_table(&ctx, "autograph", "word", true).await
}

pub async fn shuoshuo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "shuoshuo").await;
    random_text_table(&ctx, "shuoshuo", "word", true).await
}

pub async fn word(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "word").await;
    random_text_table(&ctx, "word", "text", false).await
}

pub async fn dog(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "dog").await;
    match random_row(&ctx, "dog").await {
        Some(row) => ok(&ctx, json!({"text": row_json(&row, "text")})),
        None => err(&ctx, 400, "未查询到数据"),
    }
}

pub async fn HeadImg(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "HeadImg").await;
    let table = ctx.param_or("kw", "女生头像");
    match random_row(&ctx, table).await {
        Some(row) => {
            let imgurl = row_string(&row, "url");
            if imgurl.is_empty() {
                err(&ctx, 400, "未查询到数据")
            } else {
                proxy_bytes(&ctx, &imgurl, "image/png").await
            }
        }
        None => err(&ctx, 400, "未查询到数据"),
    }
}

pub async fn today(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "today").await;
    let key = today_key("lishishangdejintian");
    if let Some(cached) = ctx.state.cache.get(&key).await {
        if let Ok(value) = serde_json::from_str::<Value>(&cached) {
            return ok(&ctx, value);
        }
    }

    let Some(html) = http_text(&ctx, "https://lishishangdejintian.51240.com/").await else {
        return err(&ctx, 400, "请求失败");
    };
    let mut results = Vec::new();
    for item in html.split("<li").skip(1).take(30) {
        let li = item.split("</li>").next().unwrap_or(item);
        let href = between(li, "href=\"", "\"").unwrap_or("");
        let title = strip_tags(between(li, ">", "</a>").unwrap_or(""))
            .trim()
            .to_string();
        let today = strip_tags(li).replace(&title, "").trim().to_string();
        let url = format!("https://lishishangdejintian.51240.com/{href}");
        let body = if let Some(detail) = http_text(&ctx, &url).await {
            let body_html = between(&detail, "<div class=\"neirong\">", "</div>").unwrap_or("");
            strip_tags(body_html).replace("更多：https://www.51240.com/", "")
        } else {
            String::new()
        };
        results.push(json!({"today": today, "title": title, "body": body}));
    }

    let value = Value::Array(results);
    let _ = ctx.state.cache.set(&key, value.to_string(), 86_400).await;
    ok(&ctx, value)
}

pub async fn du_word(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "du_word").await;
    let Some(time) = ctx.param("time") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!(
        "http://www.dutangapp.cn/u/toxic?date={}",
        urlencoding::encode(time)
    );
    match http_json(&ctx, &url).await {
        Some(v) => int(&ctx, json_with_msg(v, "查询成功")),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn xiaohua(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "xiaohua").await;
    let url = "http://i.itpk.cn/api.php?question=%E7%AC%91%E8%AF%9D&api_key=c7c0fe42b152684bc6971e881dbba254&api_secret=2q9k54fd2ufs";
    match http_post_text(&ctx, url).await {
        Some(text) => {
            let value = super::parse_json_loose(&text).unwrap_or_else(|| json!({"msg": text}));
            ok(&ctx, value)
        }
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn chengyu(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "chengyu").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!(
        "http://i.itpk.cn/api.php?question=@cy{}&api_key=c7c0fe42b152684bc6971e881dbba254&api_secret=2q9k54fd2ufs",
        urlencoding::encode(text)
    );
    match http_post_text(&ctx, &url).await {
        Some(msg) if !msg.is_empty() => ok(&ctx, json!({"msg": msg})),
        _ => err(&ctx, 400, "请求失败"),
    }
}

pub async fn colors(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "colors").await;
    let color = rand_hex_color();
    let r = u8::from_str_radix(&color[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&color[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&color[4..6], 16).unwrap_or(0);
    ok(
        &ctx,
        json!({"color": color, "RGB": {"r": r, "g": g, "b": b}}),
    )
}

pub async fn content_image_from_list(ctx: &ApiCtx, file: &str) -> Response {
    let Some(raw) = read_public_text(ctx, file).await else {
        return err(ctx, 400, "数据文件不存在");
    };
    let urls: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();
    if urls.is_empty() {
        return err(ctx, 400, "未查询到数据");
    }
    let url = *urls.choose(&mut rand::thread_rng()).unwrap();
    match ctx.param("type") {
        Some("url") => redirect(url),
        Some("image") => proxy_bytes(ctx, url, "image/png").await,
        _ => ok(ctx, json!({"url": url})),
    }
}

pub(crate) async fn huluxia_image(ctx: &ApiCtx) -> Response {
    let endpoint = "http://floor.huluxia.com/post/list/ANDROID/2.1?platform=2&gkey=000000&app_version=3.5.0.86.1&versioncode=20141391&market_id=floor_baidu&_key=D9417A952C422C763CB47A07D0AEE76E5BDF7BB8DA6C7C6120DFFE260FD2B8C95E8EA7EE26C85FD2151293C3FF1CB9F967533BEF008BC875&device_code=%5Bw%5Dd4%3A61%3A2e%3Aa0%3Ac7%3Ade-%5Bi%5D861918033679755-%5Bs%5D89860619100072454095&start=0&count=200&cat_id=56&tag_id=5601&sort_by=1";
    let Some(v) = http_json(ctx, endpoint).await else {
        return err(ctx, 400, "请求失败");
    };
    let mut urls = Vec::new();
    if let Some(posts) = v.get("posts").and_then(Value::as_array) {
        for post in posts {
            if let Some(images) = post.get("images").and_then(Value::as_array) {
                for img in images {
                    if let Some(url) = img.as_str() {
                        urls.push(url.to_string());
                    }
                }
            }
        }
    }
    let Some(url) = urls.choose(&mut rand::thread_rng()).cloned() else {
        return err(ctx, 400, "未查询到数据");
    };
    match ctx.param("type") {
        Some("url") => redirect(&format!("https://api.gqink.cn/api/v2/image?url={url}")),
        Some("image") => proxy_bytes(ctx, &url, "image/png").await,
        _ => ok(
            ctx,
            json!({"url": format!("https://api.gqink.cn/api/v2/image?url={url}")}),
        ),
    }
}

pub(crate) async fn amap_cached(ctx: &ApiCtx, key: String, url: String, pointer: &str) -> Response {
    if let Some(cached) = ctx.state.cache.get(&key).await {
        if let Ok(v) = serde_json::from_str::<Value>(&cached) {
            return ok(ctx, v);
        }
    }
    match http_json(ctx, &url).await {
        Some(v) => {
            let data = if pointer.is_empty() {
                v
            } else {
                vclone(&v, pointer)
            };
            let _ = ctx.state.cache.set(&key, data.to_string(), 43_200).await;
            ok(ctx, data)
        }
        None => err(ctx, 400, "请求失败"),
    }
}

pub(crate) fn err_api() -> Value {
    json!({"code": 400, "msg": "接口错误", "api": ""})
}

fn strip_tags(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}
