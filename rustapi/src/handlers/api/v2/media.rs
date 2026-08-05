#![allow(non_snake_case)]

use axum::response::Response;
use rand::seq::SliceRandom;
use reqwest::multipart::{Form, Part};
use serde_json::{json, Value};
use std::path::Path;
use uuid::Uuid;

use super::{
    bytes_response, err, file_ext, http_bytes, http_json, int, is_image_file, ok, proxy_bytes,
    read_public_text, redirect, row_json, row_string, text_response, valid_http_url, vclone, vstr,
    ApiCtx,
};
use crate::{respond, services, util};

pub async fn UserInfo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "UserInfo").await;
    let bro = util::get_bro(&ctx.ua);
    let os = util::get_os_info(&ctx.ua).0;
    let location = baidu_ip_location(&ctx, &ctx.ip).await.unwrap_or_default();
    ok(
        &ctx,
        json!({"ip": ctx.ip, "location": location, "os": os, "browser": bro}),
    )
}

pub async fn Url(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Url").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    if !valid_http_url(url) {
        return err(&ctx, 400, "参数错误");
    }
    let api = match ctx.param("site") {
        Some("url.cn") => format!(
            "http://shorturl.8446666.sojson.com/qq/shorturl?url={}",
            urlencoding::encode(url)
        ),
        Some("t.cn") => format!(
            "http://shorturl.8446666.sojson.com/sina/shorturl?url={}",
            urlencoding::encode(url)
        ),
        _ => return err(&ctx, 400, "参数错误"),
    };
    match http_json(&ctx, &api).await {
        Some(v) => ok(
            &ctx,
            json!({"a": vclone(&v, "/longurl"), "b": vclone(&v, "/shorturl"), "msg": vclone(&v, "/message")}),
        ),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn qlogo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "qlogo").await;
    let Some(qq) = ctx.param("qq") else {
        return err(&ctx, 400, "参数错误");
    };
    proxy_bytes(
        &ctx,
        &format!("https://q1.qlogo.cn/g?b=qq&nk={qq}&s=100"),
        "image/png",
    )
    .await
}

pub async fn Gravatar(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Gravatar").await;
    let Some(email) = ctx.param("email") else {
        return err(&ctx, 400, "参数错误");
    };
    proxy_bytes(
        &ctx,
        &format!(
            "https://gravatar.loli.net/avatar/{}?s=65&r=G&d=",
            util::md5_hex(email)
        ),
        "image/png",
    )
    .await
}

pub async fn Bing_img(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Bing_img").await;
    let Some(v) = http_json(
        &ctx,
        "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=8",
    )
    .await
    else {
        return err(&ctx, 400, "请求失败");
    };
    let images = v
        .get("images")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let Some(img) = images.choose(&mut rand::thread_rng()) else {
        return err(&ctx, 400, "未查询到数据");
    };
    let url = format!("https://www.bing.com/{}", vstr(img, "/url"));
    match ctx.param("type") {
        Some("url") => redirect(&url),
        Some("image") => proxy_bytes(&ctx, &url, "image/png").await,
        _ => ok(
            &ctx,
            json!({"url": url, "title": vclone(img, "/copyright")}),
        ),
    }
}

pub async fn image(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "image").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    if !valid_http_url(url) {
        return err(&ctx, 400, "参数错误");
    }
    proxy_bytes(&ctx, url, "image/png").await
}

