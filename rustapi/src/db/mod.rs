use crate::error::AppResult;
use crate::util::now_ts;
use sqlx::{MySql, MySqlPool, Row};

pub async fn insert_log(
    pool: &MySqlPool,
    name: &str,
    ua: &str,
    method: &str,
    ip: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO log (name, time, UserAgent, request, ip) VALUES (?, ?, ?, ?, ?)")
        .bind(name)
        .bind(now_ts())
        .bind(ua)
        .bind(method)
        .bind(ip)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn random_row(pool: &MySqlPool, table: &str) -> AppResult<Option<sqlx::mysql::MySqlRow>> {
    // table names are internal constants only
    let sql = format!("SELECT * FROM `{table}` ORDER BY RAND() LIMIT 1");
    let row = sqlx::query(&sql).fetch_optional(pool).await?;
    Ok(row)
}

pub async fn count_table(pool: &MySqlPool, table: &str) -> AppResult<i64> {
    let sql = format!("SELECT COUNT(*) AS c FROM `{table}`");
    let row = sqlx::query(&sql).fetch_one(pool).await?;
    Ok(row.try_get::<i64, _>("c").unwrap_or(0))
}

pub async fn page_rows(
    pool: &MySqlPool,
    table: &str,
    page: i64,
    page_num: i64,
) -> AppResult<Vec<sqlx::mysql::MySqlRow>> {
    let offset = (page.max(1) - 1) * page_num;
    let sql = format!("SELECT * FROM `{table}` ORDER BY id DESC LIMIT ? OFFSET ?");
    let rows = sqlx::query(&sql)
        .bind(page_num)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn find_user_by_login(
    pool: &MySqlPool,
    account: &str,
    password_md5: &str,
) -> AppResult<Option<(i64, String, String)>> {
    let row = if account.contains('@') {
        sqlx::query("SELECT UID, UserName, Email FROM user WHERE Email=? AND Password=? LIMIT 1")
            .bind(account)
            .bind(password_md5)
            .fetch_optional(pool)
            .await?
    } else {
        sqlx::query("SELECT UID, UserName, Email FROM user WHERE UserName=? AND Password=? LIMIT 1")
            .bind(account)
            .bind(password_md5)
            .fetch_optional(pool)
            .await?
    };
    Ok(row.map(|r| {
        (
            r.get::<i64, _>("UID"),
            r.get::<String, _>("UserName"),
            r.get::<String, _>("Email"),
        )
    }))
}

pub type Db = MySqlPool;
pub type DbRow = <MySql as sqlx::Database>::Row;
