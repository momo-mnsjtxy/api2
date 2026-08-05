# 1:1 功能对照清单（相对 oldapi）

## 总览（检查结论）

| 维度 | 结论 |
|------|------|
| 控制器 action 覆盖 | **齐全**：oldapi 111 个 public action → rustapi 均有对应路由/handler |
| V2 公网 API | **路由齐全**（74/74），其中约 **12+** 个深度未对齐（见 partial） |
| Page / Update / Web | **路由齐全**；Update.net / 多站点音乐 / Skey 深度不足 |
| 行为 1:1 | **未完成**：表面兼容，下列子系统仍是桩或精简实现 |

### 未真正 1:1 的核心缺口

1. **`api/skey` + `qq_login`**：`login.class.php`（约 500 行）整文件未移植，仅 `do=` 分发外壳  
2. **多站点音乐 `\Musics`**：`Music` / `Kugou` / `qq` / `kuwo` / `xiami` / `migu` / `kg` / `MusicUrl` 搜索/解析不可用  
3. **`QrReader`**：无二维码解码实现  
4. **`IP_IMG`**：只回底图，未做 GD 文字叠加  
5. **邮件 SMTP**：默认写 `runtime/mail/*.eml`，非 PHPMailer 实发  
6. **极验 Geetest**：固定 failback，未对接真实极验 API  
7. **`update/net`**：依赖 `GetWangYiYunInfo` 抓取类，当前失败  
8. **控制台/登录 UI**：功能表单可用，并非原 Think 模板像素级还原  
9. **拼音词典**：`data/pinyin_dict.txt` 为精简集，非完整 `hanzi()`  
10. **明确不在范围**：`oldapi/tool/*`、`AAAAAA/*`

状态：`done` = 路由+主逻辑可用 · `partial` = 有接口但深度不足 · `broken` = 调用即失败/空壳

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
| IP_IMG | partial | 有资源时仅返回底图，无文字叠加 |
| Email | partial | 默认落盘 .eml |
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
| Music | partial | `music_search` 空实现 → data=[] |
| Music_hot | done | 读 `public/hot.json` |
| MD5 | done | |
| QrCode | done | PNG 编码 OK |
| QrReader | partial | 固定解析失败形状 |
| Mobile | partial | 归属地字段不完整 |
| whois | done | |
| pinyin | partial | 词典不全 |
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
| Music_List_163 | partial | playlist API best-effort |
| district | done | |
| WeatherInfo | done | |
| IpSadd | done | |
| Kugou | partial | 缺 `\Musics` |
| qq | partial | 缺 `\Musics` |
| kuwo | partial | 缺 `\Musics` |
| xiami | partial | 缺 `\Musics` |
| migu | partial | 缺 `\Musics` |
| kg | partial | 缺 `\Musics` |
| MusicUrl | partial | 缺 `\Musics` |
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
| Music_hot | partial | 上游 collect 失败时「更新失败」 |
| Word | done | |
| Wordtime | done | |
| netease | done | |
| headimg | done | |
| words | done | |
| qianming | done | |
| wangming | done | |
| net | broken | 缺 GetWangYiYunInfo，返回内部错误 |
| dog | done | |

## api/skey

| action | 状态 | 备注 |
|--------|------|------|
| index `do=*`（16 个） | partial | 分发齐全；业务全是 stub，非真实 QQ 登录 |

`do` 列表：checkvc, dovc, getvc, qqlogin, getqrpic, qrlogin, list, danxiang, del, authf, getqrpic3rd, qrlogin3rd, idpic, idlogin, InfoNull, NickNull

## web

| path | 状态 | 备注 |
|------|------|------|
| login/index/{index,GtCode,callback} | done | UI 简化；极验 failback |
| register/index/{index,GtCode,Email,callback} | done | 邮件落盘 |
| index/index/{index,page,setting,appkey,LoginOut,ip,log} | done | 控制台功能在，非原模板 1:1 视觉 |

## 建议补齐优先级

1. 移植 `extend/qqskey/login.class.php` → `services::qqskey`  
2. 移植 `\Musics` / `tool/Music` → `services::music`  
3. 补 `GetWangYiYunInfo` + 修 `update/net`  
4. `IP_IMG` 字体绘制、`QrReader` 解码、真实 SMTP/Geetest  
5. 还原 Askama/原 HTML 控制台模板  
