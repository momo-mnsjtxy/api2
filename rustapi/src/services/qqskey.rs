use serde_json::{json, Value};

/// Placeholder for extend/qqskey/login.class.php — preserves `do` dispatch surface.
pub struct QqLogin;

impl QqLogin {
    pub fn new() -> Self {
        Self
    }

    pub fn dispatch(&self, do_cmd: &str, params: &[(String, String)]) -> Value {
        let get = |k: &str| {
            params
                .iter()
                .find(|(a, _)| a == k)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        match do_cmd {
            "getqrpic" | "idpic" | "getqrpic3rd" => json!({
                "code": 200,
                "msg": "qqskey stub — port login.class.php for full behavior",
                "do": do_cmd,
            }),
            "checkvc" | "dovc" | "getvc" | "qqlogin" | "qrlogin" | "list" | "danxiang" | "del"
            | "authf" | "qrlogin3rd" | "idlogin" | "InfoNull" | "NickNull" => json!({
                "code": 400,
                "msg": format!("qqskey action '{do_cmd}' requires full qq_login port"),
                "uin": get("uin"),
            }),
            _ => json!({"code": 400, "msg": "unknown do"}),
        }
    }
}

impl Default for QqLogin {
    fn default() -> Self {
        Self::new()
    }
}
