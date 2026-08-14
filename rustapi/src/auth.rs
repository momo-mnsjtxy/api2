use axum_extra::extract::cookie::{Cookie, CookieJar};
use tower_sessions::Session;

pub const COOKIE_UID: &str = "UID";
pub const COOKIE_USER: &str = "UserName";

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub uid: i64,
    pub username: String,
    pub email: String,
}

pub async fn current_user(session: &Session, jar: &CookieJar) -> Option<AuthUser> {
    let sid: Option<i64> = session.get("UID").await.ok().flatten();
    let semail: Option<String> = session.get("Email").await.ok().flatten();
    let sname: Option<String> = session.get("UserName").await.ok().flatten();
    let cuid = jar.get(COOKIE_UID).map(|c| c.value().to_string());
    let cname = jar.get(COOKIE_USER).map(|c| c.value().to_string());

    match (sid, semail, sname, cuid, cname) {
        (Some(uid), Some(email), Some(username), Some(cuid), Some(cname))
            if cuid == uid.to_string() && cname == username =>
        {
            Some(AuthUser {
                uid,
                username,
                email,
            })
        }
        _ => None,
    }
}

pub async fn login_user(
    session: &Session,
    jar: CookieJar,
    uid: i64,
    username: &str,
    email: &str,
) -> CookieJar {
    let _ = session.insert("UID", uid).await;
    let _ = session.insert("Email", email).await;
    let _ = session.insert("UserName", username).await;

    let jar = jar.add(
        Cookie::build((COOKIE_UID, uid.to_string()))
            .path("/")
            .max_age(time::Duration::seconds(86400))
            .build(),
    );
    jar.add(
        Cookie::build((COOKIE_USER, username.to_string()))
            .path("/")
            .max_age(time::Duration::seconds(86400))
            .build(),
    )
}

pub async fn logout_user(session: &Session, jar: CookieJar) -> CookieJar {
    let _ = session.remove::<i64>("UID").await;
    let _ = session.remove::<String>("Email").await;
    let _ = session.remove::<String>("UserName").await;
    jar.remove(Cookie::from(COOKIE_UID))
        .remove(Cookie::from(COOKIE_USER))
}
