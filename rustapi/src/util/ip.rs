use axum::http::HeaderMap;

pub fn client_ip(headers: &HeaderMap, fallback: Option<&str>) -> String {
    if let Some(v) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        return v.split(',').next().unwrap_or(v).trim().to_string();
    }
    if let Some(v) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return v.to_string();
    }
    if let Some(v) = headers.get("client-ip").and_then(|v| v.to_str().ok()) {
        return v.to_string();
    }
    fallback.unwrap_or("0.0.0.0").to_string()
}
