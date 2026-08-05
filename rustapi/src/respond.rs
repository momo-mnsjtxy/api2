use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Local;
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Clone, Serialize)]
pub struct Copyright {
    pub name: &'static str,
    pub url: &'static str,
    pub time: String,
}

pub fn copyright() -> Copyright {
    Copyright {
        name: "与梦城",
        url: "https://www.gqink.cn",
        time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    }
}

/// ThinkPHP `INT($type, $data)` — json default, xml when type=xml.
pub fn int(output_type: Option<&str>, data: Value) -> Response {
    match output_type.map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("xml") => xml_response(&data),
        _ => json_response(data),
    }
}

pub fn json_response(data: Value) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        data.to_string(),
    )
        .into_response()
}

pub fn xml_response(data: &Value) -> Response {
    let body = value_to_xml(data, "root");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/xml; charset=utf-8")],
        body,
    )
        .into_response()
}

pub fn ok_data(data: Value) -> Value {
    json!({
        "code": 200,
        "Copyright": copyright(),
        "data": data,
    })
}

pub fn err_msg(code: i64, msg: impl Into<String>) -> Value {
    json!({"code": code, "msg": msg.into()})
}

fn value_to_xml(value: &Value, tag: &str) -> String {
    match value {
        Value::Null => format!("<{tag}></{tag}>"),
        Value::Bool(b) => format!("<{tag}>{}</{tag}>", b),
        Value::Number(n) => format!("<{tag}>{n}</{tag}>"),
        Value::String(s) => format!("<{tag}>{}</{tag}>", xml_escape(s)),
        Value::Array(arr) => {
            let mut out = String::new();
            for (i, item) in arr.iter().enumerate() {
                out.push_str(&value_to_xml(item, &format!("item{i}")));
            }
            format!("<{tag}>{out}</{tag}>")
        }
        Value::Object(map) => {
            let mut out = String::new();
            for (k, v) in map {
                let safe = sanitize_tag(k);
                out.push_str(&value_to_xml(v, &safe));
            }
            format!("<{tag}>{out}</{tag}>")
        }
    }
}

fn sanitize_tag(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() || s.as_bytes()[0].is_ascii_digit() {
        format!("n_{s}")
    } else {
        s
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
