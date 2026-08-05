#![allow(non_snake_case)]

use axum::response::Response;
use base64::{engine::general_purpose, Engine as _};
use serde_json::{json, Value};
use tokio::process::Command;

use super::{
    between, bytes_response, cache_key, clean_video_title, collapse_html, err, head_status,
    http_json, http_post_json, http_post_text, http_text, int, json_ok, json_with_msg, now_millis,
    read_public_text, redirect, text_response, vclone, vstr, ApiCtx,
};
use crate::{respond, services, util};

pub async fn MD5(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "MD5").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    super::ok(&ctx, json!({"text": text, "MD5": util::md5_hex(text)}))
}

pub async fn QrCode(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "QrCode").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    let size = ctx.param_i64("site", 300).max(64) as u32;
    match services::qrcode::render_png(text, size) {
        Ok(bytes) => bytes_response("image/png", bytes),
        Err(_) => err(&ctx, 400, "二维码生成失败"),
    }
}

pub async fn QrReader(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "QrReader").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    match services::qrcode::read_qr_from_url(&ctx.state.http, url).await {
        Some(text) => super::ok(&ctx, json!({"url": url, "text": text})),
        None => err(&ctx, 400, "解析失败"),
    }
}

pub async fn Mobile(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Mobile").await;
    let Some(mobile) = ctx.param("mobile") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!("https://cx.shouji.360.cn/phonearea.php?number={mobile}");
    if let Some(v) = http_json(&ctx, &url).await {
        let province = vstr(&v, "/data/province");
        if !province.is_empty() {
            let city = vstr(&v, "/data/city");
            let operator = vstr(&v, "/data/sp");
            return super::ok(
                &ctx,
                json!({
                    "mobile": mobile,
                    "province": province,
                    "city": city,
                    "operator": operator,
                    "prefix": "",
                    "location": format!("{province}{city}{operator}"),
                }),
            );
        }
    }
    err(&ctx, 400, "未查询到该号码信息")
}

pub async fn whois(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "whois").await;
    let Some(domain) = ctx.param("domain") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!("https://api.devopsclub.cn/api/whoisquery?domain={domain}&type=json");
    let Some(v) = http_json(&ctx, &url).await else {
        return err(&ctx, 400, "请求失败");
    };
    let d = v.pointer("/data/data").unwrap_or(&Value::Null);
    let domain_name = vstr(d, "/domainName");
    if domain_name.is_empty() {
        return err(&ctx, 400, "未查询到数据");
    }
    super::ok(
        &ctx,
        json!({
            "domainName": domain_name,
            "registrationTime": choose_str(d, &["/registrationTime", "/creationDate"]),
            "expirationTime": choose_str(d, &["/expirationTime", "/registryExpiryDate"]),
            "nameServer": vclone(d, "/nameServer"),
            "registrant": choose_str(d, &["/registrant", "/registrar"]),
            "registrantContactEmail": choose_str(d, &["/registrantContactEmail", "/registrarAbuseContactEmail"]),
            "sponsoringRegistrar": choose_str(d, &["/sponsoringRegistrar", "/registrarURL"]),
        }),
    )
}

pub async fn pinyin(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "pinyin").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    super::ok(
        &ctx,
        json!({"text": text, "pinyin": services::pinyin::pinyin_text(text)}),
    )
}