pub async fn IP(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "IP").await;
    if ctx.param("type") == Some("map") {
        let lon = ctx.param_or("lon", "0");
        let lat = ctx.param_or("lat", "0");
        return proxy_bytes(
            &ctx,
            &format!("https://cache.ip-api.com/{lon},{lat},10"),
            "image/png",
        )
        .await;
    }
    let ip = ctx.param("ip").unwrap_or(&ctx.ip);
    let url = format!("http://ip-api.com/json/{ip}?lang=zh-CN");
    match http_json(&ctx, &url).await {
        Some(v) => ok(
            &ctx,
            json!({
                "ip": vclone(&v, "/query"),
                "country": vclone(&v, "/country"),
                "countryCode": vclone(&v, "/countryCode"),
                "region": vclone(&v, "/region"),
                "regionName": vclone(&v, "/regionName"),
                "city": vclone(&v, "/city"),
                "zip": vclone(&v, "/zip"),
                "lat": vclone(&v, "/lat"),
                "lon": vclone(&v, "/lon"),
                "timezone": vclone(&v, "/timezone"),
                "isp": vclone(&v, "/isp"),
                "org": vclone(&v, "/org"),
                "as": vclone(&v, "/as"),
            }),
        ),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn IP_IMG(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "IP_IMG").await;
    let Some(img_bytes) = super::read_public_file(&ctx, "xhxh.jpg").await else {
        return err(&ctx, 400, "IP签名图资源缺失");
    };
    let Some(font_bytes) = super::read_public_file(&ctx, "msyh.ttf").await else {
        return err(&ctx, 400, "IP签名图资源缺失");
    };

    let location = baidu_ip_location(&ctx, &ctx.ip)
        .await
        .unwrap_or_else(|| "未知".into());
    let bro = util::get_bro(&ctx.ua);
    let os = util::get_os_info(&ctx.ua).0;
    let week = ["日", "一", "二", "三", "四", "五", "六"];
    let now = chrono::Local::now();
    let weekday = week[now.format("%w").to_string().parse::<usize>().unwrap_or(0)];
    let custom = ctx
        .param("s")
        .and_then(|s| {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let s = s.replace(' ', "+");
            STANDARD
                .decode(s.as_bytes())
                .ok()
                .and_then(|b| String::from_utf8(b).ok())
        })
        .unwrap_or_default();

    let Ok(dyn_img) = image::load_from_memory(&img_bytes) else {
        return err(&ctx, 400, "底图损坏");
    };
    let mut rgba = dyn_img.to_rgba8();
    let red = [255, 0, 0, 255];
    let black = [0, 0, 0, 255];
    let lines = [
        (
            format!("欢迎您来自{location}的朋友"),
            40.0_f32,
            16.0_f32,
            red,
        ),
        (
            format!(
                "今天是{}年{}月{}日 星期{weekday}",
                now.format("%Y"),
                now.format("%m").to_string().trim_start_matches('0'),
                now.format("%d").to_string().trim_start_matches('0'),
            ),
            72.0,
            16.0,
            red,
        ),
        (format!("您的IP是:{}", ctx.ip), 104.0, 16.0, red),
        (format!("您使用的是{os}操作系统"), 140.0, 16.0, red),
        (format!("您使用的是{bro}浏览器"), 175.0, 16.0, red),
        (custom, 200.0, 14.0, black),
    ];
    for (text, y, size, color) in lines {
        if text.is_empty() {
            continue;
        }
        let _ = services::image_text::draw_text(&mut rgba, &font_bytes, &text, 10.0, y, size, color);
    }
    let mut out = Vec::new();
    // JPEG encoder requires RGB8 (no alpha).
    let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
    if image::DynamicImage::ImageRgb8(rgb)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Jpeg)
        .is_err()
    {
        return err(&ctx, 400, "渲染失败");
    }
    bytes_response("image/jpeg", out)
}

pub async fn Email(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Email").await;
    let (Some(email), Some(title), Some(body)) =
        (ctx.param("email"), ctx.param("title"), ctx.param("body"))
    else {
        return err(&ctx, 400, "参数错误");
    };
    match services::mail::send_email(&ctx.state.config, email, title, body).await {
        Ok(_) => int(&ctx, json!({"code": "200", "msg": "发送成功"})),
        Err(e) => int(&ctx, json!({"code": "400", "msg": e.to_string()})),
    }
}

pub async fn ICP(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "ICP").await;
    let Some(domain) = ctx.param("domain") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!("https://api.ooopn.com/icp/api.php?url={domain}");
    match http_json(&ctx, &url).await {
        Some(v) => ok(
            &ctx,
            json!({
                "code": "200",
                "domain": vclone(&v, "/domain"),
                "icp": vclone(&v, "/icp"),
                "sitename": vclone(&v, "/sitename"),
                "name": vclone(&v, "/name"),
                "nature": vclone(&v, "/nature"),
                "time": vclone(&v, "/time"),
            }),
        ),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn DM_IMG(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "DM_IMG").await;
    super::content::content_image_from_list(&ctx, "img.data").await
}

