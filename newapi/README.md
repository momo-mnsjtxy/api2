# newapi — 纯 PHP 重写版

从 `oldapi`（ThinkPHP 5）迁移而来的 **无 ThinkPHP** 实现。

## 结构

```
newapi/
  public/index.php   # Web 入口
  core/              # 轻量核心（路由/请求/DB/Session/Cache/视图）
  think/             # 兼容层（旧控制器 use think\Db 等仍可工作）
  app/               # 业务控制器与视图
  extend/            # 扩展库
  config/            # 配置
```

## 运行

```bash
cd newapi
composer install
php -S 0.0.0.0:8080 router.php
```

访问示例：

- `http://127.0.0.1:8080/api/v2/MD5?text=hello`
- `http://127.0.0.1:8080/login/index/index`

## 环境变量（可选）

| 变量 | 说明 |
|------|------|
| `DB_HOST` / `DB_DATABASE` / `DB_USERNAME` / `DB_PASSWORD` / `DB_PORT` | MySQL |
| `SMTP_HOST` / `SMTP_PORT` / `SMTP_USER` / `SMTP_PASS` | 发信 |
| `GEETEST_ID` / `GEETEST_KEY` | 极验验证码 |

## URL 规则

与旧版兼容的 PATHINFO：`/{module}/{controller}/{action}`  
例如：`/api/v2/UserInfo`、`/api/page/love`、`/index/index/setting`
