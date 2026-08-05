use std::collections::HashMap;

use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use rand::Rng;
use serde_json::json;
use sqlx::Row;
use tower_sessions::Session;

use crate::{
    auth,
    services::{geetest::Geetest, mail},
    state::AppState,
    util,
};

type FormData = HashMap<String, String>;

pub async fn index(session: Session, jar: CookieJar) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Redirect::to("/index/index/index").into_response();
    }
    Html(REGISTER_HTML).into_response()
}

pub async fn gt_code(
    State(state): State<AppState>,
    session: Session,
    headers: HeaderMap,
) -> Response {
    let ip = util::client_ip(&headers, None);
    let mut gt = Geetest::new(&state.config.geetest_id, &state.config.geetest_key);
    let status = gt
        .pre_process(
            &state.http,
            &[
                ("user_id", "register"),
                ("client_type", "web"),
                ("ip_address", ip.as_str()),
            ],
            1,
        )
        .await;
    let _ = session.insert("gtserver", status).await;
    let _ = session.insert("class", "register").await;
    Json(gt.response_json()).into_response()
}

pub async fn email(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<FormData>,
) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Json(json!({"code": 200, "msg": "您已登陆"})).into_response();
    }

    let ip = util::client_ip(&headers, None);
    if !captcha_ok(&state, &session, &form, &ip).await {
        return Json(json!({"code": 400, "msg": "抱歉，验证码验证未通过"})).into_response();
    }

    let email = trimmed(&form, "Email");
    if email.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入电子邮箱账号"})).into_response();
    }
    if !valid_email(&email) {
        return Json(json!({"code": 400, "msg": "您输入的电子邮箱邮箱格式不正确"})).into_response();
    }
    if user_exists(&state, "Email", &email).await {
        return Json(json!({"code": 400, "msg": "该邮箱号已注册或已绑定其它账号"})).into_response();
    }

    let code = rand::thread_rng().gen_range(100000..=999999);
    let _ = session.insert("RegisterCode", code).await;
    let body = format!("您的验证码是 {code}");
    match mail::send_email(&state.config, &email, "梦城API注册验证", &body).await {
        Ok(_) => Json(json!({"code": 200, "msg": "验证码已经发送到您的邮箱"})).into_response(),
        Err(_) => {
            Json(json!({"code": 400, "msg": "抱歉，邮件发送失败，请联系管理员"})).into_response()
        }
    }
}

pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    Form(form): Form<FormData>,
) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Json(json!({"code": 200, "msg": "您已登陆"})).into_response();
    }

    let username = trimmed(&form, "UserName");
    let email = trimmed(&form, "Email");
    let password = trimmed(&form, "Password");
    let email_code = trimmed(&form, "Email_Code");

    if username.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入设置的用户名"})).into_response();
    }
    if email.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入电子邮箱账号"})).into_response();
    }
    if password.is_empty() {
        return Json(json!({"code": 400, "msg": "您还未输入设置的密码"})).into_response();
    }
    if !valid_username(&username) {
        return Json(json!({"code": 400, "msg": "用户名由4-16位数字字母汉字和下划线组成"}))
            .into_response();
    }
    if !valid_email(&email) {
        return Json(json!({"code": 400, "msg": "您输入的电子邮箱邮箱格式不正确"})).into_response();
    }
    if email_code.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入6位数邮箱验证码"})).into_response();
    }
    if !(6..=18).contains(&password.chars().count()) {
        return Json(json!({"code": 400, "msg": "为了您的安全请输入6~18位密码"})).into_response();
    }
    let expected = session
        .get::<i32>("RegisterCode")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if email_code.parse::<i32>().unwrap_or_default() != expected {
        return Json(
            json!({"code": 400, "msg": format!("{email_code}抱歉！邮箱验证码错误{expected}")}),
        )
        .into_response();
    }
    if user_exists(&state, "UserName", &username).await {
        return Json(json!({"code": 400, "msg": "此用户名已经被使用"})).into_response();
    }
    if user_exists(&state, "Email", &email).await {
        return Json(json!({"code": 400, "msg": "该邮箱号已注册或已绑定其它账号"})).into_response();
    }

    let appkey = util::md5_hex(&format!(
        "{}{}",
        rand::thread_rng().gen_range(1_000_000_000_i64..=9_999_999_999_i64),
        util::now_ts()
    ));
    let inserted = sqlx::query(
        "INSERT INTO user (UserName, Password, Email, APPKEY, AddTime) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&username)
    .bind(util::md5_hex(&password))
    .bind(&email)
    .bind(appkey)
    .bind(util::now_ts())
    .execute(&state.pool)
    .await
    .is_ok();

    if inserted {
        let _ = session.remove::<i32>("RegisterCode").await;
        Json(json!({"code": 200, "msg": "恭喜您，注册成功"})).into_response()
    } else {
        Json(json!({"code": 400, "msg": "抱歉内部错误，请联系管理员"})).into_response()
    }
}