pub async fn C_IMG(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "C_IMG").await;
    super::content::huluxia_image(&ctx).await
}

pub async fn audio(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "audio").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    let lang = ctx.param_or("lang", "zh");
    let url = format!(
        "http://tts.baidu.com/text2audio?lan={lang}&ie=UTF-8&spd=3&text={}",
        urlencoding::encode(text)
    );
    match http_bytes(&ctx, &url).await {
        Some(bytes) => bytes_response("audio/mpeg", bytes),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn qunlogo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "qunlogo").await;
    let Some(qun) = ctx.param("qun") else {
        return err(&ctx, 400, "参数错误");
    };
    proxy_bytes(
        &ctx,
        &format!("http://p.qlogo.cn/gh/{qun}/{qun}/640/"),
        "image/png",
    )
    .await
}

pub async fn upload(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "upload").await;
    let Some((name, bytes)) = ctx.file.clone() else {
        return upload_error(&ctx, "文件错误");
    };
    if !is_image_file(&name, &bytes, 5 * 1024 * 1024) {
        return upload_error(&ctx, "非法的文件格式");
    }
    let ext = file_ext(&name);
    let rel = format!("uploads/{}.{}", Uuid::new_v4(), ext);
    let path = Path::new(&ctx.state.config.public_dir).join(&rel);
    if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }
    match tokio::fs::write(&path, bytes).await {
        Ok(_) => {
            let base = if ctx.state.config.app_url.is_empty() {
                "https://api.gqink.cn".to_string()
            } else {
                ctx.state.config.app_url.trim_end_matches('/').to_string()
            };
            int(
                &ctx,
                json!({
                    "code": 200,
                    "msg": "上传成功",
                    "Copyright": respond::copyright(),
                    "data": {"code": 1, "msg": "上传成功", "url": format!("{base}/public/{rel}")}
                }),
            )
        }
        Err(e) => upload_error(&ctx, &e.to_string()),
    }
}

pub async fn Baidu_Upload(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Baidu_Upload").await;
    let Some((name, bytes)) = ctx.file.clone() else {
        return img_upload_result(&ctx, 400, "非法的文件格式", 0, "非法的文件格式", None);
    };
    if !is_image_file(&name, &bytes, 10 * 1024 * 1024) {
        return img_upload_result(&ctx, 400, "非法的文件格式", 0, "非法的文件格式", None);
    }
    let part = Part::bytes(bytes).file_name(name);
    let form = Form::new().part("image", part);
    let resp = ctx
        .state
        .http
        .post("https://graph.baidu.com/upload")
        .multipart(form)
        .send()
        .await;
    let Ok(resp) = resp else {
        return img_upload_result(&ctx, 400, "上传失败", 0, "上传失败", None);
    };
    let text = resp.text().await.unwrap_or_default();
    let Some(v) = super::parse_json_loose(&text) else {
        return img_upload_result(&ctx, 200, "上传失败", 0, "上传失败", None);
    };
    if vstr(&v, "/msg") == "Success" {
        img_upload_result(
            &ctx,
            200,
            "上传成功",
            1,
            "上传成功",
            Some(format!(
                "https://graph.baidu.com/resource/{}.jpg",
                vstr(&v, "/data/sign")
            )),
        )
    } else {
        img_upload_result(&ctx, 400, "上传失败", 0, "上传失败", None)
    }
}

