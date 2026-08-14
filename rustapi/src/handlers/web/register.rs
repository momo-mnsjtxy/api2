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
    Html(register_html()).into_response()
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

fn register_html() -> String {
    use super::ui;
    let panel = format!(
        r#"<h2 class="panel-title">创建账户</h2>
<p class="panel-sub">免费注册，获取 APPID / APPKEY</p>
<form class="form-stack" method="post" action="/register/index/callback">
  {gt}
  {user}
  {email}
  {code}
  {pass}
  <button class="btn btn-filled btn-block" type="submit">注册</button>
</form>
<form class="form-stack" method="post" action="/register/index/Email" style="margin-top:1rem">
  {gt2}
  {email2}
  <button class="btn btn-tonal btn-block" type="submit">发送邮箱验证码</button>
</form>
<p class="auth-links">已有账号？ <a href="/login/index/index">去登陆</a></p>"#,
        gt = ui::geetest_hidden(),
        gt2 = ui::geetest_hidden(),
        user = ui::field("用户名", "UserName", "text", r#"required"#),
        email = ui::field("邮箱", "Email", "email", r#"required"#),
        code = ui::field(
            "邮箱验证码",
            "Email_Code",
            "text",
            r#"placeholder="先发送验证码" required"#
        ),
        pass = ui::field("密码", "Password", "password", r#"required"#),
        email2 = ui::field("邮箱", "Email", "email", r#"placeholder="接收验证码的邮箱" required"#),
    );
    ui::auth_layout("账号注册", &panel)
}