pub async fn fanyi(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "fanyi").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    let lang = match ctx.param("lang") {
        Some(s)
            if matches!(
                s,
                "ZH_CN2EN"
                    | "ZH_CN2JA"
                    | "ZH_CN2KR"
                    | "ZH_CN2FR"
                    | "ZH_CN2RU"
                    | "ZH_CN2SP"
                    | "EN2ZH_CN"
                    | "JA2ZH_CN"
                    | "KR2ZH_CN"
                    | "FR2ZH_CN"
                    | "RU2ZH_CN"
                    | "SP2ZH_CN"
            ) =>
        {
            s
        }
        _ => "AUTO",
    };
    let url = format!(
        "http://fanyi.youdao.com/translate?&doctype=json&type={lang}&i={}",
        urlencoding::encode(text)
    );
    match http_json(&ctx, &url).await {
        Some(v) => super::ok(
            &ctx,
            json!({
                "lang": vstr(&v, "/type"),
                "text": vstr(&v, "/translateResult/0/0/src"),
                "tgt": vstr(&v, "/translateResult/0/0/tgt"),
            }),
        ),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn ping(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "ping").await;
    let Some(host) = ctx.param("ip") else {
        return err(&ctx, 400, "参数错误");
    };
    let output = Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("-w")
        .arg("5")
        .arg(host)
        .output()
        .await;
    let Ok(output) = output else {
        return super::ok(
            &ctx,
            json!({"code": 404, "msg": format!("Ping请求找不到主机{host}")}),
        );
    };
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        return super::ok(&ctx, json!({"code": 403, "msg": "请求目标超时"}));
    }
    let ip = between(&stdout, "(", ")").unwrap_or(host).to_string();
    let (min, avg, max) = parse_ping_times(&stdout);
    let loc = http_json(&ctx, &format!("http://ip-api.com/json/{ip}?lang=zh-CN"))
        .await
        .unwrap_or(Value::Null);
    super::ok(
        &ctx,
        json!({
            "code": 200,
            "msg": "请求目标成功",
            "ip": ip,
            "domain_ip": stdout.lines().next().unwrap_or("").to_string(),
            "ping_min": min,
            "ping_avg": avg,
            "ping_max": max,
            "country": vstr(&loc, "/country"),
            "region": vstr(&loc, "/regionName"),
            "city": vstr(&loc, "/city"),
            "isp": vstr(&loc, "/isp"),
            "org": format!("{}{}", vstr(&loc, "/org"), vstr(&loc, "/as")),
        }),
    )
}

pub async fn robot(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "robot").await;
    let Some(text) = ctx.param("text") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!(
        "http://i.itpk.cn/api.php?question={}&api_key=c7c0fe42b152684bc6971e881dbba254&api_secret=2q9k54fd2ufs",
        urlencoding::encode(text)
    );
    let Some(resp) = http_post_text(&ctx, &url).await else {
        return err(&ctx, 400, "请求失败");
    };
    if let Some(v) = super::parse_json_loose(&resp) {
        let msg = robot_msg(&v).unwrap_or_else(|| resp.clone());
        super::ok(&ctx, json!({"text": text, "msg": msg}))
    } else {
        super::ok(&ctx, json!({"text": text, "msg": resp}))
    }
}

pub async fn weather(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "weather").await;
    let Some(mut location) = ctx.param("location").map(str::to_string) else {
        return err(&ctx, 400, "参数错误");
    };
    if location == "AUTO" {
        if let Some(v) = http_json(
            &ctx,
            &format!("http://ip-api.com/json/{}?lang=zh-CN", ctx.ip),
        )
        .await
        {
            let city = vstr(&v, "/city");
            if !city.is_empty() {
                location = city;
            }
        }
    }
    let url = format!(
        "https://api.seniverse.com/v3/weather/daily.json?key=t6lanuj8zky47a6v&unit=c&start=0&days=3&location={}",
        urlencoding::encode(&location)
    );
    match http_json(&ctx, &url).await {
        Some(v) if v.pointer("/results/0/location").is_some() => super::ok(
            &ctx,
            json!({
                "location": vclone(&v, "/results/0/location"),
                "daily": vclone(&v, "/results/0/daily"),
            }),
        ),
        _ => super::ok(&ctx, json!({"code": 400, "msg": "地区错误"})),
    }
}

