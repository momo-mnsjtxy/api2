#![allow(non_snake_case)]

use axum::response::Response;
use serde_json::{json, Value};

use super::{between, clean_video_title, err, http_json, http_text, ok, vclone, vstr, ApiCtx};

pub async fn douyin(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "douyin").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let Some(html) = http_text(&ctx, url).await else {
        return err(&ctx, 400, "解析错误");
    };
    let play = between(&html, "playAddr: \"", "\"").unwrap_or("");
    let name = between(&html, "<p class=\"desc\">", "</p>").unwrap_or("");
    let auth = between(&html, "<p class=\"name nowrap\">", "</p>").unwrap_or("");
    if play.is_empty() {
        return err(&ctx, 400, "解析错误");
    }
    let Some(s_vid) = between(play, "s_vid=", "&") else {
        return err(&ctx, 400, "解析错误");
    };
    let api = format!("https://aweme.snssdk.com/aweme/v1/play/?s_vid={s_vid}&line=0");
    let Some(body) = http_text(&ctx, &api).await else {
        return err(&ctx, 400, "解析错误");
    };
    let Some(link) = between(&body, "<a href=\"http://", "\">") else {
        return err(&ctx, 400, "解析错误");
    };
    super::int(
        &ctx,
        json!({
            "code": "200",
            "Copyright": crate::respond::copyright(),
            "data": {"msg": "解析成功", "name": name, "auth": auth, "url": format!("http://{link}")}
        }),
    )
}

pub async fn kuaishou(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "kuaishou").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let Some(html) = http_text(&ctx, url).await else {
        return err(&ctx, 400, "解析错误");
    };
    let pagedata = between(&html, "id=\"hide-pagedata\" data-pagedata=\"", "\"")
        .or_else(|| between(&html, "data-pagedata='", "'"))
        .unwrap_or("");
    if pagedata.is_empty() {
        return err(&ctx, 400, "解析错误");
    }
    let page_json = html_unescape(pagedata);
    let Some(v) = super::parse_json_loose(&page_json) else {
        return err(&ctx, 400, "解析错误");
    };
    let photo_id = vstr(&v, "/photoId");
    if photo_id.is_empty() {
        return err(&ctx, 400, "解析错误");
    }
    let param = format!("client_key=56c3713c&photoIds={photo_id}");
    let sig = crate::util::md5_hex(&format!("{}23caab00356c", param.replace('&', "")));
    let api = format!("http://api.gifshow.com/rest/n/photo/info?{param}&sig={sig}");
    let Some(info) = http_json(&ctx, &api).await else {
        return err(&ctx, 400, "解析错误");
    };
    let video = info.pointer("/photos/0").unwrap_or(&Value::Null);
    ok(
        &ctx,
        json!({
            "url": vstr(video, "/main_mv_url"),
            "cover": vstr(video, "/thumbnail_url"),
            "title": clean_video_title(&vstr(video, "/caption")),
        }),
    )
}

pub async fn weishi(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "weishi").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let Some(feedid) = between(url, "feed/", "/wsfeed") else {
        return err(&ctx, 400, "解析错误");
    };
    let resp = ctx
        .state
        .http
        .post(
            "https://h5.qzone.qq.com/webapp/json/weishi/WSH5GetPlayPage?t=0.4185745904612037&g_tk=",
        )
        .form(&[("feedid", feedid)])
        .send()
        .await;
    let Ok(resp) = resp else {
        return err(&ctx, 400, "解析错误");
    };
    let text = resp.text().await.unwrap_or_default();
    let Some(v) = super::parse_json_loose(&text) else {
        return err(&ctx, 400, "解析错误");
    };
    if v.get("ret").and_then(Value::as_i64) != Some(0) {
        return err(&ctx, 400, "解析错误");
    }
    let video = v.pointer("/data/feeds/0").unwrap_or(&Value::Null);
    ok(
        &ctx,
        json!({
            "url": vstr(video, "/video_url"),
            "cover": vstr(video, "/images/0/url"),
            "title": clean_video_title(&vstr(video, "/feed_desc")),
            "width": vclone(video, "/images/0/width"),
            "height": vclone(video, "/images/0/height"),
        }),
    )
}

pub async fn pipixia(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "pipixia").await;
    let Some(input) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let resolved = ctx
        .state
        .http
        .get(input)
        .send()
        .await
        .ok()
        .map(|r| r.url().to_string())
        .unwrap_or_else(|| input.to_string());
    let Some(item_id) = between(&resolved, "item/", "?") else {
        return err(&ctx, 400, "解析错误");
    };
    let url = format!("https://h5.pipix.com/bds/webapi/item/detail/?item_id={item_id}");
    match http_json(&ctx, &url).await {
        Some(v) => ok(
            &ctx,
            json!({
                "title": vstr(&v, "/data/item/share/title"),
                "author": vstr(&v, "/data/item/author/name"),
                "url": vstr(&v, "/data/item/origin_video_download/url_list/0/url"),
            }),
        ),
        None => err(&ctx, 400, "解析错误"),
    }
}

pub async fn miaopai(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "miaopai").await;
    let Some(input) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let Some(smid) = between(input, "/show/", ".htm") else {
        return err(&ctx, 400, "解析错误");
    };
    let api =
        format!("http://n.miaopai.com/api/aj_media/info.json?smid={smid}&appid=530&_cb=_jsonp");
    let Some(text) = http_text(&ctx, &api).await else {
        return err(&ctx, 400, "解析错误");
    };
    let json_url = between(&text, "json\":\"", "\"")
        .unwrap_or("")
        .replace("\\/", "/");
    if json_url.is_empty() {
        return err(&ctx, 400, "解析错误");
    }
    match http_json(&ctx, &json_url).await {
        Some(v) => ok(
            &ctx,
            json!({"url": format!("{}{}{}", vstr(&v, "/result/0/scheme"), vstr(&v, "/result/0/host"), vstr(&v, "/result/0/path"))}),
        ),
        None => err(&ctx, 400, "解析错误"),
    }
}

fn html_unescape(s: &str) -> String {
    s.replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
}
