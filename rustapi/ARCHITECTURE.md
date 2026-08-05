# rustapi — oldapi 1:1 Rust 重写架构

目标：在 **不依赖 ThinkPHP / PHP** 的前提下，用 Rust **逐接口、逐行为**还原 `oldapi` 的全部功能与 URL 约定。

参照源：`/workspace/oldapi`（ThinkPHP 5 多模块应用）。

---

## 1. 边界与兼容承诺

| 维度 | 约定 |
|------|------|
| URL | 保持 PATHINFO：`/{module}/{controller}/{action}` |
| 查询参数 | 同名：`type=json\|xml`、`page`、`appid`/`appkey`（鉴权逻辑仍按原注释状态） |
| 响应 | `INT(type,data)` → JSON（默认）或 XML；字段名尽量不变 |
| 会话 | Cookie `UID`/`UserName` + Session `UID`/`Email`/`UserName` 对齐校验 |
| 数据库 | 同一 MySQL schema（`user/api/request/log/hot/love/...`） |
| 静态资源 | `public/hot.json`、`address.json`、`img.data`、`msyh.ttf`、`xhxh.jpg`、`uploads/` |
| 不在范围 | `oldapi/tool/*` 独立脚本、`AAAAAA/*` 半成品控制台（非 Think 主路径） |

---

## 2. 分层架构

```
                    ┌─────────────────────────────┐
   HTTP             │  axum Router (PATHINFO)     │
                    └──────────────┬──────────────┘
                                   │
          ┌────────────────────────┼────────────────────────┐
          ▼                        ▼                        ▼
   handlers/api/*           handlers/web/*            static files
   (v2/page/update/skey)    (login/register/index)
          │                        │
          └────────────┬───────────┘
                       ▼
              ┌─────────────────┐
              │  AppState       │  Pool / HttpClient / Config / Cache
              └────────┬────────┘
                       ▼
        ┌──────────────┼──────────────┐
        ▼              ▼              ▼
   services/*     repositories/*   respond / auth / util
   (外部 API、     (SQL 访问)       (JSON/XML、日志、UA/IP)
    邮件、验证码、
    二维码、音乐…)
```

### 2.1 目录结构

```
rustapi/
├── Cargo.toml
├── ARCHITECTURE.md          # 本文档
├── PARITY.md                # 接口 1:1 清单与状态
├── .env.example
├── config/
│   └── default.toml
├── templates/               # Askama：登录/注册/控制台
│   ├── login/
│   ├── register/
│   └── index/
├── public/                  # 静态资源（从 oldapi/public 拷贝）
├── src/
│   ├── main.rs              # 启动、绑定、路由挂载
│   ├── config.rs
│   ├── state.rs             # AppState
│   ├── error.rs
│   ├── respond.rs           # json/xml/INT、版权字段
│   ├── auth.rs              # session+cookie 登录态
│   ├── util/                # ip/ua/md5/http helpers
│   ├── db/                  # sqlx pool + 公共查询
│   ├── cache.rs             # 文件/Redis 缓存（对应 Cache::store）
│   ├── services/            # 领域服务（邮件、极验、拼音、QQSKEY…）
│   ├── handlers/
│   │   ├── mod.rs           # 总路由：按 module/controller/action 分发
│   │   ├── api/
│   │   │   ├── mod.rs
│   │   │   ├── index.rs
│   │   │   ├── v2.rs        # ~74 actions（可按文件拆分 v2/*.rs）
│   │   │   ├── page.rs
│   │   │   ├── update.rs
│   │   │   └── skey.rs
│   │   └── web/
│   │       ├── login.rs
│   │       ├── register.rs
│   │       └── index.rs     # 控制台
│   └── models/              # 行结构体（User、Api、Log…）
└── tests/
    └── parity_smoke.rs
```

---

## 3. 技术选型

| 能力 | Crate | 对应旧实现 |
|------|-------|-----------|
| HTTP | `axum` + `tokio` | ThinkPHP 多模块入口 |
| DB | `sqlx` (MySQL) | `think\Db` |
| 模板 | `askama` | Think 模板 `.html` |
| Session | `tower-sessions` (cookie store) | `think\Session` |
| HTTP 客户端 | `reqwest` | curl / Guzzle |
| 序列化 | `serde` / `serde_json` / `quick-xml` | `json()` / `xml()` |
| 配置 | `config` + `dotenvy` | `application/*.php` |
| 缓存 | `moka` 或 Redis(`redis`) + 文件回退 | `Cache::store('redis')` |
| 上传 | `axum::extract::Multipart` | `request()->file()->move` |
| 图像 | `image` + `ab_glyph` | GD + `msyh.ttf` |
| QR | `qrcode` + `image` | phpqrcode |
| 密码 | `md5`（兼容旧库存） | `md5($Password)` |
| 邮件 | `lettre` | PHPMailer SMTP |
| 日志 | `tracing` | runtime 日志 |