pub async fn WeatherInfo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "WeatherInfo").await;
    let Some(keyword) = ctx.param("keyword") else {
        return err(&ctx, 400, "参数错误");
    };
    let live_url = format!("https://restapi.amap.com/v3/weather/weatherInfo?city={keyword}&key=fcf6233aaffbec6673b803191db3d00c");
    let all_url = format!("https://restapi.amap.com/v3/weather/weatherInfo?city={keyword}&key=fcf6233aaffbec6673b803191db3d00c&extensions=all");
    let live = http_json(&ctx, &live_url).await.unwrap_or(Value::Null);
    let cache_key = cache_key("WeatherInfo_", keyword);
    let weather = if let Some(cached) = ctx.state.cache.get(&cache_key).await {
        serde_json::from_str::<Value>(&cached).unwrap_or(Value::Null)
    } else {
        let forecasts = http_json(&ctx, &all_url)
            .await
            .map(|v| vclone(&v, "/forecasts"))
            .unwrap_or(Value::Null);
        let _ = ctx
            .state
            .cache
            .set(&cache_key, forecasts.to_string(), 43_200)
            .await;
        forecasts
    };
    super::ok(
        &ctx,
        json!({"time": vclone(&live, "/lives"), "weather": weather}),
    )
}

pub async fn laji(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "laji").await;
    let Some(kw) = ctx.param("kw") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!(
        "https://ai.sm.cn/quark/1/ai?format=json&q={}是什么垃圾",
        urlencoding::encode(kw)
    );
    let Some(v) = http_json(&ctx, &url).await else {
        return err(&ctx, 400, "未找到相关结果");
    };
    if v.pointer("/data/0/guide").is_none() || v.pointer("/data/0/tts").is_none() {
        let msg = vstr(&v, "/data/0/value/answer").replace("夸克宝宝", "");
        return int(
            &ctx,
            json!({"code": 400, "msg": if msg.is_empty() { "未找到相关结果".into() } else { msg }}),
        );
    }
    int(
        &ctx,
        json!({
            "code": 200,
            "msg": vstr(&v, "/data/0/guide"),
            "Copyright": respond::copyright(),
            "data": {
                "title": vstr(&v, "/data/0/value/title"),
                "desc": vstr(&v, "/data/0/value/desc"),
                "pic": vstr(&v, "/data/0/value/pic"),
            }
        }),
    )
}

pub async fn address(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "address").await;
    match read_public_text(&ctx, "address.json")
        .await
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
    {
        Some(v) => int(&ctx, v),
        None => err(&ctx, 400, "数据文件不存在"),
    }
}

pub async fn check(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "check").await;
    let Some(domain) = ctx.param("domain") else {
        return err(&ctx, 400, "参数错误");
    };
    let url = format!("https://api.oioweb.cn/api/baidu.php?url={domain}");
    match http_json(&ctx, &url).await {
        Some(v) if v.get("code").and_then(Value::as_i64) == Some(1) => int(
            &ctx,
            json!({
                "code": 200,
                "msg": "查询成功",
                "Copyright": respond::copyright(),
                "data": {
                    "domain": domain,
                    "baidu": vclone(&v, "/data/baidu"),
                    "sougou": vclone(&v, "/data/sougou"),
                    "360": vclone(&v, "/data/360"),
                }
            }),
        ),
        _ => err(&ctx, 400, "请求失败"),
    }
}

pub async fn check_domain(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "check_domain").await;
    let Some(domain) = ctx.param("domain") else {
        return err(&ctx, 400, "参数错误");
    };
    let resp = ctx
        .state
        .http
        .post("http://panda.www.net.cn/cgi-bin/check.cgi")
        .body(format!("area_domain={domain}"))
        .send()
        .await;
    let resp = match resp {
        Ok(r) => r.text().await.ok(),
        Err(_) => None,
    };
    let Some(xml) = resp else {
        return err(&ctx, 400, "请求失败");
    };
    let original = between(&xml, "<original>", "</original>").unwrap_or(&xml);
    let (code, msg) = if original.starts_with("210") {
        (0, "域名可以注册")
    } else if original.starts_with("211") {
        (1, "域名已经被注册")
    } else {
        return err(&ctx, 400, "参数错误");
    };
    int(
        &ctx,
        json!({
            "code": 200,
            "msg": "查询成功",
            "Copyright": respond::copyright(),
            "data": {"code": code, "domain": domain, "msg": msg}
        }),
    )
}

