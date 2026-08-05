# rustapi — oldapi 的 Rust 1:1 重写

完整架构说明见 [`ARCHITECTURE.md`](./ARCHITECTURE.md)，接口对照见 [`PARITY.md`](./PARITY.md)。

## 快速开始

```bash
cd rustapi
cp .env.example .env
# 编辑 DATABASE_URL 等

cargo run
# 默认监听 0.0.0.0:8080
```

## URL（与 oldapi 兼容）

| 路径 | 说明 |
|------|------|
| `/api/v2/{Action}` | 公网 API（如 `/api/v2/MD5?text=hi`） |
| `/api/page/{action}` | 分页/点赞 |
| `/api/update/{Action}` | 数据更新任务 |
| `/api/skey/index?do=` | QQ SKEY 工具 |
| `/login/index/*` | 登录 |
| `/register/index/*` | 注册 |
| `/index/index/*` | 控制台 |

响应支持 `type=json|xml`（默认 JSON）。

## 技术栈

Axum · SQLx(MySQL) · tower-sessions · reqwest · moka 缓存
