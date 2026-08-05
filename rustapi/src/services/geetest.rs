use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::util;

/// GeeTest CAPTCHA v3 server SDK (ported from gt3-php-sdk `GeetestLib`).
pub struct Geetest {
    pub captcha_id: String,
    pub private_key: String,
    response: Value,
}

impl Geetest {
    pub fn new(id: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            captcha_id: id.into(),
            private_key: key.into(),
            response: json!({}),
        }
    }

    pub fn response_json(&self) -> Value {
        self.response.clone()
    }

    /// Register with GeeTest; returns 1 on success, 0 for failback.
    pub async fn pre_process(
        &mut self,
        http: &Client,
        param: &[(&str, &str)],
        new_captcha: i32,
    ) -> i32 {
        let mut query: Vec<(&str, String)> = vec![
            ("gt", self.captcha_id.clone()),
            ("new_captcha", new_captcha.to_string()),
        ];
        for (k, v) in param {
            query.push((*k, (*v).to_string()));
        }
        let url = match reqwest::Url::parse_with_params("http://api.geetest.com/register.php", &query)
        {
            Ok(u) => u,
            Err(_) => {
                self.failback_process();
                return 0;
            }
        };
        let challenge = match http
            .get(url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            Ok(resp) => resp.text().await.unwrap_or_default(),
            Err(_) => String::new(),
        };
        let challenge = challenge.trim().to_string();
        if challenge.len() != 32 {
            self.failback_process();
            return 0;
        }
        self.success_process(&challenge);
        1
    }

    fn success_process(&mut self, challenge: &str) {
        let challenge = util::md5_hex(&format!("{challenge}{}", self.private_key));
        self.response = json!({
            "success": 1,
            "gt": self.captcha_id,
            "challenge": challenge,
            "new_captcha": 1,
        });
    }

    fn failback_process(&mut self) {
        let rnd1 = util::md5_hex(&rand::random::<u32>().to_string());
        let rnd2 = util::md5_hex(&rand::random::<u32>().to_string());
        let challenge = format!("{rnd1}{}", &rnd2[..2.min(rnd2.len())]);
        self.response = json!({
            "success": 0,
            "gt": self.captcha_id,
            "challenge": challenge,
            "new_captcha": 1,
        });
    }

    /// Online secondary validation against GeeTest validate.php.
    pub async fn success_validate(
        &self,
        http: &Client,
        challenge: &str,
        validate: &str,
        seccode: &str,
        param: &[(&str, &str)],
    ) -> bool {
        if !self.check_validate(challenge, validate) {
            return false;
        }
        let mut form = HashMap::new();
        form.insert("seccode".to_string(), seccode.to_string());
        form.insert(
            "timestamp".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );
        form.insert("challenge".to_string(), challenge.to_string());
        form.insert("captchaid".to_string(), self.captcha_id.clone());
        form.insert("json_format".to_string(), "1".into());
        form.insert("sdk".to_string(), "rust_3.0.0".into());
        for (k, v) in param {
            form.insert((*k).to_string(), (*v).to_string());
        }
        let text = match http
            .post("http://api.geetest.com/validate.php")
            .form(&form)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) => resp.text().await.unwrap_or_default(),
            Err(_) => return false,
        };
        let expect = util::md5_hex(seccode);
        let Ok(obj) = serde_json::from_str::<Value>(&text) else {
            return text.trim() == expect;
        };
        obj.get("seccode")
            .and_then(Value::as_str)
            .map(|s| s == expect)
            .unwrap_or(false)
    }

    /// Failback secondary validation (gt3: md5(challenge) == validate).
    pub fn fail_validate(&self, challenge: &str, validate: &str, _seccode: &str) -> bool {
        if challenge.is_empty() || validate.is_empty() {
            return false;
        }
        util::md5_hex(challenge) == validate
    }

    fn check_validate(&self, challenge: &str, validate: &str) -> bool {
        validate.len() == 32
            && util::md5_hex(&format!("{}geetest{challenge}", self.private_key)) == validate
    }
}
