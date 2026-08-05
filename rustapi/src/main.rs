pub mod auth;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod respond;
pub mod services;
pub mod state;
pub mod util;

use std::{collections::HashMap, net::SocketAddr};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Multipart, Path, Query, State},
    http::{HeaderMap, Method, Request},
    response::{IntoResponse, Response},
    routing::{any, get},
    Router,
};
use handlers::api::ApiCtx;
use state::AppState;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "api2=info,tower_http=info,sqlx=warn".into()
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = config::AppConfig::from_env()?;
    std::fs::create_dir_all(&cfg.cache_dir).ok();
    std::fs::create_dir_all(format!("{}/uploads", cfg.public_dir)).ok();
    std::fs::create_dir_all("runtime/mail").ok();

    let pool = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(20)
        .connect(&cfg.database_url)
        .await?;

    let cache = cache::AppCache::new(&cfg.cache_dir);
    let state = AppState::new(pool, cfg.clone(), cache);

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let public_dir = cfg.public_dir.clone();
    let app = Router::new()
        .route("/", get(handlers::web::index::home))
        // PATHINFO-compatible explicit mounts (oldapi style)
        .route("/api/v2", any(api_v2_index))
        .route("/api/v2/", any(api_v2_index))
        .route("/api/v2/upload", any(api_v2_upload_named))
        .route("/api/v2/Baidu_Upload", any(api_v2_baidu_upload))
        .route("/api/v2/Sogou_Upload", any(api_v2_sogou_upload))
        .route("/api/v2/:action", any(api_v2_action))
        .route("/api/page", any(api_page_action_default))
        .route("/api/page/", any(api_page_action_default))
        .route("/api/page/:action", any(api_page_action))
        .route("/api/update", any(api_update_action_default))
        .route("/api/update/", any(api_update_action_default))
        .route("/api/update/:action", any(api_update_action))
        .route("/api/skey", any(handlers::api::skey::index))
        .route("/api/skey/", any(handlers::api::skey::index))
        .route("/api/skey/index", any(handlers::api::skey::index))
        .route("/api/index", any(api_index_stub))
        .route("/api/index/", any(api_index_stub))
        .route("/api/index/index", any(api_index_stub))
        .route("/login", get(handlers::web::login::index))
        .route("/login/", get(handlers::web::login::index))
        .route("/login/index", get(handlers::web::login::index))
        .route("/login/index/", get(handlers::web::login::index))
        .route("/login/index/index", get(handlers::web::login::index))
        .route("/login/index/GtCode", any(handlers::web::login::gt_code))
        .route("/login/index/callback", any(handlers::web::login::callback))
        .route("/register", get(handlers::web::register::index))
        .route("/register/", get(handlers::web::register::index))
        .route("/register/index", get(handlers::web::register::index))
        .route("/register/index/", get(handlers::web::register::index))
        .route("/register/index/index", get(handlers::web::register::index))
        .route("/register/index/GtCode", any(handlers::web::register::gt_code))
        .route("/register/index/Email", any(handlers::web::register::email))
        .route(
            "/register/index/callback",
            any(handlers::web::register::callback),
        )
        .route("/index", get(handlers::web::index::index))
        .route("/index/", get(handlers::web::index::index))
        .route("/index/index", get(handlers::web::index::index))
        .route("/index/index/", get(handlers::web::index::index))
        .route("/index/index/index", get(handlers::web::index::index))
        .route("/index/index/page", any(handlers::web::index::page))
        .route("/index/index/setting", any(handlers::web::index::setting))
        .route("/index/index/appkey", any(handlers::web::index::appkey))
        .route("/index/index/LoginOut", any(handlers::web::index::LoginOut))
        .route("/index/index/ip", any(handlers::web::index::ip))
        .route("/index/index/log", any(handlers::web::index::log))
        // fallback PATHINFO: /{module}/{controller}/:action
        .route("/:module/:controller/:action", any(pathinfo_fallback))
        .nest_service("/public", ServeDir::new(public_dir))
        .layer(DefaultBodyLimit::max(12 * 1024 * 1024))
        .layer(session_layer)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = cfg.app_host.parse()?;
    tracing::info!("rustapi listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn api_v2_index(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let ctx = build_ctx(state, method, headers, query, None).await;
    handlers::api::v2::dispatch("index", ctx).await
}

async fn api_v2_action(
    State(state): State<AppState>,
    Path(action): Path<String>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    req: Request<axum::body::Body>,
) -> Response {
    let (params, file) = collect_params(query, req).await;
    let ctx = build_ctx(state, method, headers, params, file).await;
    handlers::api::v2::dispatch(&action, ctx).await
}

async fn api_v2_upload_named(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    multipart: Multipart,
) -> Response {
    api_v2_multipart(state, method, headers, query, multipart, "upload").await
}

async fn api_v2_baidu_upload(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    multipart: Multipart,
) -> Response {
    api_v2_multipart(state, method, headers, query, multipart, "Baidu_Upload").await
}

async fn api_v2_sogou_upload(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    multipart: Multipart,
) -> Response {
    api_v2_multipart(state, method, headers, query, multipart, "Sogou_Upload").await
}