async fn captcha_ok(state: &AppState, session: &Session, form: &FormData, ip: &str) -> bool {
    let gt = Geetest::new(&state.config.geetest_id, &state.config.geetest_key);
    let challenge = trimmed(form, "geetest_challenge");
    let validate = trimmed(form, "geetest_validate");
    let seccode = trimmed(form, "geetest_seccode");
    let gtserver = session
        .get::<i32>("gtserver")
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    if gtserver == 1 {
        gt.success_validate(
            &state.http,
            &challenge,
            &validate,
            &seccode,
            &[
                ("user_id", "register"),
                ("client_type", "web"),
                ("ip_address", ip),
            ],
        )
        .await
    } else {
        gt.fail_validate(&challenge, &validate, &seccode)
    }
}

async fn user_exists(state: &AppState, column: &str, value: &str) -> bool {
    let column = match column {
        "Email" => "Email",
        "UserName" => "UserName",
        _ => return false,
    };
    let sql = format!("SELECT UID FROM user WHERE {column}=? LIMIT 1");
    sqlx::query(&sql)
        .bind(value)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<i64, _>("UID").ok())
        .is_some()
}

fn trimmed(form: &FormData, key: &str) -> String {
    form.get(key)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

fn valid_username(username: &str) -> bool {
    (3..=16).contains(&username.len())
        && username
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && domain
            .rsplit('.')
            .next()
            .map(|tld| (2..=8).contains(&tld.len()))
            .unwrap_or(false)
        && email
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.' | b'@'))
}

const REGISTER_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>账号注册 - 梦城API</title>
  <link rel="shortcut icon" href="https://cdn.gqink.cn/favicon.ico">
  <link href="https://cdn.gqink.cn/var2/css/app.min.css" rel="stylesheet" type="text/css">
</head>
<body class="authentication-bg">
  <main class="container" style="max-width:560px;margin-top:5rem">
    <div class="card">
      <div class="card-header text-center bg-primary">
        <img src="https://cdn.gqink.cn/blog/logo.svg" alt="梦城API" height="48">
      </div>
      <div class="card-body">
        <h3 class="text-center">账号注册</h3>
        <form method="post" action="/register/index/callback">
          <input type="hidden" name="geetest_challenge" value="rust-fallback">
          <input type="hidden" name="geetest_validate" value="d22957a63a507af42dc95a259d8bc8f5">
          <input type="hidden" name="geetest_seccode" value="rust-fallback">
          <div class="form-group mb-3"><label>用户名</label><input class="form-control" name="UserName" required></div>
          <div class="form-group mb-3"><label>邮箱</label><input class="form-control" type="email" name="Email" required></div>
          <div class="form-group mb-3"><label>邮箱验证码</label><input class="form-control" name="Email_Code" placeholder="先 POST /register/index/Email 发送验证码" required></div>
          <div class="form-group mb-3"><label>密码</label><input class="form-control" type="password" name="Password" required></div>
          <button class="btn btn-primary btn-block" type="submit">注册</button>
        </form>
        <form class="mt-3" method="post" action="/register/index/Email">
          <input type="hidden" name="geetest_challenge" value="rust-fallback">
          <input type="hidden" name="geetest_validate" value="d22957a63a507af42dc95a259d8bc8f5">
          <input type="hidden" name="geetest_seccode" value="rust-fallback">
          <div class="input-group">
            <input class="form-control" type="email" name="Email" placeholder="输入邮箱发送验证码">
            <div class="input-group-append"><button class="btn btn-info" type="submit">发送验证码</button></div>
          </div>
        </form>
        <p class="text-center mt-3">已有账号? <a href="/login/index/index">登陆</a></p>
      </div>
    </div>
  </main>
  <footer class="footer footer-alt">2020 © 梦城 - www.gqink.cn</footer>
  <script src="https://cdn.gqink.cn/var2/javascript/app.min.js"></script>
  <script src="https://cdn.gqink.cn/blog/New/js/gt.js"></script>
</body>
</html>"#;
