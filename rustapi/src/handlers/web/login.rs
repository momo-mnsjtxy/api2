use std::collections::HashMap;

use axum::{
    extract::{Form, State},
    http::HeaderMap,
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use serde_json::json;
use tower_sessions::Session;

use crate::{auth, db, services::geetest::Geetest, state::AppState, util};

type FormData = HashMap<String, String>;

pub async fn index(session: Session, jar: CookieJar) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Redirect::to("/index/index/index").into_response();
    }
    Html(login_html()).into_response()
}

pub async fn gt_code(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    headers: HeaderMap,
) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Redirect::to("/index/index/index").into_response();
    }
    let ip = util::client_ip(&headers, None);
    let mut gt = Geetest::new(&state.config.geetest_id, &state.config.geetest_key);
    let status = gt
        .pre_process(
            &state.http,
            &[
                ("user_id", "Login"),
                ("client_type", "web"),
                ("ip_address", ip.as_str()),
            ],
            1,
        )
        .await;
    let _ = session.insert("gtserver", status).await;
    let _ = session.insert("class", "Login").await;
    Json(gt.response_json()).into_response()
}

pub async fn callback(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    headers: HeaderMap,
    Form(form): Form<FormData>,
) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Json(json!({"code": 200, "msg": "您已登陆"})).into_response();
    }

    let gt = Geetest::new(&state.config.geetest_id, &state.config.geetest_key);
    let challenge = trimmed(&form, "geetest_challenge");
    let validate = trimmed(&form, "geetest_validate");
    let seccode = trimmed(&form, "geetest_seccode");
    let gtserver = session
        .get::<i32>("gtserver")
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let ip = util::client_ip(&headers, None);
    let captcha_ok = if gtserver == 1 {
        gt.success_validate(
            &state.http,
            &challenge,
            &validate,
            &seccode,
            &[
                ("user_id", "Login"),
                ("client_type", "web"),
                ("ip_address", ip.as_str()),
            ],
        )
        .await
    } else {
        gt.fail_validate(&challenge, &validate, &seccode)
    };
    if !captcha_ok {
        return Json(json!({"code": 400, "msg": "抱歉，验证码验证未通过"})).into_response();
    }

    let username = trimmed(&form, "UserName");
    let password = trimmed(&form, "Password");
    if username.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入邮箱或用户名"})).into_response();
    }
    if password.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入您的登陆密码"})).into_response();
    }

    let password_md5 = util::md5_hex(&password);
    match db::find_user_by_login(&state.pool, &username, &password_md5).await {
        Ok(Some((uid, user_name, email))) => {
            let jar = auth::login_user(&session, jar, uid, &user_name, &email).await;
            (jar, Json(json!({"code": 200, "msg": "登陆成功"}))).into_response()
        }
        Ok(None) => Json(json!({"code": 400, "msg": "您输入账号或者密码错误"})).into_response(),
        Err(_) => Json(json!({"code": 400, "msg": "您输入账号或者密码错误"})).into_response(),
    }
}

fn trimmed(form: &FormData, key: &str) -> String {
    form.get(key)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

fn login_html() -> String {
    use super::ui;
    let panel = format!(
        r#"<h2 class="panel-title">欢迎回来</h2>
<p class="panel-sub">登录后管理接口调用与 APPKEY</p>
<form class="form-stack" method="post" action="/login/index/callback">
  {gt}
  {user}
  {pass}
  <button class="btn btn-filled btn-block" type="submit">登陆</button>
</form>
<p class="auth-links">还没有账户？ <a href="/register/index/index">立即注册</a></p>"#,
        gt = ui::geetest_hidden(),
        user = ui::field(
            "邮箱或用户名",
            "UserName",
            "text",
            r#"autocomplete="username" required"#
        ),
        pass = ui::field(
            "密码",
            "Password",
            "password",
            r#"autocomplete="current-password" required"#
        ),
    );
    ui::auth_layout("账号登陆", &panel)
}
