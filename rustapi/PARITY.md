# 1:1 功能对照清单（相对 oldapi）

状态：`todo` / `done` / `partial`

> `done` = 路由已挂载且逻辑已按 PHP 行为移植  
> `partial` = 接口可用但依赖桩/上游能力受限（见备注）

## api/v2

| action | 状态 | 备注 |
|--------|------|------|
| index | done | |
| UserInfo | done | |
| Url | done | |
| qlogo | done | |
| Gravatar | done | |
| Bing_img | done | |
| image | done | |
| IP | done | |
| IP_IMG | partial | 缺字体/底图时 JSON 错误，非完整 GD 绘制 |
| Email | partial | 默认写 runtime/mail .eml |
| ICP | done | |
| DM_IMG | done | |
| C_IMG | done | |
| netease | done | |
| love | done | |
| Name | done | |
| autograph | done | |
| shuoshuo | done | |
| word | done | |
| Music_163 | done | |
| Music | partial | 多站点搜索依赖 services::music |
| Music_hot | done | |
| MD5 | done | |
| QrCode | done | |
| QrReader | partial | 无解码库时返回失败形状 |
| Mobile | partial | 归属地数据精简 |
| whois | done | |
| pinyin | done | 字典可扩充 data/pinyin_dict.txt |
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
| today | done | 带缓存 |
| audio | done | |
| colors | done | |
| qunlogo | done | |
| chengyu | done | |
| xiaohua | done | |
| HttpCode | done | |
| RandPass | done | |
| Go | done | |
| Music_List_163 | partial | 列表抓取 best-effort |
| district | done | 缓存 |
| WeatherInfo | done | 缓存 |
| IpSadd | done | 缓存 |
| Kugou | partial | |
| qq | partial | |
| kuwo | partial | |
| xiami | partial | |
| migu | partial | |
| kg | partial | |
| MusicUrl | partial | |
| QQChat | done | |
| QQInfo | done | |
| douyin | done | |
| qqxj | done | 缓存 |
| telxj | done | 缓存 |
| idcard | done | 缓存 |
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
| Music_hot | done | |
| Word | done | |
| Wordtime | done | |
| netease | done | |
| headimg | done | |
| words | done | |
| qianming | done | |
| wangming | done | |
| net | partial | 原 PHP scraper 类改为 best-effort |
| dog | done | |

## api/skey

| action | 状态 | 备注 |
|--------|------|------|
| index (do=*) | partial | 表面 1:1；完整 qq_login 需续移植 login.class.php |

## web

| path | 状态 |
|------|------|
| login/index/{index,GtCode,callback} | done |
| register/index/{index,GtCode,Email,callback} | done |
| index/index/{index,page,setting,appkey,LoginOut,ip,log} | done |
