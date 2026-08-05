use std::collections::HashMap;

use axum::{
    extract::{Form, Query, State},
    response::{Html, IntoResponse, Redirect, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{Local, TimeZone};
use rand::Rng;
use serde_json::{json, Value};
use sqlx::{mysql::MySqlRow, Column, MySql, MySqlPool, QueryBuilder, Row};
use tower_sessions::Session;

use crate::{auth, services::mail, state::AppState, util};

type Params = HashMap<String, String>;
type FormData = HashMap<String, String>;

#[derive(Clone, Debug)]
struct UserView {
    uid: i64,
    username: String,
    email: String,
    appkey: String,
    code: i64,
}

#[derive(Clone, Debug)]
struct ApiView {
    keyword: String,
    name: String,
    des: String,
    action: String,
    code: i64,
    raw: HashMap<String, String>,
}

pub async fn home(session: Session, jar: CookieJar) -> Response {
    if auth::current_user(&session, &jar).await.is_some() {
        Redirect::to("/index/index/index").into_response()
    } else {
        Redirect::to("/login/index/index").into_response()
    }
}

pub async fn index(State(state): State<AppState>, session: Session, jar: CookieJar) -> Response {
    let (user, apis) = match page_context(&state, &session, &jar).await {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let stats = dashboard_stats(&state.pool, user.uid, None).await;
    let recent = recent_requests(&state.pool, user.uid, None, 30, 0)
        .await
        .unwrap_or_default();
    let body = format!(
        r#"<h1>控制中心</h1>
<div class="row">
  {cards}
</div>
<h3>最近调用</h3>
{recent}
"#,
        cards = stat_cards(&stats),
        recent = request_table(&recent),
    );
    Html(layout("控制中心", &user, &apis, &body)).into_response()
}

pub async fn page(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    Query(params): Query<Params>,
) -> Response {
    let (user, apis) = match page_context(&state, &session, &jar).await {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let keyword = params.get("api").map(String::as_str).unwrap_or_default();
    let Some(api) = load_api(&state.pool, keyword).await.unwrap_or(None) else {
        return Html(simple_error("接口不存在")).into_response();
    };
    if api.code == 0 {
        return Html(simple_error("接口异常已经被关闭")).into_response();
    }

    if params.get("type").map(String::as_str) == Some("list") {
        let p = params
            .get("p")
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(1);
        let count = count_requests(&state.pool, user.uid, Some(&api.keyword), None, None)
            .await
            .unwrap_or(0);
        let rows = recent_requests(&state.pool, user.uid, Some(&api.keyword), 15, (p - 1) * 15)
            .await
            .unwrap_or_default();
        return Json(json!({
            "conf": {"count": count, "PageCount": ((count + 14) / 15).max(0), "Page": p},
            "data": rows.iter().map(request_row_json).collect::<Vec<_>>(),
        }))
        .into_response();
    }

    let stats = dashboard_stats(&state.pool, user.uid, Some(&api.keyword)).await;
    let params_table = action_table(&api.action);
    let body = format!(
        r#"<h1>{name}</h1>
<p>{des}</p>
<div class="row">{cards}</div>
<h3>请求方式</h3>
<table class="table table-bordered"><tr><th>GET</th><th>POST</th><th>PUT</th></tr><tr><td>{get}</td><td>{post}</td><td>{put}</td></tr></table>
<h3>参数说明</h3>{params_table}
<h3>调用记录</h3>
<p><a class="btn btn-secondary" href="/index/index/page?api={keyword}&type=list">JSON 列表</a></p>
"#,
        name = html_escape(&api.name),
        des = html_escape(&api.des),
        cards = stat_cards(&stats),
        get = support_text(api.raw.get("get")),
        post = support_text(api.raw.get("post")),
        put = support_text(api.raw.get("put")),
        params_table = params_table,
        keyword = html_escape(&api.keyword),
    );
    Html(layout(&api.name, &user, &apis, &body)).into_response()
}

pub async fn setting(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    Query(params): Query<Params>,
    form: Option<Form<FormData>>,
) -> Response {
    let (user, apis) = match page_context(&state, &session, &jar).await {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let form = form.map(|Form(form)| form).unwrap_or_default();
    match params.get("type").map(String::as_str) {
        Some("doEmail") => return send_repass_code(&state, &session, &user).await,
        Some("RePass") => return reset_password(&state, &session, &user, &form).await,
        Some("ReEmailCode") => return send_reemail_code(&state, &session, &user, &form).await,
        Some("ReEmail") => return reset_email(&state, &session, &user, &form).await,
        _ => {}
    }

    let body = format!(
        r#"<h1>账号设置</h1>
<div class="row">
  <div class="col-md-6">
    <h3>修改登陆密码</h3>
    <form method="post" action="/index/index/setting?type=RePass">
      <p>UID: {uid}</p><p>绑定邮箱: {email}</p>
      <input class="form-control mb-2" type="password" name="Password" placeholder="新密码">
      <input class="form-control mb-2" name="Email_Code" placeholder="邮箱验证码">
      <button class="btn btn-primary" type="submit">修改密码</button>
    </form>
    <form method="post" action="/index/index/setting?type=doEmail" class="mt-2"><button class="btn btn-info" type="submit">发送改密验证码</button></form>
  </div>
  <div class="col-md-6">
    <h3>修改邮箱绑定</h3>
    <form method="post" action="/index/index/setting?type=ReEmail">
      <p>当前邮箱: {email}</p>
      <input class="form-control mb-2" type="email" name="Email" placeholder="新的邮箱号">
      <input class="form-control mb-2" name="EmailCode" placeholder="邮箱验证码">
      <button class="btn btn-primary" type="submit">修改邮箱</button>
    </form>
    <form method="post" action="/index/index/setting?type=ReEmailCode" class="mt-2">
      <input class="form-control mb-2" type="email" name="Email" placeholder="新的邮箱号">
      <button class="btn btn-info" type="submit">发送换绑验证码</button>
    </form>
  </div>
</div>"#,
        uid = user.uid,
        email = html_escape(&user.email),
    );
    Html(layout("账号设置", &user, &apis, &body)).into_response()
}

pub async fn appkey(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    Query(params): Query<Params>,
) -> Response {
    let (user, apis) = match page_context(&state, &session, &jar).await {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    if params.get("type").map(String::as_str) == Some("APPKEY") {
        let appkey = new_appkey();
        let updated = sqlx::query("UPDATE user SET APPKEY=? WHERE UID=?")
            .bind(&appkey)
            .bind(user.uid)
            .execute(&state.pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .unwrap_or(false);
        if updated {
            return Json(
                json!({"code": 200, "msg": format!("新的APPKEY是：{appkey}"), "New": appkey}),
            )
            .into_response();
        }
        return Json(json!({"code": 400, "msg": "抱歉，重置成功，请联系管理员"})).into_response();
    }

    let status = if user.code > 0 { "正常" } else { "异常" };
    let body = format!(
        r#"<h1>APPKEY</h1>
<form method="post" action="/index/index/appkey?type=APPKEY">
  <div class="form-group"><label>APPID</label><input class="form-control" readonly value="{uid}"></div>
  <div class="form-group"><label>用户名</label><input class="form-control" readonly value="{username}"></div>
  <div class="form-group"><label>APPKEY</label><input class="form-control" readonly value="{appkey}"></div>
  <div class="form-group"><label>APPID状态</label><input class="form-control" readonly value="{status}"></div>
  <button class="btn btn-dark" type="submit">重置</button>
</form>"#,
        uid = user.uid,
        username = html_escape(&user.username),
        appkey = html_escape(&user.appkey),
        status = status,
    );
    Html(layout("APPKEY", &user, &apis, &body)).into_response()
}

pub async fn login_out(session: Session, jar: CookieJar) -> Response {
    if auth::current_user(&session, &jar).await.is_none() {
        return Redirect::to("/login/index/index").into_response();
    }
    let jar = auth::logout_user(&session, jar).await;
    (jar, Redirect::to("/login/index/index")).into_response()
}

#[allow(non_snake_case)]
pub async fn LoginOut(session: Session, jar: CookieJar) -> Response {
    login_out(session, jar).await
}

pub async fn ip(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    Query(params): Query<Params>,
) -> Response {
    if auth::current_user(&session, &jar).await.is_none() {
        return Redirect::to("/login/index/index").into_response();
    }
    let address = params
        .get("address")
        .map(String::as_str)
        .unwrap_or_default();
    if address.is_empty() {
        return Json(json!({"code": 400, "msg": "address required"})).into_response();
    }
    let url = format!("http://ip-api.com/json/{address}?lang=zh-CN");
    for _ in 0..10 {
        if let Ok(resp) = state.http.get(&url).send().await {
            if let Ok(value) = resp.json::<Value>().await {
                return Json(value).into_response();
            }
        }
    }
    Json(json!({"code": 400, "msg": "内部错误"})).into_response()
}

pub async fn log() -> Response {
    Html("开发中").into_response()
}

async fn page_context(
    state: &AppState,
    session: &Session,
    jar: &CookieJar,
) -> Result<(UserView, Vec<ApiView>), Response> {
    let Some(auth_user) = auth::current_user(session, jar).await else {
        return Err(Redirect::to("/login/index/index").into_response());
    };
    let user = load_user(&state.pool, auth_user.uid, &auth_user.username)
        .await
        .unwrap_or(None)
        .unwrap_or(UserView {
            uid: auth_user.uid,
            username: auth_user.username,
            email: auth_user.email,
            appkey: String::new(),
            code: 0,
        });
    let apis = load_apis(&state.pool).await.unwrap_or_default();
    Ok((user, apis))
}

async fn load_user(pool: &MySqlPool, uid: i64, username: &str) -> sqlx::Result<Option<UserView>> {
    let row = sqlx::query(
        "SELECT UID, UserName, Email, APPKEY, Code FROM user WHERE UID=? AND UserName=? LIMIT 1",
    )
    .bind(uid)
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|row| UserView {
        uid: get_i64(&row, "UID").unwrap_or(uid),
        username: get_string(&row, "UserName").unwrap_or_else(|| username.to_string()),
        email: get_string(&row, "Email").unwrap_or_default(),
        appkey: get_string(&row, "APPKEY").unwrap_or_default(),
        code: get_i64(&row, "Code").unwrap_or_default(),
    }))
}

async fn load_apis(pool: &MySqlPool) -> sqlx::Result<Vec<ApiView>> {
    let rows = sqlx::query("SELECT * FROM api WHERE Code=1 ORDER BY id ASC")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(api_from_row).collect())
}

async fn load_api(pool: &MySqlPool, keyword: &str) -> sqlx::Result<Option<ApiView>> {
    let row = sqlx::query("SELECT * FROM api WHERE Keyword=? LIMIT 1")
        .bind(keyword)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(api_from_row))
}

fn api_from_row(row: &MySqlRow) -> ApiView {
    let mut raw = HashMap::new();
    for column in row.columns() {
        let name = column.name();
        raw.insert(name.to_string(), get_string(row, name).unwrap_or_default());
    }
    ApiView {
        keyword: get_string(row, "Keyword").unwrap_or_default(),
        name: get_string(row, "Name")
            .unwrap_or_else(|| get_string(row, "Keyword").unwrap_or_default()),
        des: get_string(row, "Des").unwrap_or_default(),
        action: get_string(row, "action").unwrap_or_default(),
        code: get_i64(row, "Code").unwrap_or(1),
        raw,
    }
}

#[derive(Default)]
struct Stats {
    today: i64,
    yesterday: i64,
    day2: i64,
    day3: i64,
    day4: i64,
    day5: i64,
    day6: i64,
    week: i64,
    lastweek: i64,
    month: i64,
    lastmonth: i64,
    count: i64,
    security_count: i64,
}

async fn dashboard_stats(pool: &MySqlPool, uid: i64, keyword: Option<&str>) -> Stats {
    let (today_start, today_end) = util::range_today();
    let (yesterday_start, yesterday_end) = util::range_yesterday();
    let (week_start, week_end) = util::range_this_week();
    let (lastweek_start, lastweek_end) = util::range_last_week();
    let (month_start, month_end) = util::range_this_month();
    let (lastmonth_start, lastmonth_end) = util::range_last_month();
    Stats {
        today: count_requests(pool, uid, keyword, None, Some((today_start, today_end)))
            .await
            .unwrap_or(0),
        yesterday: count_requests(
            pool,
            uid,
            keyword,
            None,
            Some((yesterday_start, yesterday_end)),
        )
        .await
        .unwrap_or(0),
        day2: count_day(pool, uid, keyword, 2).await,
        day3: count_day(pool, uid, keyword, 3).await,
        day4: count_day(pool, uid, keyword, 4).await,
        day5: count_day(pool, uid, keyword, 5).await,
        day6: count_day(pool, uid, keyword, 6).await,
        week: count_requests(pool, uid, keyword, None, Some((week_start, week_end)))
            .await
            .unwrap_or(0),
        lastweek: count_requests(
            pool,
            uid,
            keyword,
            None,
            Some((lastweek_start, lastweek_end)),
        )
        .await
        .unwrap_or(0),
        month: count_requests(pool, uid, keyword, None, Some((month_start, month_end)))
            .await
            .unwrap_or(0),
        lastmonth: count_requests(
            pool,
            uid,
            keyword,
            None,
            Some((lastmonth_start, lastmonth_end)),
        )
        .await
        .unwrap_or(0),
        count: count_requests(pool, uid, keyword, None, None)
            .await
            .unwrap_or(0),
        security_count: count_requests(pool, uid, keyword, Some(1), None)
            .await
            .unwrap_or(0),
    }
}

async fn count_day(pool: &MySqlPool, uid: i64, keyword: Option<&str>, days_ago: i64) -> i64 {
    let (start, end) = util::day_range_offset(days_ago);
    count_requests(pool, uid, keyword, None, Some((start, end)))
        .await
        .unwrap_or(0)
}

async fn count_requests(
    pool: &MySqlPool,
    uid: i64,
    keyword: Option<&str>,
    code: Option<i64>,
    range: Option<(i64, i64)>,
) -> sqlx::Result<i64> {
    let mut qb = QueryBuilder::<MySql>::new("SELECT COUNT(*) AS c FROM request WHERE UID=");
    qb.push_bind(uid);
    if let Some(keyword) = keyword {
        qb.push(" AND Keyword=");
        qb.push_bind(keyword);
    }
    if let Some(code) = code {
        qb.push(" AND Code=");
        qb.push_bind(code);
    }
    if let Some((start, end)) = range {
        qb.push(" AND TIME BETWEEN ");
        qb.push_bind(start);
        qb.push(" AND ");
        qb.push_bind(end);
    }
    let row = qb.build().fetch_one(pool).await?;
    Ok(row.try_get::<i64, _>("c").unwrap_or(0))
}

async fn recent_requests(
    pool: &MySqlPool,
    uid: i64,
    keyword: Option<&str>,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<MySqlRow>> {
    let mut qb = QueryBuilder::<MySql>::new(
        "SELECT TIME, UserAgent, IP, Request, Keyword, Code FROM request WHERE UID=",
    );
    qb.push_bind(uid);
    if let Some(keyword) = keyword {
        qb.push(" AND Keyword=");
        qb.push_bind(keyword);
    }
    qb.push(" ORDER BY id DESC LIMIT ");
    qb.push_bind(limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);
    qb.build().fetch_all(pool).await
}

async fn send_repass_code(state: &AppState, session: &Session, user: &UserView) -> Response {
    let code = rand::thread_rng().gen_range(100000..=999999);
    let _ = session.insert("RePass", code).await;
    let body = format!("您的验证码是 {code}");
    match mail::send_email(&state.config, &user.email, "梦城API修改密码验证", &body).await {
        Ok(_) => Json(json!({"code": 200, "msg": "验证码已经发送到您的邮箱"})).into_response(),
        Err(_) => {
            Json(json!({"code": 400, "msg": "抱歉，邮件发送失败，请联系管理员"})).into_response()
        }
    }
}

async fn reset_password(
    state: &AppState,
    session: &Session,
    user: &UserView,
    form: &FormData,
) -> Response {
    let password = trimmed(form, "Password");
    let email_code = trimmed(form, "Email_Code");
    if password.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入您要设置的新的密码"})).into_response();
    }
    if email_code.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入6位数邮箱验证码"})).into_response();
    }
    let expected = session
        .get::<i32>("RePass")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if email_code.parse::<i32>().unwrap_or_default() != expected {
        return Json(json!({"code": 400, "msg": "抱歉！邮箱验证码错误"})).into_response();
    }
    let password_md5 = util::md5_hex(&password);
    let same = sqlx::query("SELECT UID FROM user WHERE UID=? AND Password=? LIMIT 1")
        .bind(user.uid)
        .bind(&password_md5)
        .fetch_optional(&state.pool)
        .await
        .ok()
        .flatten()
        .is_some();
    if same {
        return Json(json!({"code": 400, "msg": "新密码不能跟旧密码相同"})).into_response();
    }
    let updated = sqlx::query("UPDATE user SET Password=? WHERE UID=?")
        .bind(password_md5)
        .bind(user.uid)
        .execute(&state.pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false);
    if updated {
        let _ = session.remove::<i32>("RePass").await;
        Json(json!({"code": 200, "msg": "恭喜你！密码修改成功"})).into_response()
    } else {
        Json(json!({"code": 400, "msg": "抱歉！修改失败。请重试"})).into_response()
    }
}

async fn send_reemail_code(
    state: &AppState,
    session: &Session,
    user: &UserView,
    form: &FormData,
) -> Response {
    let email = trimmed(form, "Email");
    if email == user.email {
        return Json(json!({"code": 400, "msg": "新邮箱不能跟旧邮箱相同"})).into_response();
    }
    let code = rand::thread_rng().gen_range(100000..=999999);
    let _ = session.insert("ReEmail", code).await;
    let body = format!("您的验证码是 {code}");
    match mail::send_email(&state.config, &email, "梦城API修改绑定验证", &body).await {
        Ok(_) => Json(json!({"code": 200, "msg": "验证码已经发送到您的邮箱"})).into_response(),
        Err(_) => {
            Json(json!({"code": 400, "msg": "抱歉，邮件发送失败，请联系管理员"})).into_response()
        }
    }
}

async fn reset_email(
    state: &AppState,
    session: &Session,
    user: &UserView,
    form: &FormData,
) -> Response {
    let email = trimmed(form, "Email");
    let email_code = trimmed(form, "EmailCode");
    if email.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入您要设置的新的密码"})).into_response();
    }
    if email_code.is_empty() {
        return Json(json!({"code": 400, "msg": "请输入6位数邮箱验证码"})).into_response();
    }
    let expected = session
        .get::<i32>("ReEmail")
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
    if email_code.parse::<i32>().unwrap_or_default() != expected {
        return Json(json!({"code": 400, "msg": "抱歉！邮箱验证码错误"})).into_response();
    }
    if email == user.email {
        return Json(json!({"code": 400, "msg": "新邮箱不能跟旧邮箱相同"})).into_response();
    }
    let updated = sqlx::query("UPDATE user SET Email=? WHERE UID=?")
        .bind(&email)
        .bind(user.uid)
        .execute(&state.pool)
        .await
        .map(|r| r.rows_affected() > 0)
        .unwrap_or(false);
    if updated {
        let _ = session.remove::<i32>("ReEmail").await;
        Json(json!({"code": 200, "msg": "恭喜你！邮箱绑定修改成功"})).into_response()
    } else {
        Json(json!({"code": 400, "msg": "抱歉！修改失败。请重试"})).into_response()
    }
}

fn stat_cards(stats: &Stats) -> String {
    let today_to = ratio(stats.today, stats.yesterday);
    let week_to = ratio(stats.week, stats.lastweek);
    let month_to = ratio(stats.month, stats.lastmonth);
    let security_to = ratio(stats.security_count, stats.count);
    [
        ("今天调用", stats.today, format!("相对昨天 {today_to:.2}%")),
        ("昨天调用", stats.yesterday, format!("近七天: {}, {}, {}, {}, {}, {}", stats.day2, stats.day3, stats.day4, stats.day5, stats.day6, stats.yesterday)),
        ("本周调用", stats.week, format!("上周 {} / {week_to:.2}%", stats.lastweek)),
        ("本月调用", stats.month, format!("上月 {} / {month_to:.2}%", stats.lastmonth)),
        ("总调用", stats.count, format!("拦截恶意 {}", stats.security_count)),
        ("拦截占比", stats.security_count, format!("{security_to:.2}%")),
    ]
    .iter()
    .map(|(title, value, note)| {
        format!(
            r#"<div class="col-md-4"><div class="card mb-3"><div class="card-body"><h5>{}</h5><h3>{}</h3><p class="text-muted">{}</p></div></div></div>"#,
            html_escape(title),
            value,
            html_escape(note)
        )
    })
    .collect::<Vec<_>>()
    .join("")
}

fn ratio(a: i64, b: i64) -> f64 {
    if a > 0 && b > 0 {
        if a < b {
            a as f64 / b as f64 * 100.0
        } else {
            b as f64 / a as f64 * 100.0
        }
    } else {
        0.0
    }
}

fn request_table(rows: &[MySqlRow]) -> String {
    let trs = rows
        .iter()
        .map(|row| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                html_escape(&format_ts(get_i64(row, "TIME").unwrap_or_default())),
                html_escape(&get_string(row, "Request").unwrap_or_default()),
                html_escape(&get_string(row, "IP").unwrap_or_default()),
                html_escape(&get_string(row, "UserAgent").unwrap_or_default()),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<table class="table table-striped"><thead><tr><th>时间</th><th>方式</th><th>IP</th><th>UserAgent</th></tr></thead><tbody>{trs}</tbody></table>"#
    )
}

fn request_row_json(row: &MySqlRow) -> Value {
    json!({
        "TIME": get_i64(row, "TIME").unwrap_or_default(),
        "UserAgent": get_string(row, "UserAgent").unwrap_or_default(),
        "IP": get_string(row, "IP").unwrap_or_default(),
        "Request": get_string(row, "Request").unwrap_or_default(),
        "Keyword": get_string(row, "Keyword").unwrap_or_default(),
        "Code": get_i64(row, "Code").unwrap_or_default(),
    })
}

fn action_table(raw: &str) -> String {
    let parsed = serde_json::from_str::<Value>(raw.trim_start_matches('\u{feff}')).ok();
    let rows = parsed
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(|row| {
            let cells = row
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|cell| format!("<td>{}</td>", html_escape(&value_to_string(cell))))
                .collect::<Vec<_>>()
                .join("");
            format!("<tr>{cells}</tr>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<table class="table table-bordered"><tbody>{rows}</tbody></table>"#)
}

fn layout(title: &str, user: &UserView, apis: &[ApiView], body: &str) -> String {
    let api_links = apis
        .iter()
        .map(|api| {
            format!(
                r#"<a class="dropdown-item" href="/index/index/page?api={}">{}</a>"#,
                urlencoding::encode(&api.keyword),
                html_escape(&api.name)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let headimg = headimg(&user.email);
    format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{title} - 梦城API</title>
  <link rel="shortcut icon" href="https://cdn.gqink.cn/blog/favicon.ico">
  <link href="https://cdn.gqink.cn/var2/css/app.min.css" rel="stylesheet" type="text/css">
</head>
<body>
<nav class="navbar navbar-expand-lg navbar-dark bg-primary">
  <a class="navbar-brand" href="/index/index/index">梦城API</a>
  <div class="navbar-nav">
    <a class="nav-item nav-link" href="/index/index/index">控制中心</a>
    <a class="nav-item nav-link" href="/index/index/appkey">APPKEY</a>
    <a class="nav-item nav-link" href="/index/index/setting">账号设置</a>
    <a class="nav-item nav-link" href="/index/index/LoginOut">退出登录</a>
  </div>
</nav>
<main class="container-fluid mt-4">
  <div class="media mb-3"><img src="{headimg}" class="mr-3 rounded" width="50" height="50" alt=""><div class="media-body"><strong>{username}</strong><br><small>{email}</small></div></div>
  <div class="mb-3">{api_links}</div>
  {body}
</main>
<script src="https://cdn.gqink.cn/var2/javascript/app.min.js"></script>
<script src="https://cdn.gqink.cn/sweetalert/sweetalert/sweetalert.min.js"></script>
</body>
</html>"#,
        title = html_escape(title),
        headimg = html_escape(&headimg),
        username = html_escape(&user.username),
        email = html_escape(&user.email),
        api_links = api_links,
        body = body,
    )
}

fn simple_error(msg: &str) -> String {
    format!(
        r#"<!doctype html><html lang="zh-CN"><meta charset="utf-8"><title>错误</title><body><h1>{}</h1><p><a href="/index/index/index">返回控制中心</a></p></body></html>"#,
        html_escape(msg)
    )
}

fn support_text(value: Option<&String>) -> &'static str {
    if value.map(String::as_str) == Some("1") {
        "支持"
    } else {
        "不支持"
    }
}

fn headimg(email: &str) -> String {
    if let Some(qq) = email.strip_suffix("@qq.com") {
        format!("https://q.qlogo.cn/g?b=qq&nk={qq}&s=100")
    } else {
        format!(
            "https://cdn.v2ex.com/gravatar/{}?s=65&r=G&d",
            util::md5_hex(email)
        )
    }
}

fn new_appkey() -> String {
    util::md5_hex(&format!(
        "{}{}",
        rand::thread_rng().gen_range(1_000_000_000_i64..=9_999_999_999_i64),
        util::now_ts()
    ))
}

fn trimmed(form: &FormData, key: &str) -> String {
    form.get(key)
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
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

fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(v) => v.to_string(),
        _ => value.to_string(),
    }
}

fn format_ts(ts: i64) -> String {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default()
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
