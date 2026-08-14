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

use super::ui;

type Params = HashMap<String, String>;
type FormData = HashMap<String, String>;

#[derive(Clone, Debug)]
struct UserView {
    uid: i64,
    username: String,
    email: String,
    appkey: String,
    code: i64,
    add_time: i64,
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
    let (week_start, week_end) = util::range_this_week();
    let (today_start, today_end) = util::range_today();
    let top_apis = ranked_keywords(&state.pool, user.uid, week_start, week_end, 8)
        .await
        .unwrap_or_default();
    let top_ips = ranked_ips(&state.pool, user.uid, week_start, week_end, 8)
        .await
        .unwrap_or_default();
    let methods = method_breakdown(&state.pool, user.uid, week_start, week_end)
        .await
        .unwrap_or_default();
    let hourly = hourly_today(&state.pool, user.uid, today_start, today_end)
        .await
        .unwrap_or_else(|_| vec![0; 24]);
    let security = recent_security(&state.pool, user.uid, 12)
        .await
        .unwrap_or_default();
    let recent = recent_requests(&state.pool, user.uid, None, 50, 0)
        .await
        .unwrap_or_default();
    let base = public_base(&state);
    let body = home_body(
        &state,
        &user,
        &apis,
        &stats,
        &top_apis,
        &top_ips,
        &methods,
        &hourly,
        &security,
        &recent,
        &base,
    );
    Html(layout("控制中心", "home", &user, &apis, None, &body)).into_response()
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
    let get_ok = support_text(api.raw.get("get")) == "支持";
    let post_ok = support_text(api.raw.get("post")) == "支持";
    let put_ok = support_text(api.raw.get("put")) == "支持";
    let methods_inner = format!(
        r#"<div class="support-grid">
  <div class="support-pill"><strong>GET</strong><span class="{g}">{gt}</span></div>
  <div class="support-pill"><strong>POST</strong><span class="{p}">{pt}</span></div>
  <div class="support-pill"><strong>PUT</strong><span class="{u}">{ut}</span></div>
</div>"#,
        g = if get_ok { "support-ok" } else { "support-no" },
        p = if post_ok { "support-ok" } else { "support-no" },
        u = if put_ok { "support-ok" } else { "support-no" },
        gt = support_text(api.raw.get("get")),
        pt = support_text(api.raw.get("post")),
        ut = support_text(api.raw.get("put")),
    );
    let base = public_base(&state);
    let endpoint = format!(
        "{base}/api/v2/{kw}?appid={uid}&appkey={key}",
        base = base.trim_end_matches('/'),
        kw = api.keyword,
        uid = user.uid,
        key = user.appkey,
    );
    let endpoint_card = ui::section_card(
        "接口地址",
        &format!(
            r#"<button type="button" class="btn btn-tonal" onclick="navigator.clipboard.writeText(document.getElementById('ep-url').value)">{}复制</button>"#,
            ui::icon("content_copy")
        ),
        &format!(
            r#"<label class="field"><span class="field-label">GET 示例</span>
<input class="field-ro" id="ep-url" readonly value="{ep}"></label>
<p class="stat-note">POST / PUT 请将参数放在 Body；返回格式可用 type=json|xml</p>"#,
            ep = ui::html_escape(&endpoint),
        ),
    );
    let try_panel = playground_panel(&apis, &user, &base, Some(&api.keyword));
    let body = format!(
        r#"<p class="lead">{des}</p>
<div class="stat-grid">{cards}</div>
{methods}
{endpoint}
{params}
{try_panel}
{chart}
<p class="split-actions">{json_btn}{logs_btn}</p>
"#,
        des = ui::html_escape(&api.des),
        cards = stat_cards(&stats),
        methods = ui::section_card("请求方式", "", &methods_inner),
        endpoint = endpoint_card,
        params = ui::section_card("参数说明", "", &params_table),
        try_panel = try_panel,
        chart = week_chart(&stats),
        json_btn = ui::outlined_button_link(
            "JSON 调用列表",
            &format!(
                "/index/index/page?api={}&type=list",
                urlencoding::encode(&api.keyword)
            ),
        ),
        logs_btn = ui::outlined_button_link(
            "完整日志",
            &format!("/index/index/log?kw={}", urlencoding::encode(&api.keyword)),
        ),
    );
    Html(layout(&api.name, "page", &user, &apis, Some(&api.keyword), &body)).into_response()
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
        r#"<div class="form-grid">
  {pass_card}
  {email_card}
</div>"#,
        pass_card = ui::section_card(
            "修改登陆密码",
            "",
            &format!(
                r#"<form class="form-stack" method="post" action="/index/index/setting?type=RePass">
  <label class="field"><span class="field-label">UID</span><input class="field-ro" readonly value="{uid}"></label>
  <label class="field"><span class="field-label">绑定邮箱</span><input class="field-ro" readonly value="{email}"></label>
  {pw}
  {code}
  {submit}
</form>
<form method="post" action="/index/index/setting?type=doEmail" style="margin-top:0.75rem">{send}</form>"#,
                uid = user.uid,
                email = ui::html_escape(&user.email),
                pw = ui::field("新密码", "Password", "password", r#"placeholder="6~18 位新密码" required"#),
                code = ui::field("邮箱验证码", "Email_Code", "text", r#"placeholder="6 位验证码" required"#),
                submit = ui::filled_button("修改密码", ""),
                send = ui::tonal_button("发送改密验证码", ""),
            ),
        ),
        email_card = ui::section_card(
            "修改邮箱绑定",
            "",
            &format!(
                r#"<form class="form-stack" method="post" action="/index/index/setting?type=ReEmail">
  <label class="field"><span class="field-label">当前邮箱</span><input class="field-ro" readonly value="{email}"></label>
  {ne}
  {code}
  {submit}
</form>
<form class="form-stack" method="post" action="/index/index/setting?type=ReEmailCode" style="margin-top:0.75rem">
  {ne2}
  {send}
</form>"#,
                email = ui::html_escape(&user.email),
                ne = ui::field("新邮箱", "Email", "email", r#"placeholder="新的邮箱号" required"#),
                code = ui::field("邮箱验证码", "EmailCode", "text", r#"placeholder="6 位验证码" required"#),
                submit = ui::filled_button("修改邮箱", ""),
                ne2 = ui::field("新邮箱", "Email", "email", r#"placeholder="新的邮箱号" required"#),
                send = ui::tonal_button("发送换绑验证码", ""),
            ),
        ),
    );
    Html(layout("账号设置", "setting", &user, &apis, None, &body)).into_response()
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
    let body = ui::section_card(
        "应用凭证",
        "",
        &format!(
            r#"<form class="form-stack" method="post" action="/index/index/appkey?type=APPKEY">
  <label class="field"><span class="field-label">APPID</span><input class="field-ro" readonly value="{uid}"></label>
  <label class="field"><span class="field-label">用户名</span><input class="field-ro" readonly value="{username}"></label>
  <label class="field"><span class="field-label">APPKEY</span><input class="field-ro" readonly value="{appkey}"></label>
  <label class="field"><span class="field-label">状态</span><input class="field-ro" readonly value="{status}"></label>
  <div class="split-actions">{reset}</div>
</form>"#,
            uid = user.uid,
            username = ui::html_escape(&user.username),
            appkey = ui::html_escape(&user.appkey),
            status = status,
            reset = ui::filled_button("重置 APPKEY", ""),
        ),
    );
    Html(layout("APPKEY", "appkey", &user, &apis, None, &body)).into_response()
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

pub async fn log(
    State(state): State<AppState>,
    session: Session,
    jar: CookieJar,
    Query(params): Query<Params>,
) -> Response {
    let (user, apis) = match page_context(&state, &session, &jar).await {
        Ok(ctx) => ctx,
        Err(resp) => return resp,
    };
    let p = params
        .get("p")
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|v| *v > 0)
        .unwrap_or(1);
    let kw = params.get("kw").map(String::as_str).filter(|s| !s.is_empty());
    let total = count_requests(&state.pool, user.uid, kw, None, None)
        .await
        .unwrap_or(0);
    let rows = recent_requests(&state.pool, user.uid, kw, 40, (p - 1) * 40)
        .await
        .unwrap_or_default();
    let pages = ((total + 39) / 40).max(1);
    let filter = format!(
        r#"<form class="toolbar" method="get" action="/index/index/log">
  <input class="field-input" style="max-width:220px" name="kw" value="{kw}" placeholder="按接口 Keyword 筛选">
  <button class="btn btn-tonal" type="submit">{icon}筛选</button>
  <a class="btn btn-outlined" href="/index/index/log">全部</a>
</form>"#,
        kw = ui::html_escape(kw.unwrap_or("")),
        icon = ui::icon("filter_alt"),
    );
    let pager = format!(
        r#"<div class="pager">第 {p} / {pages} 页 · 共 {total} 条
  <a class="btn btn-outlined" href="/index/index/log?p={prev}{kwq}">上一页</a>
  <a class="btn btn-outlined" href="/index/index/log?p={next}{kwq}">下一页</a>
</div>"#,
        p = p,
        pages = pages,
        total = total,
        prev = (p - 1).max(1),
        next = (p + 1).min(pages),
        kwq = kw
            .map(|k| format!("&kw={}", urlencoding::encode(k)))
            .unwrap_or_default(),
    );
    let body = format!(
        "{filter}{table}{pager}",
        filter = filter,
        table = ui::section_card("调用日志", "", &request_table_full(&rows)),
        pager = pager,
    );
    Html(layout("调用日志", "log", &user, &apis, None, &body)).into_response()
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
            add_time: 0,
        });
    let apis = load_apis(&state.pool).await.unwrap_or_default();
    Ok((user, apis))
}

async fn load_user(pool: &MySqlPool, uid: i64, username: &str) -> sqlx::Result<Option<UserView>> {
    let row = sqlx::query(
        "SELECT UID, UserName, Email, APPKEY, Code, AddTime FROM user WHERE UID=? AND UserName=? LIMIT 1",
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
        add_time: get_i64(&row, "AddTime").unwrap_or_default(),
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


fn public_base(state: &AppState) -> String {
    if state.config.app_url.trim().is_empty() {
        "http://127.0.0.1:8080".into()
    } else {
        state.config.app_url.trim_end_matches('/').to_string()
    }
}

#[derive(Clone)]
struct RankRow {
    key: String,
    count: i64,
}

async fn ranked_keywords(
    pool: &MySqlPool,
    uid: i64,
    start: i64,
    end: i64,
    limit: i64,
) -> sqlx::Result<Vec<RankRow>> {
    let rows = sqlx::query(
        "SELECT Keyword AS k, COUNT(*) AS c FROM request WHERE UID=? AND TIME BETWEEN ? AND ? AND Keyword IS NOT NULL AND Keyword<>'' GROUP BY Keyword ORDER BY c DESC LIMIT ?",
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| RankRow {
            key: get_string(r, "k").unwrap_or_default(),
            count: get_i64(r, "c").unwrap_or(0),
        })
        .collect())
}

async fn ranked_ips(
    pool: &MySqlPool,
    uid: i64,
    start: i64,
    end: i64,
    limit: i64,
) -> sqlx::Result<Vec<RankRow>> {
    let rows = sqlx::query(
        "SELECT IP AS k, COUNT(*) AS c FROM request WHERE UID=? AND TIME BETWEEN ? AND ? AND IP IS NOT NULL AND IP<>'' GROUP BY IP ORDER BY c DESC LIMIT ?",
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| RankRow {
            key: get_string(r, "k").unwrap_or_default(),
            count: get_i64(r, "c").unwrap_or(0),
        })
        .collect())
}

async fn method_breakdown(
    pool: &MySqlPool,
    uid: i64,
    start: i64,
    end: i64,
) -> sqlx::Result<Vec<RankRow>> {
    let rows = sqlx::query(
        "SELECT Request AS k, COUNT(*) AS c FROM request WHERE UID=? AND TIME BETWEEN ? AND ? GROUP BY Request ORDER BY c DESC",
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .iter()
        .map(|r| RankRow {
            key: get_string(r, "k").unwrap_or_else(|| "UNKNOWN".into()),
            count: get_i64(r, "c").unwrap_or(0),
        })
        .collect())
}

async fn hourly_today(
    pool: &MySqlPool,
    uid: i64,
    start: i64,
    end: i64,
) -> sqlx::Result<Vec<i64>> {
    let rows = sqlx::query(
        "SELECT HOUR(FROM_UNIXTIME(TIME)) AS h, COUNT(*) AS c FROM request WHERE UID=? AND TIME BETWEEN ? AND ? GROUP BY h",
    )
    .bind(uid)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;
    let mut out = vec![0_i64; 24];
    for r in rows {
        let h = get_i64(&r, "h").unwrap_or(0).clamp(0, 23) as usize;
        out[h] = get_i64(&r, "c").unwrap_or(0);
    }
    Ok(out)
}

async fn recent_security(pool: &MySqlPool, uid: i64, limit: i64) -> sqlx::Result<Vec<MySqlRow>> {
    sqlx::query(
        "SELECT TIME, UserAgent, IP, Request, Keyword, Code FROM request WHERE UID=? AND Code=1 ORDER BY id DESC LIMIT ?",
    )
    .bind(uid)
    .bind(limit)
    .fetch_all(pool)
    .await
}

fn home_body(
    _state: &AppState,
    user: &UserView,
    apis: &[ApiView],
    stats: &Stats,
    top_apis: &[RankRow],
    top_ips: &[RankRow],
    methods: &[RankRow],
    hourly: &[i64],
    security: &[MySqlRow],
    recent: &[MySqlRow],
    base: &str,
) -> String {
    let cred = credentials_bar(user, base, apis.len());
    let cards = format!(r#"<div class="stat-grid">{}</div>"#, stat_cards(stats));
    let charts = format!(
        r#"<div class="dash-split">
  <div>{week}</div>
  <div class="stack-gap">
    {methods}
    {hourly}
  </div>
</div>"#,
        week = week_chart(stats),
        methods = methods_chart(methods),
        hourly = hourly_chart(hourly),
    );
    let ranks = format!(
        r#"<div class="dash-tri">
  {apis}
  {ips}
  {sec}
</div>"#,
        apis = rank_card("本周热门接口", top_apis, true),
        ips = rank_card("本周来源 IP", top_ips, false),
        sec = security_card(security),
    );
    let catalog = api_catalog(apis, top_apis);
    let play = playground_panel(apis, user, base, None);
    let logs = ui::section_card(
        "最近调用明细",
        &format!(
            r#"<div class="split-actions">
  <input class="field-input" id="log-filter" style="max-width:200px" placeholder="筛选接口/IP/UA" oninput="filterLogs()">
  <button type="button" class="btn btn-tonal" onclick="exportLogs()">{}导出 JSON</button>
  <a class="btn btn-outlined" href="/index/index/log">{}全部日志</a>
</div>"#,
            ui::icon("download"),
            ui::icon("list_alt"),
        ),
        &request_table_full(recent),
    );
    format!(
        "{cred}{cards}{charts}{ranks}{catalog}{play}{logs}{script}",
        cred = cred,
        cards = cards,
        charts = charts,
        ranks = ranks,
        catalog = catalog,
        play = play,
        logs = logs,
        script = DASH_SCRIPT,
    )
}

fn credentials_bar(user: &UserView, base: &str, api_count: usize) -> String {
    let status = if user.code > 0 { "正常" } else { "异常" };
    let status_cls = if user.code > 0 { "chip-good" } else { "chip-warn" };
    format!(
        r#"<section class="cred-bar surface-card">
  <div class="cred-main">
    <div>
      <p class="eyebrow">账户凭证</p>
      <h2 class="section-title" style="margin:0.2rem 0 0">你好，{name}</h2>
      <p class="stat-note">注册于 {reg} · 可用接口 {n} 个 · API Base <code>{base}/api/v2</code></p>
    </div>
    <span class="chip {status_cls}">{status}</span>
  </div>
  <div class="cred-grid">
    <label class="field"><span class="field-label">APPID</span>
      <div class="copy-row"><input class="field-ro" id="cred-appid" readonly value="{uid}"><button type="button" class="btn btn-tonal" onclick="navigator.clipboard.writeText(document.getElementById('cred-appid').value)">复制</button></div>
    </label>
    <label class="field"><span class="field-label">APPKEY</span>
      <div class="copy-row"><input class="field-ro" id="cred-appkey" readonly value="{key}"><button type="button" class="btn btn-tonal" onclick="navigator.clipboard.writeText(document.getElementById('cred-appkey').value)">复制</button></div>
    </label>
    <label class="field"><span class="field-label">邮箱</span>
      <input class="field-ro" readonly value="{email}">
    </label>
  </div>
  <div class="split-actions" style="margin-top:0.85rem">
    <a class="btn btn-filled" href="/index/index/appkey">{ik}管理 APPKEY</a>
    <a class="btn btn-tonal" href="/index/index/setting">{is}账号设置</a>
    <a class="btn btn-outlined" href="/index/index/log">{il}调用日志</a>
  </div>
</section>"#,
        name = ui::html_escape(&user.username),
        reg = ui::html_escape(&format_ts(user.add_time)),
        n = api_count,
        base = ui::html_escape(base.trim_end_matches('/')),
        status_cls = status_cls,
        status = status,
        uid = user.uid,
        key = ui::html_escape(&user.appkey),
        email = ui::html_escape(&user.email),
        ik = ui::icon("key"),
        is = ui::icon("manage_accounts"),
        il = ui::icon("receipt_long"),
    )
}

fn rank_card(title: &str, rows: &[RankRow], is_api: bool) -> String {
    let max = rows.iter().map(|r| r.count).max().unwrap_or(1).max(1);
    let list = if rows.is_empty() {
        r#"<p class="stat-note">暂无数据，调用接口后会出现排行。</p>"#.to_string()
    } else {
        rows.iter()
            .map(|r| {
                let pct = (r.count as f64 / max as f64 * 100.0).round();
                let label = if is_api {
                    format!(
                        r#"<a href="/index/index/page?api={kw}">{name}</a>"#,
                        kw = urlencoding::encode(&r.key),
                        name = ui::html_escape(&r.key),
                    )
                } else {
                    format!(
                        r#"<a href="javascript:void(0)" onclick="lookupIp('{ip}')">{ip}</a>"#,
                        ip = ui::html_escape(&r.key),
                    )
                };
                format!(
                    r#"<div class="rank-row">
  <div class="rank-meta"><span>{label}</span><strong>{c}</strong></div>
  <div class="rank-bar"><span style="width:{pct}%"></span></div>
</div>"#,
                    label = label,
                    c = r.count,
                    pct = pct,
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };
    ui::section_card(title, "", &list)
}

fn security_card(rows: &[MySqlRow]) -> String {
    let list = if rows.is_empty() {
        r#"<p class="stat-note">近期没有拦截记录，状态良好。</p>"#.to_string()
    } else {
        rows.iter()
            .map(|r| {
                format!(
                    r#"<div class="sec-row">
  <div><strong>{kw}</strong> · {ip}<br><span class="stat-note">{ts} · {method}</span></div>
  <span class="chip chip-warn">拦截</span>
</div>"#,
                    kw = ui::html_escape(&get_string(r, "Keyword").unwrap_or_default()),
                    ip = ui::html_escape(&get_string(r, "IP").unwrap_or_default()),
                    ts = ui::html_escape(&format_ts(get_i64(r, "TIME").unwrap_or_default())),
                    method = ui::html_escape(&get_string(r, "Request").unwrap_or_default()),
                )
            })
            .collect::<Vec<_>>()
            .join("")
    };
    ui::section_card("安全拦截", &ui::outlined_button_link("日志", "/index/index/log"), &list)
}

fn api_catalog(apis: &[ApiView], top: &[RankRow]) -> String {
    let counts: std::collections::HashMap<&str, i64> =
        top.iter().map(|r| (r.key.as_str(), r.count)).collect();
    let cards = apis
        .iter()
        .map(|api| {
            let c = counts.get(api.keyword.as_str()).copied().unwrap_or(0);
            format!(
                r#"<a class="api-card surface-card" data-name="{name}" data-kw="{kw}" href="/index/index/page?api={kw}">
  <div class="api-card-top"><strong>{name}</strong><span class="chip">{c} 次/周</span></div>
  <p>{des}</p>
  <code>/api/v2/{kw}</code>
</a>"#,
                name = ui::html_escape(&api.name),
                kw = urlencoding::encode(&api.keyword),
                des = ui::html_escape(&api.des),
                c = c,
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let inner = format!(
        r#"<div class="toolbar">
  <input class="field-input" id="api-search" placeholder="搜索接口名称 / Keyword" oninput="filterApis()">
  <span class="stat-note" id="api-count">{n} 个接口</span>
</div>
<div class="api-grid" id="api-grid">{cards}</div>"#,
        n = apis.len(),
        cards = cards,
    );
    ui::section_card("接口目录", "", &inner)
}

fn playground_panel(apis: &[ApiView], user: &UserView, base: &str, selected: Option<&str>) -> String {
    let options = apis
        .iter()
        .map(|a| {
            let sel = if selected == Some(a.keyword.as_str()) {
                " selected"
            } else {
                ""
            };
            format!(
                r#"<option value="{kw}"{sel}>{name} ({kw})</option>"#,
                kw = ui::html_escape(&a.keyword),
                name = ui::html_escape(&a.name),
                sel = sel,
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let inner = format!(
        r#"<div class="play-grid">
  <label class="field"><span class="field-label">接口</span>
    <select class="field-input" id="play-api">{options}</select>
  </label>
  <label class="field"><span class="field-label">方法</span>
    <select class="field-input" id="play-method"><option>GET</option><option>POST</option></select>
  </label>
  <label class="field" style="grid-column:1/-1"><span class="field-label">查询参数（一行一个 key=value）</span>
    <textarea class="field-input" id="play-params" rows="4" placeholder="type=json&#10;text=hello"></textarea>
  </label>
</div>
<div class="split-actions" style="margin:0.75rem 0">
  <button type="button" class="btn btn-filled" onclick="runPlay()">{icon}发送请求</button>
  <span class="stat-note">将自动附带 appid={uid}&amp;appkey=***</span>
</div>
<pre class="play-out" id="play-out">响应会显示在这里</pre>
<script>
window.__PLAY_BASE = {base};
window.__PLAY_APPID = {uid};
window.__PLAY_APPKEY = {key};
</script>"#,
        options = options,
        icon = ui::icon("play_arrow"),
        uid = user.uid,
        base = serde_json::to_string(base.trim_end_matches('/')).unwrap_or_else(|_| "\"\"".into()),
        key = serde_json::to_string(&user.appkey).unwrap_or_else(|_| "\"\"".into()),
    );
    ui::section_card("在线调试", "", &inner)
}

fn methods_chart(methods: &[RankRow]) -> String {
    let labels: Vec<String> = methods.iter().map(|m| m.key.clone()).collect();
    let data: Vec<i64> = methods.iter().map(|m| m.count).collect();
    let labels_json = serde_json::to_string(&labels).unwrap_or_else(|_| "[]".into());
    let data_json = serde_json::to_string(&data).unwrap_or_else(|_| "[]".into());
    let canvas = format!(
        r#"<div class="chart-box chart-box-sm"><canvas id="method-chart"></canvas></div>
<script>
(() => {{
  const el = document.getElementById('method-chart');
  if (!el || !window.Chart) return;
  const labels = {labels};
  const data = {data};
  if (!labels.length) return;
  new Chart(el, {{
    type: 'doughnut',
    data: {{
      labels,
      datasets: [{{
        data,
        backgroundColor: ['#006A6A','#4A6362','#4B607C','#9CF1F0','#D3E4FF','#CCE8E7'],
        borderWidth: 0
      }}]
    }},
    options: {{
      plugins: {{ legend: {{ position: 'bottom', labels: {{ boxWidth: 12, font: {{ family: 'Figtree' }} }} }} }},
      cutout: '62%',
      animation: {{ duration: 650 }}
    }}
  }});
}})();
</script>"#,
        labels = labels_json,
        data = data_json,
    );
    ui::section_card("请求方法占比（本周）", "", &canvas)
}

fn hourly_chart(hourly: &[i64]) -> String {
    let labels: Vec<String> = (0..24).map(|h| format!("{h:02}")).collect();
    let labels_json = serde_json::to_string(&labels).unwrap_or_else(|_| "[]".into());
    let data_json = serde_json::to_string(hourly).unwrap_or_else(|_| "[]".into());
    let canvas = format!(
        r#"<div class="chart-box chart-box-sm"><canvas id="hourly-chart"></canvas></div>
<script>
(() => {{
  const el = document.getElementById('hourly-chart');
  if (!el || !window.Chart) return;
  new Chart(el, {{
    type: 'bar',
    data: {{
      labels: {labels},
      datasets: [{{
        data: {data},
        backgroundColor: 'rgba(0,106,106,0.55)',
        borderRadius: 6,
        maxBarThickness: 14
      }}]
    }},
    options: {{
      plugins: {{ legend: {{ display: false }} }},
      scales: {{
        x: {{ grid: {{ display: false }}, ticks: {{ maxRotation: 0, autoSkip: true, maxTicksLimit: 12, color: '#3F4948' }} }},
        y: {{ beginAtZero: true, grid: {{ color: 'rgba(190,201,200,0.4)' }}, ticks: {{ precision: 0 }} }}
      }},
      animation: {{ duration: 650 }}
    }}
  }});
}})();
</script>"#,
        labels = labels_json,
        data = data_json,
    );
    ui::section_card("今日分时调用", "", &canvas)
}

fn request_table_full(rows: &[MySqlRow]) -> String {
    let export = serde_json::to_string(
        &rows
            .iter()
            .map(request_row_json)
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".into());
    let trs = rows
        .iter()
        .map(|row| {
            let ip = get_string(row, "IP").unwrap_or_default();
            let code = get_i64(row, "Code").unwrap_or(0);
            let badge = if code == 1 {
                r#"<span class="chip chip-warn">拦截</span>"#
            } else {
                r#"<span class="chip chip-good">正常</span>"#
            };
            format!(
                r#"<tr data-row="{kw} {ip} {ua}">
  <td>{ts}</td>
  <td><a href="/index/index/page?api={kwenc}">{kw}</a></td>
  <td><a href="javascript:void(0)" onclick="lookupIp('{ip}')">{ip}</a></td>
  <td>{method}</td>
  <td>{badge}</td>
  <td class="ua-cell">{ua}</td>
</tr>"#,
                ts = ui::html_escape(&format_ts(get_i64(row, "TIME").unwrap_or_default())),
                kw = ui::html_escape(&get_string(row, "Keyword").unwrap_or_default()),
                kwenc = urlencoding::encode(&get_string(row, "Keyword").unwrap_or_default()),
                ip = ui::html_escape(&ip),
                method = ui::html_escape(&get_string(row, "Request").unwrap_or_default()),
                badge = badge,
                ua = ui::html_escape(&get_string(row, "UserAgent").unwrap_or_default()),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<script>window.__LOG_EXPORT = {export};</script>
{table}"#,
        export = export,
        table = ui::data_table(
            &["时间", "接口", "IP", "方式", "状态", "UserAgent"],
            &trs,
        ),
    )
}

const DASH_SCRIPT: &str = r#"
<script>
function filterApis(){
  const q=(document.getElementById('api-search')?.value||'').toLowerCase();
  let n=0;
  document.querySelectorAll('#api-grid .api-card').forEach(el=>{
    const hit=!q || (el.dataset.name||'').toLowerCase().includes(q) || (el.dataset.kw||'').toLowerCase().includes(q);
    el.style.display=hit?'':'none';
    if(hit) n++;
  });
  const c=document.getElementById('api-count'); if(c) c.textContent=n+' 个接口';
}
function filterLogs(){
  const q=(document.getElementById('log-filter')?.value||'').toLowerCase();
  document.querySelectorAll('tr[data-row]').forEach(tr=>{
    tr.style.display=!q || (tr.dataset.row||'').toLowerCase().includes(q) ? '' : 'none';
  });
}
function exportLogs(){
  const blob=new Blob([JSON.stringify(window.__LOG_EXPORT||[],null,2)],{type:'application/json'});
  const a=document.createElement('a');
  a.href=URL.createObjectURL(blob);
  a.download='api-calls.json';
  a.click();
}
async function lookupIp(ip){
  try{
    const r=await fetch('/index/index/ip?address='+encodeURIComponent(ip));
    const d=await r.json();
    alert(JSON.stringify(d,null,2));
  }catch(e){ alert('查询失败'); }
}
async function runPlay(){
  const api=document.getElementById('play-api')?.value;
  const method=(document.getElementById('play-method')?.value||'GET').toUpperCase();
  const raw=document.getElementById('play-params')?.value||'';
  const out=document.getElementById('play-out');
  const params=new URLSearchParams();
  params.set('appid', String(window.__PLAY_APPID||''));
  params.set('appkey', String(window.__PLAY_APPKEY||''));
  params.set('type','json');
  raw.split(/\n/).forEach(line=>{
    const s=line.trim(); if(!s||s.startsWith('#')) return;
    const i=s.indexOf('=');
    if(i<0) params.append(s,''); else params.append(s.slice(0,i), s.slice(i+1));
  });
  const base=(window.__PLAY_BASE||'').replace(/\/$/,'');
  const url=base+'/api/v2/'+encodeURIComponent(api)+(method==='GET'?('?'+params.toString()):'');
  out.textContent='请求中…\n'+url;
  try{
    const init={method, headers:{}};
    if(method!=='GET'){
      init.headers['Content-Type']='application/x-www-form-urlencoded';
      init.body=params.toString();
    }
    const r=await fetch(url, init);
    const t=await r.text();
    let pretty=t;
    try{ pretty=JSON.stringify(JSON.parse(t),null,2);}catch(_){}
    out.textContent='HTTP '+r.status+'\n\n'+pretty;
  }catch(e){ out.textContent='请求失败: '+e; }
}
</script>
"#;


fn week_chart(stats: &Stats) -> String {
    let labels = serde_json::to_string(&["6天前", "5天前", "4天前", "3天前", "前天", "昨天", "今天"])
        .unwrap_or_else(|_| "[]".into());
    let data = serde_json::to_string(&[
        stats.day6,
        stats.day5,
        stats.day4,
        stats.day3,
        stats.day2,
        stats.yesterday,
        stats.today,
    ])
    .unwrap_or_else(|_| "[]".into());
    ui::section_card(
        "近七天调用",
        &format!(
            r#"<a class="btn btn-tonal" href="javascript:location.reload()">{}刷新</a>"#,
            ui::icon("refresh")
        ),
        &ui::chart_block("revenue-chart", &labels, &data),
    )
}

fn stat_cards(stats: &Stats) -> String {
    let today_to = signed_ratio(stats.today, stats.yesterday);
    let week_to = signed_ratio(stats.week, stats.lastweek);
    let month_to = signed_ratio(stats.month, stats.lastmonth);
    let security_to = if stats.count > 0 {
        stats.security_count as f64 / stats.count as f64 * 100.0
    } else {
        0.0
    };
    [
        (
            "今天调用",
            stats.today,
            format!("相对昨天"),
            ui::trend_chip(today_to, true),
        ),
        (
            "昨天调用",
            stats.yesterday,
            format!(
                "近六日 {}, {}, {}, {}, {}, {}",
                stats.day2, stats.day3, stats.day4, stats.day5, stats.day6, stats.yesterday
            ),
            String::new(),
        ),
        (
            "本周调用",
            stats.week,
            format!("上周 {}", stats.lastweek),
            ui::trend_chip(week_to, true),
        ),
        (
            "本月调用",
            stats.month,
            format!("上月 {}", stats.lastmonth),
            ui::trend_chip(month_to, true),
        ),
        (
            "总调用",
            stats.count,
            format!("拦截恶意 {}", stats.security_count),
            String::new(),
        ),
        (
            "拦截占比",
            stats.security_count,
            format!("{security_to:.2}% of total"),
            String::new(),
        ),
    ]
    .iter()
    .map(|(title, value, note, extra)| ui::stat_card(title, *value, note, extra))
    .collect::<Vec<_>>()
    .join("")
}

fn signed_ratio(a: i64, b: i64) -> f64 {
    if b == 0 {
        if a > 0 {
            100.0
        } else {
            0.0
        }
    } else {
        (a - b) as f64 / b as f64 * 100.0
    }
}

#[allow(dead_code)]
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
            let ip = get_string(row, "IP").unwrap_or_default();
            format!(
                r#"<tr><td>{}</td><td><a href="javascript:void(0)" onclick="fetch('/index/index/ip?address={}').then(r=>r.json()).then(d=>alert(JSON.stringify(d,null,2)))">{}</a></td><td>{}</td><td>{}</td></tr>"#,
                ui::html_escape(&format_ts(get_i64(row, "TIME").unwrap_or_default())),
                urlencoding::encode(&ip),
                ui::html_escape(&ip),
                ui::html_escape(&get_string(row, "UserAgent").unwrap_or_default()),
                ui::html_escape(&get_string(row, "Request").unwrap_or_default()),
            )
        })
        .collect::<Vec<_>>()
        .join("");
    ui::data_table(&["时间", "IP", "UserAgent", "方式"], &trs)
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
                .map(|cell| format!("<td>{}</td>", ui::html_escape(&value_to_string(cell))))
                .collect::<Vec<_>>()
                .join("");
            format!("<tr>{cells}</tr>")
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<div class="table-wrap"><table class="data-table"><tbody>{rows}</tbody></table></div>"#)
}

fn layout(
    title: &str,
    active: &str,
    user: &UserView,
    apis: &[ApiView],
    active_api: Option<&str>,
    body: &str,
) -> String {
    let api_links = apis
        .iter()
        .map(|api| ui::api_nav_link(&api.keyword, &api.name, active_api))
        .collect::<Vec<_>>()
        .join("");
    ui::app_layout(
        title,
        &user.username,
        &user.email,
        &headimg(&user.email),
        active,
        &api_links,
        body,
    )
}

fn simple_error(msg: &str) -> String {
    ui::error_page(msg)
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