pub async fn Url_Sec(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Url_Sec").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let api = format!("https://cgi.urlsec.qq.com/index.php?m=check&a=check&callback=jQuery111306943167371763181_1567183944271&url={}&_={}", urlencoding::encode(url), now_millis());
    let Some(text) = http_text(&ctx, &api).await else {
        return err(&ctx, 400, "请求失败");
    };
    let Some(v) = super::parse_json_loose(&text) else {
        return err(&ctx, 400, "请求失败");
    };
    if v.get("reCode").and_then(Value::as_i64).unwrap_or(1) != 0 {
        return int(&ctx, json!({"code": 400, "msg": vclone(&v, "/data")}));
    }
    let white = v
        .pointer("/data/results/whitetype")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let kind = if white == 1 || white == 3 {
        "正常"
    } else {
        "拦截"
    };
    let code = if kind == "正常" { 1 } else { 0 };
    int(
        &ctx,
        json!({
            "code": 200,
            "msg": "查询成功",
            "Copyright": respond::copyright(),
            "data": {
                "code": code,
                "msg": "检测成功",
                "url": vclone(&v, "/data/results/url"),
                "type": kind,
            }
        }),
    )
}

pub async fn HttpCode(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "HttpCode").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    match head_status(&ctx, url).await {
        Some(code) => super::ok(&ctx, json!({"HttpCode": code, "url": url})),
        None => err(&ctx, 400, "请求失败"),
    }
}

pub async fn RandPass(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "RandPass").await;
    let num = ctx.param_i64("num", 16).clamp(1, 256) as usize;
    let (a, b, c) = util::rand_password(num);
    super::ok(&ctx, json!({"a": a, "b": b, "c": c}))
}

pub async fn Go(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "Go").await;
    let Some(url) = ctx.param("url") else {
        return err(&ctx, 400, "参数错误");
    };
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta name="robots" content="noindex, nofollow"><meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1"><link rel="dns-prefetch" href="{0}" /><title>加载中，请您稍候…</title></head><body><h2>加载中，请您稍候…</h2><script>setTimeout(function(){{location.href="{0}";}},1000);setTimeout(function(){{window.opener=null;window.close();}},50000);</script></body></html>"#,
        html_escape(url)
    );
    text_response("text/html; charset=utf-8", html)
}

pub async fn district(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "district").await;
    let Some(keyword) = ctx.param("keyword") else {
        return err(&ctx, 400, "参数错误");
    };
    let key = cache_key("city_", keyword);
    let url = format!("https://restapi.amap.com/v3/config/district?keywords={keyword}&subdistrict=3&key=fcf6233aaffbec6673b803191db3d00c&offset=5000");
    super::content::amap_cached(&ctx, key, url, "/districts").await
}

pub async fn IpSadd(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "IpSadd").await;
    let ip = ctx.param("ip").unwrap_or(&ctx.ip).to_string();
    let key = cache_key("IpSadd_", &ip);
    let url = format!(
        "https://restapi.amap.com/v3/ip?ip={ip}&output=json&key=fcf6233aaffbec6673b803191db3d00c"
    );
    super::content::amap_cached(&ctx, key, url, "").await
}

pub async fn QQChat(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "QQChat").await;
    let Some(qq) = ctx.param("qq") else {
        return err(&ctx, 400, "参数错误");
    };
    redirect(&format!("tencent://message/?uin={qq}"))
}

