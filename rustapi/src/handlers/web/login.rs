use std::collections::HashMap;

use axum::{
    extract::{Form, State},
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
    Html(LOGIN_HTML).into_response()
}

pub async fn gt_code(State(state): State<AppState>, session: Session, jar: CookieJar) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        return Redirect::to("/index/index/index").into_response();
    }
    let gt = Geetest::new(&state.config.geetest_id, &state.config.geetest_key);
    let status = gt.pre_process();
    let _ = session.insert("gtserver", status).await;
    let _ = session.insert("class", "Login").await;
    Json(gt.response_json()).into_response()
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
    let captcha_ok = if gtserver == 1 {
        gt.success_validate(&challenge, &validate, &seccode)
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

const LOGIN_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>账号登陆 - 梦城API</title>
  <link rel="shortcut icon" href="https://cdn.gqink.cn/blog/favicon.ico">
  <link href="https://cdn.gqink.cn/var2/css/app.min.css" rel="stylesheet" type="text/css">
</head>
<body class="authentication-bg">
  <main class="container" style="max-width:520px;margin-top:5rem">
    <div class="card">
      <div class="card-header text-center bg-primary">
        <img src="https://cdn.gqink.cn/blog/logo.svg" alt="梦城API" height="48">
      </div>
      <div class="card-body">
        <h3 class="text-center">账号登录</h3>
        <p class="text-muted text-center">输入您的账号密码来访问控制面板</p>
        <form method="post" action="/login/index/callback">
          <input type="hidden" name="geetest_challenge" value="rust-fallback">
          <input type="hidden" name="geetest_validate" value="rust-fallback">
          <input type="hidden" name="geetest_seccode" value="rust-fallback">
          <div class="form-group mb-3">
            <label>邮箱或用户名</label>
            <input class="form-control" name="UserName" autocomplete="username" required>
          </div>
          <div class="form-group mb-3">
            <label>密码</label>
            <input class="form-control" type="password" name="Password" autocomplete="current-password" required>
          </div>
          <button class="btn btn-primary btn-block" type="submit">登陆</button>
        </form>
        <p class="text-center mt-3">还没有账户? <a href="/register/index/index">注册</a></p>
      </div>
    </div>
  </main>
  <footer class="footer footer-alt">2020 © 梦城 - www.gqink.cn</footer>
  <script src="https://cdn.gqink.cn/var2/javascript/app.min.js"></script>
  <script src="https://cdn.gqink.cn/blog/New/js/gt.js"></script>
</body>
</html>"#;
