use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use image::GenericImageView;
use md5::{Digest, Md5};
use rand::Rng;
use regex::Regex;
use reqwest::{
    header::{
        HeaderMap, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONNECTION, COOKIE, REFERER,
        SET_COOKIE, USER_AGENT,
    },
    Client, Method,
};
use serde_json::{json, Value};
use url::form_urlencoded;

const QQ_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/69.0.3497.100 Safari/537.36";
const XLOGIN_REFERRER: &str = "https://xui.ptlogin2.qq.com/cgi-bin/xlogin";

pub struct QqLogin {
    client: Client,
    referrer: Option<String>,
    trace_x: i64,
    trace_y: i64,
    trace_time: i64,
}

struct CurlResponse {
    header_text: String,
    body: Vec<u8>,
}

impl CurlResponse {
    fn combined_text(&self) -> String {
        format!(
            "{}{}",
            self.header_text,
            String::from_utf8_lossy(&self.body)
        )
    }
}

impl QqLogin {
    pub fn new(client: &Client) -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .user_agent(QQ_UA)
            .build()
            .unwrap_or_else(|_| client.clone());

        Self {
            client,
            referrer: None,
            trace_x: 0,
            trace_y: 0,
            trace_time: 0,
        }
    }

    pub async fn dispatch(&mut self, do_cmd: &str, params: &[(String, String)]) -> Value {
        let get = |k: &str| {
            params
                .iter()
                .find(|(a, _)| a == k)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };

        match do_cmd {
            "checkvc" => {
                let uin = get("uin");
                let tokenid = get("tokenid");
                self.checkvc(&uin, &tokenid).await
            }
            "dovc" => {
                let uin = get("uin");
                let sig = get("sig");
                let ans = get("ans");
                let cap_cd = get("cap_cd");
                let sess = get("sess");
                let collectname = get("collectname");
                let websig = get("websig");
                let cdata = get("cdata");
                let sid = get("sid");
                self.dovc(
                    &uin,
                    &sig,
                    &ans,
                    &cap_cd,
                    &sess,
                    &collectname,
                    &websig,
                    &cdata,
                    &sid,
                )
                .await
            }
            "getvc" => {
                let uin = get("uin");
                let sig = get("sig");
                let sess = get("sess");
                let sid = get("sid");
                let websig = get("websig");
                self.getvc(&uin, &sig, &sess, &sid, &websig).await
            }
            "qqlogin" => {
                let uin = get("uin");
                let pwd = get("pwd");
                let p = get("p");
                let vcode = get("vcode");
                let pt_verifysession = get("pt_verifysession");
                let cookie = get("cookie");
                self.qqlogin(&uin, &pwd, &p, &vcode, &pt_verifysession, &cookie)
                    .await
            }
            "getqrpic" => self.getqrpic().await,
            "qrlogin" => {
                let qrsig = get("qrsig");
                self.qrlogin(&qrsig).await
            }
            "getqrpic3rd" => {
                let daid = get("daid");
                let appid = get("appid");
                self.getqrpic3rd(&daid, &appid).await
            }
            "qrlogin3rd" => {
                let daid = get("daid");
                let appid = get("appid");
                let qrsig = get("qrsig");
                self.qrlogin3rd(&daid, &appid, &qrsig).await
            }
            "idpic" => self.idpic().await,
            "idlogin" => {
                let qrsig = get("qrsig");
                self.idlogin(&qrsig).await
            }
            "list" => {
                let uin = get("uin");
                let skey = get("skey");
                let p_skey = get("p_skey");
                self.list(&uin, &skey, &p_skey).await
            }
            "danxiang" => {
                let uin = get("uin");
                let qq = get("qq");
                let skey = get("skey");
                self.danxiang(&uin, &qq, &skey).await
            }
            "del" => {
                let uin = get("uin");
                let qq = get("qq");
                let skey = get("skey");
                let p_skey = get("p_skey");
                self.del(&uin, &qq, &skey, &p_skey).await
            }
            "authf" => {
                let uin = get("uin");
                let skey = get("skey");
                let p_skey = get("p_skey");
                let ptcz = get("ptcz");
                let rk = get("RK");
                self.authf(&uin, &skey, &p_skey, &ptcz, &rk).await
            }
            "InfoNull" => {
                let uin = get("uin");
                let skey = get("skey");
                let p_skey = get("p_skey");
                let rk = get("RK");
                self.InfoNull(&uin, &skey, &p_skey, &rk).await
            }
            "NickNull" => {
                let uin = get("uin");
                let skey = get("skey");
                let p_skey = get("p_skey");
                let rk = get("RK");
                self.NickNull(&uin, &skey, &p_skey, &rk).await
            }
            _ => json!({"code": 400, "msg": "unknown do"}),
        }
    }

    pub async fn dovc(
        &mut self,
        uin: &str,
        sig: &str,
        ans: &str,
        cap_cd: &str,
        sess: &str,
        collectname: &str,
        websig: &str,
        cdata: &str,
        sid: &str,
    ) -> Value {
        if php_empty(uin) {
            return json!({"saveOK": -1, "msg": "QQ不能为空"});
        }
        if php_empty(sig) {
            return json!({"saveOK": -1, "msg": "sig不能为空"});
        }
        if php_empty(ans) {
            return json!({"saveOK": -1, "msg": "验证码不能为空"});
        }
        if php_empty(cap_cd) {
            return json!({"saveOK": -1, "msg": "cap_cd不能为空"});
        }
        if php_empty(sess) {
            return json!({"saveOK": -1, "msg": "sess不能为空"});
        }
        if php_empty(sid) {
            return json!({"saveOK": -1, "msg": "sid不能为空"});
        }

        let collectname = if collectname.is_empty() {
            "collect"
        } else {
            collectname
        };
        let width = ans
            .split(',')
            .next()
            .unwrap_or_default()
            .parse()
            .unwrap_or(0);
        let collect = self.getcollect(width).await;

        let dfp_url = "https://ssl.captcha.qq.com/dfpReg?0=Mozilla%2F5.0%20(Windows%20NT%2010.0%3B%20WOW64)%20AppleWebKit%2F537.36%20(KHTML%2C%20like%20Gecko)%20Chrome%2F69.0.3497.100%20Safari%2F537.36&1=zh-CN&2=1.5&3=1.6&4=24&5=8&6=-480&7=1&8=1&9=1&10=u&11=function&12=u&13=Win32&14=0&15=f14d5531d44759dfdac2c422c0276dde&16=408d1e375fc96dcedebc2b02d580bac6&17=a1f937b6ee969f22e6122bdb5cb48bde&18=10x2x102x69&19=702efbf2d84a0bfb7224d4a1bfe36e0a&20=872136824912136824&21=1%3B&22=1%3B1%3B1%3B1%3B1%3B1%3B1%3B0%3B1%3Bobject27UTF-8&23=0&24=10%3B1&25=126a2202136b27316760a4f9c2c2e1a9&26=48000_2_1_0_2_explicit_speakers&27=d7959e801195e05311be04517d04a522&28=ANGLE(Intel(R)UHDGraphics620Direct3D11vs_5_0ps_5_0)&29=60f09e9c459c29f92ce6fc61751ea45b&30=9c04b80df743b5904a3835fbc06a476e&31=0&32=0&33=0&34=0&35=0&36=0&37=0&38=0&39=0&40=0&41=0&42=0&43=0&44=0&45=0&46=0&47=0&48=0&49=0&50=0&fesig=5744539509613183248&ut=391&appid=0&refer=https%3A%2F%2Fssl.captcha.qq.com%2Fcap_union_new_show&domain=ssl.captcha.qq.com&fph=&fpv=0.0.15&ptcz=";
        let referer = self.referrer.as_deref();
        let fpsig_data = self
            .curl_text(dfp_url, None, referer, None, "application/json")
            .await;
        let fpsig_json = json_decode(&fpsig_data);
        let fpsig = str_field(&fpsig_json, "fpsig");
        let collectdata = str_field(&collect, "collectdata");
        let eks = str_field(&collect, "eks");
        let tlg = urlencoding::decode(&collectdata)
            .map(|s| s.len())
            .unwrap_or_else(|_| collectdata.len());
        let post = format!(
            "aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&color=&showtype=&fb=1&theme=&lang=2052&ua={}&grayscale=1&subsid=2&sess={sess}&fwidth=0&sid={sid}&forcestyle=0&wxLang=&tcScale=1&uid={uin}&cap_cd={cap_cd}&rnd={}&TCapIframeLoadTime=99&prehandleLoadTime=48&createIframeStart={}758&rand=0.330608{}&subcapclass=13&vsig={sig}&ans={ans}&{collectname}={collectdata}&websig={websig}&cdata={cdata}&fpinfo=fpsig%3D{fpsig}&eks={eks}&tlg={tlg}&vlg=0_0_0",
            encoded_ua(),
            rand_int(100000, 999999),
            now(),
            now()
        );
        let data = self
            .curl_text(
                "https://ssl.captcha.qq.com/cap_union_new_verify",
                Some(&post),
                self.referrer.as_deref(),
                None,
                "application/json",
            )
            .await;
        let arr = json_decode(&data);
        let error_code = arr.get("errorCode").and_then(Value::as_i64);

        match error_code {
            Some(0) => json!({
                "rcode": 0,
                "randstr": arr.get("randstr").cloned().unwrap_or(json!("")),
                "sig": arr.get("ticket").cloned().unwrap_or(json!("")),
            }),
            Some(50) => json!({"rcode": 50, "errmsg": "验证码输入错误！"}),
            _ => json!({
                "rcode": 9,
                "errmsg": arr.get("errMessage").and_then(Value::as_str).unwrap_or(""),
            }),
        }
    }

    pub async fn qqlogin(
        &self,
        uin: &str,
        pwd: &str,
        p: &str,
        vcode: &str,
        pt_verifysession: &str,
        cookie: &str,
    ) -> Value {
        if php_empty(uin) {
            return json!({"saveOK": -1, "msg": "QQ不能为空"});
        }
        if php_empty(pwd) {
            return json!({"saveOK": -1, "msg": "pwd不能为空"});
        }
        if php_empty(p) {
            return json!({"saveOK": -1, "msg": "密码不能为空"});
        }
        if php_empty(vcode) {
            return json!({"saveOK": -1, "msg": "验证码不能为空"});
        }
        if php_empty(pt_verifysession) {
            return json!({"saveOK": -1, "msg": "pt_verifysession不能为空"});
        }

        let v1 = if format!("s{vcode}").contains('!') {
            0
        } else {
            1
        };
        let pt_login_sig = cookie_value(cookie, "pt_login_sig");
        let ptdrvs = cookie_value(cookie, "ptdrvs");
        let url = format!(
            "https://ssl.ptlogin2.qq.com/login?u={uin}&verifycode={vcode}&pt_vcode_v1={v1}&pt_verifysession_v1={pt_verifysession}&p={p}&pt_randsalt=2&u1=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&ptredirect=0&h=1&t=1&g=1&from_ui=1&ptlang=2052&action=5-10-{}487&js_ver=19092321&js_type=1&login_sig={pt_login_sig}&pt_uistyle=40&aid=549000912&daid=5&ptdrvs={ptdrvs}&",
            now()
        );
        let referrer = "https://xui.ptlogin2.qq.com/cgi-bin/xlogin?proxy_url=https%3A//qzs.qq.com/qzone/v6/portal/proxy.html&daid=5&&hide_title_bar=1&low_login=0&qlogin_auto_login=0&no_verifyimg=1&link_target=blank&appid=549000912&style=22&target=self&s_url=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&pt_no_auth=0";
        let ret = self
            .curl_with_headers_text(&url, None, Some(referrer), Some(cookie), "application/json")
            .await;

        let Some(inner) = capture(r"ptuiCB\('(.*?)'\)", &ret) else {
            return json!({"saveOK": -2, "msg": ret});
        };
        let r = split_callback(&inner);
        let code = r.first().map(String::as_str).unwrap_or_default();
        match code {
            "0" => {
                let loginurl = r.get(2).cloned().unwrap_or_default();
                if loginurl.contains("mibao_vry") {
                    return json!({"saveOK": -3, "msg": "请先到QQ安全中心关闭网页登录保护！"});
                }
                let skey = capture(r"skey=@(.{9});", &ret).unwrap_or_default();
                let superkey = capture(r"superkey=(.*?);", &ret).unwrap_or_default();
                let data = self
                    .curl_with_headers_text(&loginurl, None, None, None, "application/json")
                    .await;
                let pskey = capture(r"p_skey=(.*?);", &data).unwrap_or_default();
                let sid = capture(r"(?i)Location: (.*?)\r\n", &data)
                    .and_then(|location| location.split("sid=").nth(1).map(ToOwned::to_owned))
                    .unwrap_or_default();

                if !skey.is_empty() && !pskey.is_empty() {
                    json!({
                        "saveOK": 0,
                        "uin": uin,
                        "sid": sid,
                        "skey": format!("@{skey}"),
                        "pskey": pskey,
                        "superkey": superkey,
                        "nick": php_urlencode(r.get(5).map(String::as_str).unwrap_or_default()),
                        "loginurl": loginurl,
                    })
                } else if pskey.is_empty() {
                    json!({"saveOK": -3, "msg": format!("登录成功，获取P_skey失败！{loginurl}")})
                } else {
                    json!({"saveOK": -3, "msg": "登录成功，获取SID失败！"})
                }
            }
            "4" => json!({"saveOK": 4, "msg": "验证码错误"}),
            "3" => json!({"saveOK": 3, "msg": "密码错误"}),
            "19" => {
                json!({"saveOK": 19, "uin": uin, "msg": "您的帐号暂时无法登录，请到 http://aq.qq.com/007 恢复正常使用"})
            }
            _ => json!({"saveOK": -6, "msg": r.get(4).cloned().unwrap_or_default()}),
        }
    }

    pub async fn getvc(
        &mut self,
        uin: &str,
        sig: &str,
        sess: &str,
        sid: &str,
        websig: &str,
    ) -> Value {
        if php_empty(uin) {
            return json!({"saveOK": -1, "msg": "请先输入QQ号码"});
        }
        if php_empty(sig) {
            return json!({"saveOK": -1, "msg": "SIG不能为空"});
        }
        if !valid_qq(uin, 12) {
            return json!({"saveOK": -2, "msg": "QQ号码不正确"});
        }

        if sess == "0" {
            let prehandle_url = format!(
                "https://ssl.captcha.qq.com/cap_union_prehandle?aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&color=&showtype=&fb=1&theme=&lang=2052&ua={}&grayscale=1&cap_cd={sig}&uid={uin}&subsid=1&callback=&sess=",
                encoded_ua()
            );
            let data = self
                .curl_text(
                    &prehandle_url,
                    None,
                    Some(XLOGIN_REFERRER),
                    None,
                    "application/json",
                )
                .await;
            let trimmed = if data.len() > 1 {
                &data[1..data.len().saturating_sub(1)]
            } else {
                ""
            };
            let arr = json_decode(trimmed);
            let sess = str_field(&arr, "sess");
            let sid = str_field(&arr, "sid");
            if sess.is_empty() {
                return json!({"saveOK": -3, "msg": "获取验证码参数失败"});
            }

            let show_url = format!(
                "https://ssl.captcha.qq.com/cap_union_new_show?aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&color=&showtype=&fb=1&theme=&lang=2052&ua={}&grayscale=1&subsid=2&sess={sess}&fwidth=0&sid={sid}&forcestyle=0&wxLang=&tcScale=1&uid={uin}&cap_cd={sig}&rnd={}&TCapIframeLoadTime=48&prehandleLoadTime=46&createIframeStart={}353",
                encoded_ua(),
                rand_int(100000, 999999),
                now()
            );
            self.referrer = Some(show_url.clone());
            let data = self
                .curl_text(
                    &show_url,
                    None,
                    Some(XLOGIN_REFERRER),
                    None,
                    "application/json",
                )
                .await;

            if let Some(vsig) = capture(r#"="([0-9a-zA-Z*_-]{187})""#, &data) {
                let height = capture(r#"Number\("(\d+)"\)"#, &data).unwrap_or_default();
                let websig = capture(r"websig=([0-9a-f]{128})", &data).unwrap_or_default();
                let collectname = capture(r"ans=.*?&([a-z]{6})=", &data).unwrap_or_default();
                let cdata = cdata_from_html(&data);
                let img_a = self.getvcpic2(uin, &vsig, sig, &sess, &sid, &websig, 1);
                let img_b = self.getvcpic2(uin, &vsig, sig, &sess, &sid, &websig, 0);
                let width = self.captcha(&img_a, &img_b).await;
                return json!({
                    "saveOK": 2,
                    "vc": vsig,
                    "sess": sess,
                    "collectname": collectname,
                    "websig": websig,
                    "ans": format!("{width},{height};"),
                    "cdata": cdata,
                    "sid": sid,
                });
            }
            if let Some(vsig) = capture(r#"vsig:"([0-9a-zA-Z*_-]{187})""#, &data) {
                let height = capture(r#"spt:"(\d+)""#, &data).unwrap_or_default();
                let websig = capture(r#"websig:"([0-9a-f]{128})""#, &data).unwrap_or_default();
                let collectname = capture(r#"collectdata:"([a-z]{6})""#, &data).unwrap_or_default();
                let cdata = cdata_from_html(&data);
                let img_a = self.getvcpic2(uin, &vsig, sig, &sess, &sid, &websig, 1);
                let img_b = self.getvcpic2(uin, &vsig, sig, &sess, &sid, &websig, 0);
                let width = self.captcha(&img_a, &img_b).await;
                return json!({
                    "saveOK": 2,
                    "vc": vsig,
                    "sess": sess,
                    "collectname": collectname,
                    "websig": websig,
                    "ans": format!("{width},{height};"),
                    "cdata": cdata,
                    "sid": sid,
                });
            }

            json!({"saveOK": -3, "msg": "获取验证码失败"})
        } else {
            let post = format!(
                "aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&color=&showtype=&fb=1&theme=&lang=2052&ua={}&grayscale=1&subsid=2&sess={sess}&sid={sid}&uid={uin}&cap_cd={sig}&rnd={}&TCapIframeLoadTime=99&prehandleLoadTime=48&createIframeStart={}758&rand=0.3944965{}",
                encoded_ua(),
                rand_int(100000, 999999),
                now(),
                now()
            );
            let referrer = format!(
                "https://ssl.captcha.qq.com/cap_union_new_show?aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&color=&showtype=&fb=1&theme=&lang=2052&ua={}&grayscale=1&subsid=2&sess={sess}&fwidth=0&sid={sid}&forcestyle=0&wxLang=&tcScale=1&uid={uin}&cap_cd={sig}&rnd={}&TCapIframeLoadTime=48&prehandleLoadTime=46&createIframeStart={}353",
                encoded_ua(),
                rand_int(100000, 999999),
                now()
            );
            self.referrer = Some(referrer.clone());
            let data = self
                .curl_text(
                    "https://ssl.captcha.qq.com/cap_union_new_getsig",
                    Some(&post),
                    Some(&referrer),
                    None,
                    "application/json",
                )
                .await;
            let arr = json_decode(&data);
            let chlg = arr.get("chlg").cloned().unwrap_or(json!({}));
            let cdata = getcdata(
                chlg.get("ans").and_then(Value::as_str).unwrap_or_default(),
                chlg.get("M").and_then(Value::as_i64).unwrap_or(0),
                chlg.get("randstr")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            let vsig = str_field(&arr, "vsig");

            if truthy(arr.get("initx")) && truthy(arr.get("inity")) {
                let height = value_to_string(arr.get("inity"));
                let img_a = self.getvcpic2(uin, &vsig, sig, sess, sid, websig, 1);
                let img_b = self.getvcpic2(uin, &vsig, sig, sess, sid, websig, 0);
                let width = self.captcha(&img_a, &img_b).await;
                json!({
                    "saveOK": 2,
                    "vc": vsig,
                    "sess": sess,
                    "ans": format!("{width},{height};"),
                    "cdata": cdata,
                })
            } else if !vsig.is_empty() {
                let image = self.getvcpic(uin, &vsig, sig, sess, sid).await;
                json!({
                    "saveOK": 0,
                    "vc": vsig,
                    "sess": sess,
                    "cdata": cdata,
                    "image": BASE64.encode(image),
                })
            } else {
                json!({"saveOK": -3, "msg": "获取验证码失败"})
            }
        }
    }

    pub async fn checkvc(&self, uin: &str, tokenid: &str) -> Value {
        if php_empty(uin) {
            return json!({"saveOK": -1, "msg": "请先输入QQ号码"});
        }
        let _tokenid = if php_empty(tokenid) {
            rand_int(2067831491, 5632894513).to_string()
        } else {
            tokenid.to_string()
        };
        if !valid_qq(uin, 14) {
            return json!({"saveOK": -2, "msg": "QQ号码不正确"});
        }

        let url = "https://xui.ptlogin2.qq.com/cgi-bin/xlogin?proxy_url=https%3A//qzs.qq.com/qzone/v6/portal/proxy.html&daid=5&&hide_title_bar=1&low_login=0&qlogin_auto_login=1&no_verifyimg=1&link_target=blank&appid=549000912&style=22&target=self&s_url=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&pt_no_auth=0";
        let data = self
            .curl_with_headers_text(url, None, None, None, "application/json")
            .await;
        let mut cookie = cookie_pairs_from_header_text(&data, false).join("; ");
        let pt_login_sig = cookie_value(&format!("{cookie};"), "pt_login_sig");
        let js_ver = capture(r"ver/(\d+)", &data).unwrap_or_default();
        let url2 = format!(
            "https://ssl.ptlogin2.qq.com/check?regmaster=&pt_tea=2&pt_vcode=1&uin={uin}&appid=549000912&js_ver={js_ver}&js_type=1&login_sig={pt_login_sig}&u1=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&r=0.{}722706&pt_uistyle=25",
            now()
        );
        let data = self
            .curl_with_headers_text(&url2, None, Some(url), Some(&cookie), "application/json")
            .await;
        let Some(inner) = capture(r"ptui_checkVC\('(.*?)'\)", &data) else {
            return json!({"saveOK": -3, "msg": format!("获取验证码失败{data}")});
        };
        for val in cookie_pairs_from_header_text(&data, false) {
            if !cookie.is_empty() {
                cookie.push_str("; ");
            }
            cookie.push_str(&val);
        }
        if !cookie.is_empty() {
            cookie.push_str("; ");
        }

        let r = split_callback(&inner);
        if r.first().map(String::as_str).unwrap_or_default() == "0" {
            json!({
                "saveOK": 0,
                "uin": uin,
                "vcode": r.get(1).cloned().unwrap_or_default(),
                "pt_verifysession": r.get(3).cloned().unwrap_or_default(),
                "cookie": cookie,
            })
        } else {
            json!({
                "saveOK": 1,
                "uin": uin,
                "sig": r.get(1).cloned().unwrap_or_default(),
                "cookie": cookie,
            })
        }
    }

    pub async fn getqrpic(&self) -> Value {
        let url = format!(
            "https://ssl.ptlogin2.qq.com/ptqrshow?appid=549000912&e=2&l=M&s=4&d=72&v=4&t=0.5409099{}&daid=5",
            now()
        );
        let arr = self.get_curl_split(&url).await;
        let qrsig = capture(r"qrsig=(.*?);", &arr.header_text).unwrap_or_default();
        if !qrsig.is_empty() {
            json!({"saveOK": 0, "qrsig": qrsig, "data": BASE64.encode(arr.body)})
        } else {
            json!({"saveOK": 1, "msg": "二维码获取失败"})
        }
    }

    pub async fn qrlogin(&self, qrsig: &str) -> Value {
        if php_empty(qrsig) {
            return json!({"saveOK": -1, "msg": "qrsig不能为空"});
        }
        let url = format!(
            "https://ssl.ptlogin2.qq.com/ptqrlogin?u1=https%3A%2F%2Fqzs.qq.com%2Fqzone%2Fv5%2Floginsucc.html%3Fpara%3Dizone&ptqrtoken={}&ptredirect=0&h=1&t=1&g=1&from_ui=1&ptlang=2052&action=0-0-{}0000&js_ver=10194&js_type=1&login_sig=&pt_uistyle=40&aid=549000912&daid=5&",
            getqrtoken(qrsig),
            now()
        );
        let ret = self
            .curl_with_headers_text(
                &url,
                None,
                Some(&url),
                Some(&format!("qrsig={qrsig}; ")),
                "application/json",
            )
            .await;
        self.parse_qr_login_response(&ret, false).await
    }

    pub async fn getqrpic3rd(&self, daid: &str, appid: &str) -> Value {
        if php_empty(daid) || php_empty(appid) {
            return json!({"saveOK": -1, "msg": "daid和appid不能为空"});
        }
        let url = format!(
            "https://ssl.ptlogin2.qq.com/ptqrshow?appid={appid}&e=2&l=M&s=4&d=72&v=4&t=0.5409099{}&daid={daid}",
            now()
        );
        let arr = self.get_curl_split(&url).await;
        let qrsig = capture(r"qrsig=(.*?);", &arr.header_text).unwrap_or_default();
        if !qrsig.is_empty() {
            json!({"saveOK": 0, "qrsig": qrsig, "data": BASE64.encode(arr.body)})
        } else {
            json!({"saveOK": 1, "msg": "二维码获取失败"})
        }
    }

    pub async fn qrlogin3rd(&self, daid: &str, appid: &str, qrsig: &str) -> Value {
        if php_empty(daid) || php_empty(appid) {
            return json!({"saveOK": -1, "msg": "daid和appid不能为空"});
        }
        if php_empty(qrsig) {
            return json!({"saveOK": -1, "msg": "qrsig不能为空"});
        }
        let s_url = match daid {
            "73" => "https://qun.qq.com/",
            "1" => "https://id.qq.com/index.html",
            _ => "https://qzs.qq.com/qzone/v5/loginsucc.html",
        };
        let url = format!(
            "https://ssl.ptlogin2.qq.com/ptqrlogin?u1={}&ptqrtoken={}&ptredirect=0&h=1&t=1&g=1&from_ui=1&ptlang=2052&action=0-0-{}0000&js_ver=10194&js_type=1&login_sig=&pt_uistyle=40&aid={appid}&daid={daid}&",
            php_urlencode(s_url),
            getqrtoken(qrsig),
            now()
        );
        let ret = self
            .curl_with_headers_text(
                &url,
                None,
                Some(&url),
                Some(&format!("qrsig={qrsig}; ")),
                "application/json",
            )
            .await;
        self.parse_qr_login_3rd_response(&ret).await
    }

    pub async fn idpic(&self) -> Value {
        self.getqrpic3rd("1", "1006102").await
    }

    pub async fn idlogin(&self, qrsig: &str) -> Value {
        self.qrlogin3rd("1", "1006102", qrsig).await
    }

    pub async fn list(&self, _uin: &str, _skey: &str, _p_skey: &str) -> Value {
        missing_php_method("list")
    }

    pub async fn danxiang(&self, _uin: &str, _qq: &str, _skey: &str) -> Value {
        missing_php_method("danxiang")
    }

    pub async fn del(&self, _uin: &str, _qq: &str, _skey: &str, _p_skey: &str) -> Value {
        missing_php_method("del")
    }

    pub async fn authf(
        &self,
        _uin: &str,
        _skey: &str,
        _p_skey: &str,
        _ptcz: &str,
        _rk: &str,
    ) -> Value {
        missing_php_method("authf")
    }

    #[allow(non_snake_case)]
    pub async fn InfoNull(&self, _uin: &str, _skey: &str, _p_skey: &str, _rk: &str) -> Value {
        missing_php_method("InfoNull")
    }

    #[allow(non_snake_case)]
    pub async fn NickNull(&self, _uin: &str, _skey: &str, _p_skey: &str, _rk: &str) -> Value {
        missing_php_method("NickNull")
    }

    async fn parse_qr_login_response(&self, ret: &str, _findpwd: bool) -> Value {
        let Some(inner) = capture(r"ptuiCB\('(.*?)'\)", ret) else {
            return json!({"saveOK": 6, "msg": ret});
        };
        let r = split_callback(&inner);
        match r.first().map(String::as_str).unwrap_or_default() {
            "0" => {
                let loginurl = r.get(2).cloned().unwrap_or_default();
                let uin = capture(r"uin=(\d+)&", ret).unwrap_or_default();
                let skey = capture(r"skey=@(.{9});", ret).unwrap_or_default();
                let superkey = capture(r"superkey=(.*?);", ret).unwrap_or_default();
                let data = self
                    .curl_with_headers_text(&loginurl, None, None, None, "application/json")
                    .await;
                let pskey = capture(r"p_skey=(.*?);", &data).unwrap_or_default();
                if !pskey.is_empty() {
                    json!({
                        "saveOK": 0,
                        "uin": uin,
                        "skey": format!("@{skey}"),
                        "pskey": pskey,
                        "superkey": superkey,
                        "nick": r.get(5).cloned().unwrap_or_default(),
                    })
                } else {
                    json!({"saveOK": 6, "msg": format!("登录成功，获取相关信息失败！{loginurl}")})
                }
            }
            "65" => json!({"saveOK": 1, "msg": "二维码已失效。"}),
            "66" => json!({"saveOK": 2, "msg": "二维码未失效。"}),
            "67" => json!({"saveOK": 3, "msg": "正在验证二维码。"}),
            _ => json!({"saveOK": 6, "msg": r.get(4).cloned().unwrap_or_default()}),
        }
    }

    async fn parse_qr_login_3rd_response(&self, ret: &str) -> Value {
        let Some(inner) = capture(r"ptuiCB\('(.*?)'\)", ret) else {
            return json!({"saveOK": 6, "msg": ret});
        };
        let r = split_callback(&inner);
        match r.first().map(String::as_str).unwrap_or_default() {
            "0" => {
                let loginurl = r.get(2).cloned().unwrap_or_default();
                let uin = capture(r"uin=(\d+)&", ret).unwrap_or_default();
                let data = self
                    .curl_with_headers_text(&loginurl, None, None, None, "application/json")
                    .await;
                let cookie_pairs = cookie_pairs_from_header_text(&data, true);
                let cookie = cookie_pairs.join("; ");
                if !cookie.is_empty() {
                    json!({
                        "saveOK": 0,
                        "uin": uin,
                        "cookie": cookie,
                        "nickname": r.get(5).cloned().unwrap_or_default(),
                    })
                } else {
                    json!({"saveOK": 6, "msg": format!("登录成功，获取相关信息失败！{loginurl}")})
                }
            }
            "65" => json!({"saveOK": 1, "msg": "二维码已失效。"}),
            "66" => json!({"saveOK": 2, "msg": "二维码未失效。"}),
            "67" => json!({"saveOK": 3, "msg": "正在验证二维码。"}),
            _ => json!({"saveOK": 6, "msg": r.get(4).cloned().unwrap_or_default()}),
        }
    }

    async fn getvcpic(&self, uin: &str, sig: &str, cap_cd: &str, sess: &str, sid: &str) -> Vec<u8> {
        let url = format!(
            "https://ssl.captcha.qq.com/cap_union_new_getcapbysig?aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&uid={uin}&color=&showtype=&fb=1&lang=2052&ua={}&grayscale=1&cap_cd={cap_cd}&rnd={}&rand=0.02398118{}&sess={sess}&sid={sid}&vsig={sig}&ischartype=1",
            encoded_ua(),
            rand_int(100000, 999999),
            now()
        );
        self.curl_bytes(&url, None, None, None, "application/json")
            .await
    }

    fn getvcpic2(
        &self,
        uin: &str,
        sig: &str,
        cap_cd: &str,
        sess: &str,
        sid: &str,
        websig: &str,
        img_index: i64,
    ) -> String {
        format!(
            "https://ssl.captcha.qq.com/cap_union_new_getcapbysig?aid=549000912&captype=&protocol=https&clientype=2&disturblevel=&apptype=2&noheader=0&color=&showtype=&fb=1&theme=&lang=2052&ua={}&grayscale=1&subsid=3&sess={sess}&fwidth=0&sid={sid}&forcestyle=0&tcScale=1&uid={uin}&cap_cd={cap_cd}&rnd={}&rand={}&websig={websig}&vsig={sig}&img_index={img_index}",
            encoded_ua(),
            rand_int(100000, 999999),
            rand_int(10000000, 99999999)
        )
    }

    async fn captcha(&self, img_a_url: &str, img_b_url: &str) -> i64 {
        let referer = self.referrer.as_deref();
        let img_a = self
            .curl_bytes(img_a_url, None, referer, None, "application/json")
            .await;
        let img_b = self
            .curl_bytes(img_b_url, None, referer, None, "application/json")
            .await;
        let Ok(img_a) = image::load_from_memory(&img_a) else {
            return 0;
        };
        let Ok(img_b) = image::load_from_memory(&img_b) else {
            return 0;
        };
        let width = img_a.width().min(img_b.width());
        let height = img_a.height().min(img_b.height());
        let mut total = 0_i64;
        let mut sum_x = 0_i64;

        if width <= 40 || height <= 40 {
            return 0;
        }

        for y in 20..(height - 20) {
            for x in 20..(width - 20) {
                let a = img_a.get_pixel(x, y).0;
                let b = img_b.get_pixel(x, y).0;
                let rgb_a = ((a[0] as i64) << 16) + ((a[1] as i64) << 8) + a[2] as i64;
                let rgb_b = ((b[0] as i64) << 16) + ((b[1] as i64) << 8) + b[2] as i64;
                if (rgb_a - rgb_b).abs() > 1_800_000 {
                    total += 1;
                    sum_x += x as i64;
                }
            }
        }

        if total == 0 {
            0
        } else {
            ((sum_x as f64 / total as f64).round() as i64) - 55
        }
    }

    async fn getcollect(&mut self, width: i64) -> Value {
        let tokenid = rand_int(2067831491, 5632894513).to_string();
        let slide_value = self.generate_slide_value(width);
        self.tdc_data(&tokenid, &slide_value).await
    }

    async fn tdc_data(&self, tokenid: &str, slide_value: &str) -> Value {
        let post =
            serde_urlencoded::to_string(&[("tokenid", tokenid), ("slideValue", slide_value)])
                .unwrap_or_default();
        let data = self
            .curl_text(
                "http://collect.qqzzz.net/",
                Some(&post),
                None,
                None,
                "application/json",
            )
            .await;
        json_decode(&data)
    }

    fn generate_slide_value(&mut self, width: i64) -> String {
        let mut sx = rand_int(700, 730);
        let sy = rand_int(295, 300);
        self.trace_x = sx;
        self.trace_y = sy;
        let ex = sx + ((width - 55) / 2);
        let mut stime = rand_int(100, 300);
        let mut res = format!("[{sx},{sy},{stime}],");
        let randy = [0, 0, 0, 0, 0, 0, 1, 1, 1, 2, 3, -1, -1, -1, -2];
        while sx < ex {
            let x = rand_int(3, 9);
            sx += x;
            let y = randy[rand_int(0, (randy.len() - 1) as i64) as usize];
            let time = rand_int(9, 18);
            stime += time;
            res.push_str(&format!("[{x},{y},{time}],"));
        }
        res.push_str(&format!("[0,0,{}]", rand_int(10, 25)));
        res
    }

    #[allow(dead_code)]
    fn generate_mousemove(&mut self, width: i64) -> String {
        let mut sx = rand_int(720, 810);
        let mut sy = rand_int(270, 290);
        let mut stime = rand_int(800, 1000);
        let mut res = format!("[{sx},{sy},{stime}],");
        while sx > self.trace_x || sy < self.trace_y {
            let x = if sx > self.trace_x {
                rand_int(-5, -1)
            } else {
                0
            };
            let y = if sy < self.trace_y { rand_int(1, 2) } else { 0 };
            sx += x;
            sy += y;
            let time = rand_int(9, 16);
            stime += time;
            res.push_str(&format!("[{x},{y},{time}],"));
        }
        let ex = self.trace_x + ((width - 55) / 2);
        while sx < ex {
            let x = rand_int(1, 6);
            sx += x;
            let time = rand_int(9, 16);
            stime += time;
            res.push_str(&format!("[{x},0,{time}],"));
        }
        res.pop();
        self.trace_time = (stime + 999) / 1000;
        self.trace_x = sx;
        self.trace_y = sy;
        res
    }

    async fn curl_text(
        &self,
        url: &str,
        post: Option<&str>,
        referer: Option<&str>,
        cookie: Option<&str>,
        accept: &str,
    ) -> String {
        String::from_utf8_lossy(&self.curl_bytes(url, post, referer, cookie, accept).await)
            .into_owned()
    }

    async fn curl_with_headers_text(
        &self,
        url: &str,
        post: Option<&str>,
        referer: Option<&str>,
        cookie: Option<&str>,
        accept: &str,
    ) -> String {
        self.curl(url, post, referer, cookie, accept)
            .await
            .map(|r| r.combined_text())
            .unwrap_or_default()
    }

    async fn curl_bytes(
        &self,
        url: &str,
        post: Option<&str>,
        referer: Option<&str>,
        cookie: Option<&str>,
        accept: &str,
    ) -> Vec<u8> {
        self.curl(url, post, referer, cookie, accept)
            .await
            .map(|r| r.body)
            .unwrap_or_default()
    }

    async fn get_curl_split(&self, url: &str) -> CurlResponse {
        self.curl(url, None, None, None, "*/*")
            .await
            .unwrap_or_else(|| CurlResponse {
                header_text: String::new(),
                body: Vec::new(),
            })
    }

    async fn curl(
        &self,
        url: &str,
        post: Option<&str>,
        referer: Option<&str>,
        cookie: Option<&str>,
        accept: &str,
    ) -> Option<CurlResponse> {
        let mut req = if let Some(post) = post {
            self.client
                .request(Method::POST, url)
                .body(post.to_string())
                .header("Content-Type", "application/x-www-form-urlencoded")
        } else {
            self.client.request(Method::GET, url)
        };

        req = req
            .header(ACCEPT, accept)
            .header(ACCEPT_ENCODING, "gzip,deflate,sdch")
            .header(ACCEPT_LANGUAGE, "zh-CN,zh;q=0.8")
            .header(CONNECTION, "keep-alive")
            .header(USER_AGENT, QQ_UA);
        if let Some(referer) = referer.filter(|s| !s.is_empty()) {
            req = req.header(REFERER, referer);
        }
        if let Some(cookie) = cookie.filter(|s| !s.is_empty()) {
            req = req.header(COOKIE, cookie);
        }

        let resp = req.send().await.ok()?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let body = resp.bytes().await.ok()?.to_vec();
        let header_text = header_text(status.as_u16(), &headers);
        Some(CurlResponse { header_text, body })
    }
}

fn php_empty(s: &str) -> bool {
    s.is_empty() || s == "0"
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn rand_int(min: i64, max: i64) -> i64 {
    rand::thread_rng().gen_range(min..=max)
}

fn encoded_ua() -> String {
    php_urlencode(&BASE64.encode(QQ_UA))
}

fn php_urlencode(input: &str) -> String {
    form_urlencoded::byte_serialize(input.as_bytes()).collect()
}

fn valid_qq(uin: &str, max_digits: usize) -> bool {
    let bytes = uin.as_bytes();
    bytes.len() >= 5
        && bytes.len() <= max_digits
        && matches!(bytes.first(), Some(b'1'..=b'9'))
        && bytes.iter().all(u8::is_ascii_digit)
}

fn json_decode(data: &str) -> Value {
    serde_json::from_str(data.trim_start_matches('\u{feff}')).unwrap_or_else(|_| json!({}))
}

fn str_field(value: &Value, key: &str) -> String {
    value_to_string(value.get(key))
}

fn value_to_string(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => {
            if *b {
                "1".to_string()
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

fn truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().map(|v| v != 0).unwrap_or(true),
        Some(Value::String(s)) => !s.is_empty() && s != "0",
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        _ => false,
    }
}

fn capture(pattern: &str, text: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .captures(text)?
        .get(1)
        .map(|m| m.as_str().to_string())
}

fn split_callback(inner: &str) -> Vec<String> {
    inner
        .replace("', '", "','")
        .split("','")
        .map(ToOwned::to_owned)
        .collect()
}

fn cookie_value(cookie: &str, name: &str) -> String {
    capture(&format!(r"{}=(.*?);", regex::escape(name)), cookie).unwrap_or_default()
}

fn cookie_pairs_from_header_text(data: &str, skip_empty_value: bool) -> Vec<String> {
    let Ok(re) = Regex::new(r"(?i)Set-Cookie: (.*?);") else {
        return Vec::new();
    };
    re.captures_iter(data)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .filter(|pair| !skip_empty_value || !pair.ends_with('='))
        .collect()
}

fn header_text(status: u16, headers: &HeaderMap) -> String {
    let mut text = format!("HTTP/1.1 {status}\r\n");
    for (name, value) in headers.iter() {
        if let Ok(value) = value.to_str() {
            text.push_str(name.as_str());
            text.push_str(": ");
            text.push_str(value);
            text.push_str("\r\n");
        }
    }
    for value in headers.get_all(SET_COOKIE).iter() {
        if let Ok(value) = value.to_str() {
            if !text.contains(value) {
                text.push_str("set-cookie: ");
                text.push_str(value);
                text.push_str("\r\n");
            }
        }
    }
    text.push_str("\r\n");
    text
}

fn getqrtoken(qrsig: &str) -> i64 {
    let mut hash = 0_i64;
    for byte in qrsig.bytes() {
        hash += ((hash << 5) & 2_147_483_647) + byte as i64 & 2_147_483_647;
        hash &= 2_147_483_647;
    }
    hash & 2_147_483_647
}

fn cdata_from_html(data: &str) -> i64 {
    let Ok(re) = Regex::new(
        r#"\{&quot;randstr&quot;:&quot;(.{4})&quot;,&quot;M&quot;:&quot;(\d+)&quot;,&quot;ans&quot;:&quot;([0-9a-f]{32})&quot;\}"#,
    ) else {
        return 0;
    };
    let Some(caps) = re.captures(data) else {
        return 0;
    };
    let randstr = caps.get(1).map(|m| m.as_str()).unwrap_or_default();
    let m = caps
        .get(2)
        .and_then(|m| m.as_str().parse::<i64>().ok())
        .unwrap_or(0);
    let ans = caps.get(3).map(|m| m.as_str()).unwrap_or_default();
    getcdata(ans, m, randstr)
}

fn getcdata(ans: &str, m: i64, randstr: &str) -> i64 {
    for r in 0..m.min(1000) {
        let mut hasher = Md5::new();
        hasher.update(format!("{randstr}{r}").as_bytes());
        let digest = format!("{:x}", hasher.finalize());
        if ans == digest {
            return r;
        }
    }
    0
}

fn missing_php_method(method: &str) -> Value {
    json!({
        "saveOK": -1,
        "msg": format!("{method} not found in extend/qqskey/login.class.php"),
    })
}