pub async fn QQInfo(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "QQInfo").await;
    let Some(qq) = ctx.param("qq") else {
        return err(&ctx, 400, "参数错误");
    };
    let url =
        format!("http://r.qzone.qq.com/fcg-bin/cgi_get_portrait.fcg?g_tk=1518561325&uins={qq}");
    let bytes_resp = ctx
        .state
        .http
        .get(&url)
        .send()
        .await;
    let bytes = match bytes_resp {
        Ok(r) => r.bytes().await.ok(),
        Err(_) => None,
    };
    let text = bytes
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    let nickname = between(&text, "portraitCallBack(", ")")
        .and_then(super::parse_json_loose)
        .and_then(|v| {
            v.pointer(&format!("/{qq}/6"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    super::ok(
        &ctx,
        json!({
            "qq": qq,
            "nickname": nickname,
            "headimg": format!("http://q1.qlogo.cn/g?b=qq&nk={qq}&s=100&t=1547904810"),
            "email": format!("{qq}@qq.com"),
        }),
    )
}

pub async fn idcard(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "idcard").await;
    let Some(idcard) = ctx.param("idcard") else {
        return err(&ctx, 400, "参数错误");
    };
    if !valid_idcard(idcard) {
        return err(&ctx, 400, "身份证格式错误");
    }
    let key = cache_key("idcard_s", idcard);
    let data = cached_scrape(&ctx, &key, &format!("https://shenfenzheng.51240.com/{idcard}__shenfenzheng/"), |html| {
        json!({
            "idcard": between(html, "证件号码</td><td bgcolor=\"#FFFFFF\"align=\"center\">", "</td>").unwrap_or(""),
            "address": between(html, "发 证 地</td><td bgcolor=\"#FFFFFF\"align=\"center\">", "</td>").unwrap_or(""),
            "sr": between(html, "出生日期</td><td bgcolor=\"#FFFFFF\"align=\"center\">", "</td>").unwrap_or(""),
            "age": between(html, "性别年龄</td><td bgcolor=\"#FFFFFF\"align=\"center\">", "</td>").unwrap_or(""),
        })
    })
    .await;
    super::ok(&ctx, data)
}

pub async fn telxj(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "telxj").await;
    let Some(tel) = ctx.param("tel") else {
        return err(&ctx, 400, "参数错误");
    };
    if !(tel.len() == 11 && tel.starts_with('1') && tel.chars().all(|c| c.is_ascii_digit())) {
        return err(&ctx, 400, "手机号格式错误");
    }
    let key = cache_key("telxj_", tel);
    let data = cached_scrape(&ctx, &key, &format!("https://jx.ip138.com/{tel}/"), |html| {
        json!({
            "xiongji": between(html, "凶吉推理</p></td><td colspan=\"3\"><p>", "</p>").unwrap_or(""),
            "anshi": between(html, "暗示的信息：</p></td><td colspan=\"3\">", "</td>").unwrap_or(""),
            "shiyun": between(html, "诗云：</p></td><td colspan=\"3\">", "</td>").unwrap_or(""),
            "leixing": between(html, "性格类型：</b></td><td><p>", "</p>").unwrap_or(""),
            "biaoxian": between(html, "具体表现：</b></td><td colspan=\"3\">", "</td>").unwrap_or(""),
            "yanyu": between(html, "谚语：</b></td><td colspan=\"3\">", "</td>").unwrap_or(""),
        })
    })
    .await;
    super::ok(&ctx, data)
}

pub async fn qqxj(ctx: ApiCtx) -> Response {
    super::log_action(&ctx, "qqxj").await;
    let Some(qq) = ctx.param("qq") else {
        return err(&ctx, 400, "参数错误");
    };
    let key = cache_key("qqxj_", qq);
    let data = cached_scrape(&ctx, &key, &format!("http://qq.link114.cn/{qq}"), |html| {
        json!({
            "xg": between(html, "<dd>", "</dd>").unwrap_or(""),
            "xyx": format!("{}{}", between(html, "<dd><font color=\"red\">", "</font>").unwrap_or(""), between(html, "</font>", "</dd>").unwrap_or("")),
            "qw": between(html, "<dl><dt>签文：</dt><dd>", "</dd></dl>").unwrap_or(""),
            "jq": between(html, "<dl><dt>解签：</dt><dd>", "</dd></dl>").unwrap_or(""),
        })
    })
    .await;
    super::ok(&ctx, data)
}

fn choose_str(v: &Value, paths: &[&str]) -> String {
    paths
        .iter()
        .map(|p| vstr(v, p))
        .find(|s| !s.is_empty())
        .unwrap_or_default()
}

fn parse_ping_times(stdout: &str) -> (String, String, String) {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.contains(" = "))
        .unwrap_or("");
    let times = line.split('=').nth(1).unwrap_or("").trim();
    let mut parts = times.split('/').map(str::trim);
    (
        parts.next().unwrap_or("").to_string(),
        parts.next().unwrap_or("").to_string(),
        parts.next().unwrap_or("").to_string(),
    )
}

