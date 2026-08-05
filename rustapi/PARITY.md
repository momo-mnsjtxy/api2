# 1:1 功能对照清单（相对 oldapi）

## 总览（检查结论）

| 维度 | 结论 |
|------|------|
| 控制器 action 覆盖 | **齐全**：oldapi 111 个 public action → rustapi 均有对应路由/handler |
| V2 公网 API | **路由齐全**（74/74）；深度缺口已大部分补齐 |
| Page / Update / Web | **路由齐全**；Update 上游失效时有兜底 |
| 行为 1:1 | **接近完成**：核心子系统已移植；UI/部分上游仍为 best-effort |

### 已补齐（相对上一轮审计）

1. **`api/skey` + `qq_login`**：移植 `login.class.php` 主流程（checkvc/dovc/getvc/qqlogin/getqrpic/qrlogin/3rd/idpic/idlogin）
2. **多站点音乐 `\Musics`**：netease/kugou/qq/kuwo/migu/kg（xiami best-effort）
3. **`QrReader`**：`quircs` 解码 URL/字节图
4. **`IP_IMG`**：`msyh.ttf` + `ab_glyph` 文字叠加
5. **邮件 SMTP**：配置 `SMTP_USER`/`SMTP_PASS` 时走 rustls SMTP；否则写 `.eml`
6. **极验 Geetest**：对接 `register.php` / `validate.php`（gt3）；宕机 failback
7. **`update/net` / Music_hot / dog / netease**：多上游 + 合成兜底
8. **拼音词典**：CJK 常用字全量字典（~20k）

### 仍非像素级 / 明确不在范围

- 控制台/登录 UI：功能可用，非原 Think 模板视觉 1:1
- Skey 的 `list/danxiang/del/authf/InfoNull/NickNull`：PHP 源无对应方法
- `oldapi/tool/*`、`AAAAAA/*` 不在范围
- 部分第三方上游（短链、ICP、视频解析等）随外部服务可用性波动

状态：`done` = 路由+主逻辑可用 · `partial` = 有接口但深度/上游不足 · `broken` = 调用即失败/空壳

---

## api/v2

| action | 状态 | 备注 |
|--------|------|------|
| index | done | |
| UserInfo | done | |
| Url | done | 依赖上游短链服务 |
| qlogo | done | |
| Gravatar | done | |
| Bing_img | done | |
| image | done | |
| IP | done | |
| IP_IMG | done | TTF 叠加位置/日期/IP/OS/浏览器/自定义文案 |
| Email | done | SMTP 或 `.eml` 兜底 |
| ICP | done | 依赖上游 |
| DM_IMG | done | |
| C_IMG | done | 依赖上游 |
| netease | done | DB 随机 |
| love | done | |
| Name | done | |
| autograph | done | |
| shuoshuo | done | |
| word | done | |
| Music_163 | done | 公开接口 best-effort |
| Music | done | 多站点搜索 |
| Music_hot | done | 读 `public/hot.json` |
| MD5 | done | |
| QrCode | done | PNG 编码 |
| QrReader | done | quircs 解码 |
| Mobile | done | 360 归属地 |
| whois | done | |
| pinyin | done | 全量常用字词典 |
| fanyi | done | |
| ping | done | |
| robot | done | |
| weather | done | |
| laji | done | |
| address | done | |
| check | done | |
| check_domain | done | |
| du_word | done | |
| Url_Sec | done | |
| Baidu_Upload | done | multipart |
| Sogou_Upload | done | multipart |
| upload | done | multipart |
| proxy | done | |
| HeadImg | done | |
| today | done | 缓存 |
| audio | done | |
| colors | done | |
| qunlogo | done | |
| chengyu | done | |
| xiaohua | done | |
| HttpCode | done | |
| RandPass | done | |
| Go | done | |
| Music_List_163 | done | playlist API best-effort |
| district | done | |
| WeatherInfo | done | |
| IpSadd | done | |
| Kugou | done | Musics |
| qq | done | Musics |
| kuwo | done | Musics |
| xiami | partial | 上游弱 / best-effort |
| migu | done | Musics |
| kg | done | Musics |
| MusicUrl | done | Musics |
| QQChat | done | |
| QQInfo | done | |
| douyin | done | 上游变更可能失效 |
| qqxj | done | |
| telxj | done | |
| idcard | done | |
| dog | done | |
| kuaishou | done | |
| weishi | done | |
| pipixia | done | |
| miaopai | done | |

## api/page

| action | 状态 |
|--------|------|
| index | done |
| netease | done |
| autograph | done |
| love | done |
| name | done |
| shuoshuo | done |
| word | done |
| log | done |
| likes | done |
| headimg | done |

## api/update

| action | 状态 | 备注 |
|--------|------|------|
| index | done | |
| Music_hot | done | gqink collect → 网易歌单兜底 |
| Word | done | |
| Wordtime | done | |
| netease | done | 多上游 + 音乐合成兜底 |
| headimg | done | |
| words | done | |
| qianming | done | |
| wangming | done | |
| net | done | 复用 netease 热评写入路径 |
| dog | done | 多源兜底 |

## api/skey

| action | 状态 | 备注 |
|--------|------|------|
| index `do=*`（主流） | done | checkvc/dovc/getvc/qqlogin/qr*/id* 已移植 |
| list/danxiang/del/authf/InfoNull/NickNull | partial | PHP `login.class.php` 无实现 |

## web

| path | 状态 | 备注 |
|------|------|------|
| login/index/{index,GtCode,callback} | done | Geetest 实对接；UI 简化 |
| register/index/{index,GtCode,Email,callback} | done | 邮件 SMTP/`.eml` |
| index/index/{index,page,setting,appkey,LoginOut,ip,log} | done | 控制台功能在，非原模板视觉 |

## 残留优先级（可选）

1. 还原 Askama/原 HTML 控制台模板像素级
2. 若找到 PHP 源再补 Skey list/danxiang 等
3. xiami 等死亡上游可继续换镜像