pub async fn Sogou_Upload(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Sogou_Upload").await;
    let Some((name, bytes)) = ctx.file.clone() else {
        return img_upload_result(&ctx, 200, "非法的文件格式", 0, "非法的文件格式", None);
    };
    if !is_image_file(&name, &bytes, 10 * 1024 * 1024) {
        return img_upload_result(&ctx, 200, "非法的文件格式", 0, "非法的文件格式", None);
    }
    let form = Form::new().part("pic_path", Part::bytes(bytes).file_name(name));
    match ctx
        .state
        .http
        .post("http://pic.sogou.com/ris_upload")
        .multipart(form)
        .send()
        .await
    {
        Ok(resp) => {
            let final_url = resp.url().to_string();
            let imgurl = urlencoding::decode(
                super::between(&final_url, ".com/ris?query=", "&oname=").unwrap_or(""),
            )
            .map(|s| s.to_string().replace("http", "https"))
            .unwrap_or_default();
            if imgurl.is_empty() {
                img_upload_result(&ctx, 200, "上传失败", 1, "上传失败", None)
            } else {
                img_upload_result(&ctx, 200, "上传成功", 1, "上传成功", Some(imgurl))
            }
        }
        Err(_) => img_upload_result(&ctx, 200, "上传失败", 1, "上传失败", None),
    }
}

pub async fn proxy(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "proxy").await;
    if ctx.param("update").is_some() {
        let page = ctx.param_or("page", "1");
        let url = format!("https://www.freeip.top/api/proxy_ips?page={page}");
        let Some(v) = http_json(&ctx, &url).await else {
            return text_response("text/plain; charset=utf-8", "内部错误");
        };
        let mut inserted = 0;
        if let Some(items) = v.pointer("/data/data").and_then(Value::as_array) {
            for item in items {
                let ip = vstr(item, "/ip");
                if ip.is_empty() {
                    continue;
                }
                let exists = sqlx::query("SELECT id FROM freeip WHERE ip=? LIMIT 1")
                    .bind(&ip)
                    .fetch_optional(&ctx.state.pool)
                    .await
                    .ok()
                    .flatten()
                    .is_some();
                if !exists {
                    let _ = sqlx::query(
                        "INSERT INTO freeip (ip, address, port, class) VALUES (?, ?, ?, ?)",
                    )
                    .bind(&ip)
                    .bind(vstr(item, "/ip_address"))
                    .bind(vstr(item, "/port"))
                    .bind(vstr(item, "/protocol"))
                    .execute(&ctx.state.pool)
                    .await;
                    inserted += 1;
                }
            }
        }
        return text_response(
            "text/plain; charset=utf-8",
            if inserted > 0 {
                "更新成功"
            } else {
                "数据重复"
            },
        );
    }
    match super::random_row(&ctx, "freeip").await {
        Some(row) => ok(
            &ctx,
            json!({
                "id": row_json(&row, "id"),
                "ip": row_json(&row, "ip"),
                "port": row_json(&row, "port"),
                "class": row_json(&row, "class"),
                "address": row_json(&row, "address"),
            }),
        ),
        None => err(&ctx, 400, "未查询到数据"),
    }
}

async fn baidu_ip_location(ctx: &ApiCtx, ip: &str) -> Option<String> {
    let url = format!("http://opendata.baidu.com/api.php?query={ip}&co=&resource_id=6006&t=1329357746681&ie=utf8&oe=gbk&format=json&tn=baidu");
    http_json(ctx, &url)
        .await
        .map(|v| vstr(&v, "/data/0/location"))
}

fn upload_error(ctx: &ApiCtx, msg: &str) -> Response {
    int(
        ctx,
        json!({
            "code": 200,
            "msg": msg,
            "Copyright": respond::copyright(),
            "data": {"code": 0, "msg": msg}
        }),
    )
}

fn img_upload_result(
    ctx: &ApiCtx,
    code: i64,
    msg: &str,
    inner_code: i64,
    inner_msg: &str,
    url: Option<String>,
) -> Response {
    let mut data = serde_json::Map::new();
    data.insert("code".into(), json!(inner_code));
    data.insert("msg".into(), json!(inner_msg));
    if let Some(url) = url {
        data.insert("imgurl".into(), json!(url));
    }
    int(
        ctx,
        json!({
            "code": code,
            "msg": msg,
            "Copyright": respond::copyright(),
            "data": Value::Object(data)
        }),
    )
}