fn robot_msg(v: &Value) -> Option<String> {
    if !vstr(v, "/content").is_empty() {
        return Some(format!("【{}】{}", vstr(v, "/title"), vstr(v, "/content")));
    }
    match vstr(v, "/type").as_str() {
        "观音灵签" => Some(format!(
            "您抽取的是第{}【{}】签<br/>签位：{}<br/>签语：{}<br/>诗意：{}<br/>解签：{}",
            vstr(v, "/number2"),
            vstr(v, "/number1"),
            vstr(v, "/haohua"),
            vstr(v, "/qianyu"),
            vstr(v, "/shiyi"),
            vstr(v, "/jieqian")
        )),
        "月老灵签" => Some(format!(
            "您抽取的是第{}签<br/>签位：{}<br/>签语：{}<br/>注释：{}<br/>解签：{}<br/>白话释义：{}",
            vstr(v, "/number2"),
            vstr(v, "/haohua"),
            vstr(v, "/shiyi"),
            vstr(v, "/zhushi"),
            vstr(v, "/jieqian"),
            vstr(v, "/baihua")
        )),
        "财神爷灵签" => Some(format!(
            "您抽取的是第{}签<br/>签语：{}<br/>注释：{}<br/>解签：{}<br/>解说：{}<br/>结果：{}<br/>婚姻：{}<br/>事业：{}<br/>功名：{}<br/>失物：{}<br/>出外移居：{}<br/>六甲：{}<br/>求财：{}<br/>交易：{}<br/>疾病：{}<br/>诉讼：{}<br/>运途：{}<br/>某事：{}<br/>合作人：{}",
            vstr(v, "/number2"), vstr(v, "/qianyu"), vstr(v, "/zhushi"), vstr(v, "/jieqian"),
            vstr(v, "/jieshuo"), vstr(v, "/jieguo"), vstr(v, "/hunyin"), vstr(v, "/shiye"),
            vstr(v, "/gongming"), vstr(v, "/shiwu"), vstr(v, "/cwyj"), vstr(v, "/liujia"),
            vstr(v, "/qiucai"), vstr(v, "/jiaoyi"), vstr(v, "/jibin"), vstr(v, "/susong"),
            vstr(v, "/yuntu"), vstr(v, "/moushi"), vstr(v, "/hhzsy")
        )),
        _ => None,
    }
}

async fn cached_scrape<F>(ctx: &ApiCtx, key: &str, url: &str, parser: F) -> Value
where
    F: Fn(&str) -> Value,
{
    if let Some(cached) = ctx.state.cache.get(key).await {
        if let Ok(v) = serde_json::from_str::<Value>(&cached) {
            return v;
        }
    }
    let html = http_text(ctx, url).await.unwrap_or_default();
    let data = parser(&collapse_html(&html));
    let _ = ctx.state.cache.set(key, data.to_string(), 43_200).await;
    data
}

fn valid_idcard(id: &str) -> bool {
    let len_ok = id.len() == 18;
    let chars_ok = id
        .chars()
        .enumerate()
        .all(|(i, c)| c.is_ascii_digit() || (i == 17 && matches!(c, 'X' | 'x')));
    len_ok && chars_ok && !id.starts_with('0')
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
