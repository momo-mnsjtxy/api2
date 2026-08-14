use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app_host: String,
    pub app_url: String,
    pub database_url: String,
    pub session_secret: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    pub geetest_id: String,
    pub geetest_key: String,
    pub public_dir: String,
    pub cache_dir: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Self {
            app_host: std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            app_url: std::env::var("APP_URL").unwrap_or_default(),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://api:api_0324@127.0.0.1:3306/api".into()),
            session_secret: std::env::var("SESSION_SECRET")
                .unwrap_or_else(|_| "dev-secret-change-me".into()),
            smtp_host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.exmail.qq.com".into()),
            smtp_port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(465),
            smtp_user: std::env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: std::env::var("SMTP_PASS").unwrap_or_default(),
            smtp_from: std::env::var("SMTP_FROM").unwrap_or_else(|_| "info@gqink.cn".into()),
            geetest_id: std::env::var("GEETEST_ID")
                .unwrap_or_else(|_| "geetest_id_placeholder".into()),
            geetest_key: std::env::var("GEETEST_KEY")
                .unwrap_or_else(|_| "geetest_key_placeholder".into()),
            public_dir: std::env::var("PUBLIC_DIR").unwrap_or_else(|_| "./public".into()),
            cache_dir: std::env::var("CACHE_DIR").unwrap_or_else(|_| "./runtime/cache".into()),
        })
    }
}