> 兼容优先：用户密码仍按 **MD5 十六进制** 与旧库比对，不做算法升级（避免破坏存量账号）。

---

## 4. 路由模型（1:1 PATHINFO）

统一入口解析：

```
/{module}/{controller}/{action}?k=v
```

| module | controller | action 数 | 说明 |
|--------|------------|-----------|------|
| `api` | `v2` | 74 + index | 公网工具 API，每次写 `log` |
| `api` | `page` | 9 + index | 分页/点赞/统计 |
| `api` | `update` | 10 + index | 爬虫/灌数 |
| `api` | `skey` | index(`do=…`) | QQ SKEY 工具箱 |
| `api` | `index` | index | 错误桩 |
| `login` | `index` | index/GtCode/callback | 登录 + 极验 |
| `register` | `index` | index/GtCode/Email/callback | 注册 + 邮件验证码 |
| `index` | `index` | index/page/setting/appkey/LoginOut/ip/log | 控制台 |

别名：`GET /` → 需登录则进控制台，否则跳转 `/login/index/index`（与旧行为一致）。

动态分发伪代码：

```rust
// handlers/mod.rs
Router::new()
  .route("/", get(web::index::home))
  .route("/{*path}", any(dispatch_pathinfo))
  .nest_service("/public", ServeDir::new("public"))
```

`dispatch_pathinfo` 解析三段后匹配 `handlers::{module}::{controller}::{action}`，action 名大小写按旧 PHP 方法名保留（`UserInfo`、`Music_163`）。

---

## 5. 横切能力

### 5.1 响应 `respond::int(type, value)`
- `type=xml` → XML root=`root`
- 默认 JSON，`Content-Type: application/json; charset=utf-8`
- 公共字段：`code` / `msg` / `Copyright{name,url,time}` / `data`

### 5.2 API 访问日志
每个 `api/v2/*`、`api/page/*` 入口插入：

```sql
INSERT INTO log(name,time,UserAgent,request,ip) VALUES (?,?,?,?,?)
```

（与旧 `Db::table('log')->data(...)->insert()` 一致）

### 5.3 鉴权
- 控制台：Cookie↔Session 六字段一致才放行，否则 redirect login
- API APPID/APPKEY：旧代码整段注释 → Rust 同样默认关闭，保留可编译的钩子函数 `check_appkey()` 以便开启

### 5.4 缓存键
与旧键一致，例如：
`lishishangdejintian{Y-m-d}`、`Music_List_ID_163{id}`、`city_{kw}`、`WeatherInfo_{kw}`、`IpSadd_{ip}`、`qqxj_{qq}`、`telxj_{tel}`、`idcard_s{id}`

### 5.5 whereTime
实现 today / yesterday / week / last week / month / last month / between，字段存 Unix 时间戳。

---

## 6. 功能域划分（1:1 清单摘要）

### A. `api/v2` — 公网 API
**信息/代理：** UserInfo, Url, qlogo, Gravatar, Bing_img, image, IP, IP_IMG, Email, ICP, DM_IMG, C_IMG, address, HttpCode, Go, proxy, upload, Baidu_Upload, Sogou_Upload  

**内容随机库：** netease, love, Name, autograph, shuoshuo, word, dog, HeadImg, today, du_word, xiaohua, chengyu, colors  

**音乐：** Music_163, Music, Music_hot, Music_List_163, Kugou, qq, kuwo, xiami, migu, kg, MusicUrl, audio  

**工具：** MD5, QrCode, QrReader, Mobile, whois, pinyin, fanyi, ping, robot, weather, WeatherInfo, laji, check, check_domain, Url_Sec, RandPass, district, IpSadd, QQChat, QQInfo, idcard, telxj, qqxj  

**短视频解析：** douyin, kuaishou, weishi, pipixia, miaopai  

**群头像等：** qunlogo  