async fn api_v2_multipart(
    state: AppState,
    method: Method,
    headers: HeaderMap,
    mut query: HashMap<String, String>,
    mut multipart: Multipart,
    action: &str,
) -> Response {
    let mut file = None;
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" || name == "image" {
            let filename = field.file_name().unwrap_or("upload.bin").to_string();
            if let Ok(data) = field.bytes().await {
                file = Some((filename, data.to_vec()));
            }
        } else if let Ok(text) = field.text().await {
            query.insert(name, text);
        }
    }
    let ctx = build_ctx(state, method, headers, query, file).await;
    handlers::api::v2::dispatch(action, ctx).await
}

async fn api_page_action_default(
    State(state): State<AppState>,
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    dispatch_page("index", state, params, headers, method).await
}

async fn api_page_action(
    State(state): State<AppState>,
    Path(action): Path<String>,
    method: Method,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    dispatch_page(&action, state, params, headers, method).await
}

async fn dispatch_page(
    action: &str,
    state: AppState,
    params: HashMap<String, String>,
    headers: HeaderMap,
    method: Method,
) -> Response {
    match action {
        "index" => handlers::api::page::index(Query(params)).await,
        "netease" => handlers::api::page::netease(State(state), Query(params), headers, method).await,
        "autograph" => {
            handlers::api::page::autograph(State(state), Query(params), headers, method).await
        }
        "love" => handlers::api::page::love(State(state), Query(params), headers, method).await,
        "name" => handlers::api::page::name(State(state), Query(params), headers, method).await,
        "shuoshuo" => {
            handlers::api::page::shuoshuo(State(state), Query(params), headers, method).await
        }
        "word" => handlers::api::page::word(State(state), Query(params), headers, method).await,
        "log" => handlers::api::page::log(State(state), Query(params)).await,
        "likes" => handlers::api::page::likes(State(state), Query(params)).await,
        "headimg" => {
            handlers::api::page::headimg(State(state), Query(params), headers, method).await
        }
        _ => handlers::api::page::index(Query(params)).await,
    }
}

async fn api_update_action_default(State(state): State<AppState>) -> Response {
    let _ = state;
    handlers::api::update::index().await
}

async fn api_update_action(
    State(state): State<AppState>,
    Path(action): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    match action.as_str() {
        "index" => handlers::api::update::index().await,
        "Music_hot" => handlers::api::update::music_hot(State(state)).await,
        "Word" => handlers::api::update::word(State(state)).await,
        "Wordtime" => handlers::api::update::wordtime(State(state), Query(params)).await,
        "netease" => handlers::api::update::netease(State(state)).await,
        "headimg" => handlers::api::update::headimg(State(state), Query(params)).await,
        "words" => handlers::api::update::words(State(state), Query(params)).await,
        "qianming" => handlers::api::update::qianming(State(state), Query(params)).await,
        "wangming" => handlers::api::update::wangming(State(state), Query(params)).await,
        "net" => handlers::api::update::net(State(state)).await,
        "dog" => handlers::api::update::dog(State(state)).await,
        _ => handlers::api::update::index().await,
    }
}

async fn api_index_stub() -> Response {
    handlers::api::index::index().await
}

async fn pathinfo_fallback(
    State(state): State<AppState>,
    Path((module, controller, action)): Path<(String, String, String)>,
    method: Method,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
    req: Request<axum::body::Body>,
) -> Response {
    let module = module.to_ascii_lowercase();
    let controller = controller.to_ascii_lowercase();
    match (module.as_str(), controller.as_str()) {
        ("api", "v2") => {
            let (params, file) = collect_params(query, req).await;
            let ctx = build_ctx(state, method, headers, params, file).await;
            handlers::api::v2::dispatch(&action, ctx).await
        }
        ("api", "page") => dispatch_page(&action, state, query, headers, method).await,
        ("api", "update") => {
            api_update_action(State(state), Path(action), Query(query)).await
        }
        ("api", "skey") => handlers::api::skey::index(Query(query)).await,
        ("api", "index") => handlers::api::index::index().await,
        _ => (
            axum::http::StatusCode::NOT_FOUND,
            format!("not found: /{module}/{controller}/:action"),
        )
            .into_response(),
    }
}

async fn build_ctx(
    state: AppState,
    method: Method,
    headers: HeaderMap,
    params: HashMap<String, String>,
    file: Option<(String, Vec<u8>)>,
) -> ApiCtx {
    let ua = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let ip = util::client_ip(&headers, None);
    ApiCtx {
        state,
        params,
        method: method.to_string(),
        ua,
        ip,
        headers,
        file,
    }
}

async fn collect_params(
    mut query: HashMap<String, String>,
    req: Request<axum::body::Body>,
) -> (HashMap<String, String>, Option<(String, Vec<u8>)>) {
    let content_type = req
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if content_type.starts_with("multipart/form-data") {
        // Re-extract via Multipart is hard from Request; read body bytes and skip for now.
        // Upload endpoints that need multipart should be hit with dedicated extractor later.
        let bytes = axum::body::to_bytes(req.into_body(), 12 * 1024 * 1024)
            .await
            .unwrap_or_else(|_| Bytes::new());
        // Best-effort: no multipart parse here; file remains None unless boundary parsed.
        let _ = bytes;
        return (query, None);
    }

    if content_type.contains("application/x-www-form-urlencoded")
        || req.method() == Method::POST
        || req.method() == Method::PUT
    {
        if let Ok(bytes) = axum::body::to_bytes(req.into_body(), 2 * 1024 * 1024).await {
            if let Ok(map) = serde_urlencoded::from_bytes::<HashMap<String, String>>(&bytes) {
                for (k, v) in map {
                    query.entry(k).or_insert(v);
                }
            }
        }
    }

    (query, None)
}
