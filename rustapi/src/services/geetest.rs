use serde_json::{json, Value};
use uuid::Uuid;

pub struct Geetest {
    pub captcha_id: String,
    #[allow(dead_code)]
    pub private_key: String,
}

impl Geetest {
    pub fn new(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            captcha_id: id.into(),
            private_key: key.into(),
        }
    }

    /// Returns 0 = failback mode (matches stub/old failback path).
    pub fn pre_process(&self) -> i32 {
        0
    }

    pub fn response_json(&self) -> Value {
        json!({
            "success": 0,
            "gt": self.captcha_id,
            "challenge": Uuid::new_v4().simple().to_string(),
            "new_captcha": true,
        })
    }

    pub fn fail_validate(&self, challenge: &str, _validate: &str, _seccode: &str) -> bool {
        !challenge.is_empty()
    }

    pub fn success_validate(&self, challenge: &str, validate: &str, seccode: &str) -> bool {
        self.fail_validate(challenge, validate, seccode)
    }
}