### B. `api/page` — 分页与点赞
netease / autograph / love / name / shuoshuo / word / log / likes / headimg  
（`page=love&id=` 点赞流；`log?page=day` 统计）

### C. `api/update` — 数据更新任务
Music_hot, Word, Wordtime, netease, headimg, words, qianming, wangming, net, dog  

### D. `api/skey` — QQ 登录工具
`do=`：checkvc, dovc, getvc, qqlogin, getqrpic, qrlogin, list, danxiang, del, authf, getqrpic3rd, qrlogin3rd, idpic, idlogin, InfoNull, NickNull  

### E. Web 控制台
- login：页面 + GtCode + callback  
- register：页面 + GtCode + Email + callback  
- index：仪表盘统计、API 文档页、改密/换绑、APPKEY 重置、Logout、IP 查询  

---

## 7. 数据模型（表）

| 表 | 用途 |
|----|------|
| `user` | 账号 / APPKEY |
| `api` | 控制台导航与文档元数据 |
| `request` | 每用户调用明细（鉴权开启时写） |
| `log` | 全站 API 访问日志 |
| `likes` | 点赞（cid+ip+keyword） |
| `hot` / `love` / `name` / `autograph` / `shuoshuo` / `word` / `dog` | 内容库 |
| `freeip` | 代理池 |
| 动态表（如 `女生头像`） | `HeadImg`/`headimg` 按 `kw` |

---

## 8. 服务层映射

| 旧 PHP | Rust `services/` |
|--------|------------------|
| `email()` + PHPMailer | `services::mail` (lettre) |
| `GeetestLib` | `services::geetest`（含 failback） |
| `QrCode()` / `QrReader()` | `services::qrcode` |
| `pinyin()` / `hanzi()` | `services::pinyin` |
| `music163` / `m_163` / `\Musics` | `services::music` |
| `PhoneLocation` | `services::mobile` |
| `GetWangYiYunInfo` | `services::netease` |
| `qq_login` (extend/qqskey) | `services::qqskey` |
| `get_bro` / `get_os_info` / `get_ip` | `util::{ua,ip}` |
| QueryList / Guzzle 抓取 | `reqwest` + `scraper` |

---

## 9. 实现阶段（按可验证切片）

1. **骨架**：配置、State、PATHINFO 路由、respond、DB pool、session  
2. **Web 三件套**：login / register / dashboard（模板 + 鉴权）  
3. **page + 简单 v2**：MD5/colors/RandPass/love/word…（DB 随机与分页）  
4. **v2 外部 HTTP 类**：IP/天气/翻译/短链/视频解析…  
5. **update 灌数任务**  
6. **skey / qqskey**  
7. **媒体类**：二维码、IP_IMG、upload、音频流  
8. **PARITY 对照测试** + 与 oldapi 同参抽样 diff  

每阶段保持 URL 与 JSON 字段稳定，便于对照 `oldapi` 回归。

---

## 10. 配置

环境变量（`.env`）：

```
APP_URL=
APP_HOST=0.0.0.0:8080
DATABASE_URL=mysql://api:api_0324@127.0.0.1:3306/api
SESSION_SECRET=change-me
REDIS_URL=redis://127.0.0.1:6379/   # 可选，缺省文件/内存缓存
SMTP_HOST=smtp.exmail.qq.com
SMTP_PORT=465
SMTP_USER=
SMTP_PASS=
GEETEST_ID=
GEETEST_KEY=
PUBLIC_DIR=./public
```

---

## 11. 明确的非目标 / 风险

- **不**再引入 PHP / ThinkPHP 运行时  
- 部分上游站点（短视频、备案查询等）可能已失效：Rust 侧保留相同请求路径与错误形状，行为随上游变化（与旧版相同风险）  
- `QrReader`、完整 `\Musics` 多站点若缺算法依赖，提供等价接口与可替换实现，并在 `PARITY.md` 标注  
- 密码 MD5 仅兼容历史，不作为新系统安全建议  

---

## 12. 验收标准

1. `PARITY.md` 中每个 action 状态为 `done`  
2. 抽样：同 URL + 同参数，Rust 与 oldapi 的 `code`/关键 `data` 字段一致（时间戳、随机内容除外）  
3. 登录→控制台→重置 APPKEY→登出全流程可用  
4. `type=xml` 与默认 JSON 双通道可用  
5. `cargo test` 与基础 smoke（MD5 / page/love / login）通过  
