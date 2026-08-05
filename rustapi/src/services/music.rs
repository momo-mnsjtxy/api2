use rand::Rng;
use reqwest::Client;
use serde_json::{json, Value};
use url::Url;

const DESKTOP_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/60.0.3112.50 Safari/537.36";
const MOBILE_UA: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 9_1 like Mac OS X) AppleWebKit/601.1.46 (KHTML, like Gecko) Version/9.0 Mobile/13B143 Safari/601.1";

/// Port of m_163() / music163 helpers — best-effort via public endpoints.
pub async fn m_163(client: &Client, id: &str) -> Option<Value> {
    let detail_url = format!("http://music.163.com/api/song/detail/?id={id}&ids=%5B{id}%5D");
    let lyric_url = format!("http://music.163.com/api/song/lyric?os=pc&id={id}&lv=-1&kv=-1&tv=-1");

    let detail = client
        .get(&detail_url)
        .send()
        .await
        .ok()?
        .json::<Value>()
        .await
        .ok()?;
    let lyric = client
        .get(&lyric_url)
        .send()
        .await
        .ok()?
        .json::<Value>()
        .await
        .ok()?;

    let song = detail.get("songs")?.as_array()?.first()?;
    let name = song.get("name").cloned().unwrap_or(json!(""));
    let pic = song
        .pointer("/album/blurPicUrl")
        .cloned()
        .unwrap_or(json!(""));
    let artists = song
        .get("artists")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| {
                    x.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let lrc = lyric.pointer("/lrc/lyric").cloned().unwrap_or(json!(""));

    Some(json!({
        "song_name": name,
        "artists": artists,
        "pic": pic,
        "lrc": lrc,
        "id": id,
    }))
}

pub async fn music_search(client: &Client, input: &str, site: &str, page: &str) -> Value {
    let page = page.parse::<i64>().unwrap_or(1).max(1);
    let site = normalize_site(site);
    let filter = if (site.is_empty() || site == "_") && looks_like_url(input) {
        "url"
    } else if is_supported_site(site) && looks_like_provider_id(input, site) {
        "id"
    } else {
        "name"
    };

    data(client, input, filter, site, page).await
}

pub async fn data(client: &Client, input: &str, filter: &str, site: &str, page: i64) -> Value {
    let input = input.trim();
    let filter = filter.trim().to_ascii_lowercase();
    let site = normalize_site(site);
    let page = page.max(1);

    if input.is_empty() {
        return error_response("请求的数据错误");
    }

    let result = match filter.as_str() {
        "name" => {
            if !is_supported_site(site) {
                Err("抱歉不支持该网站".to_string())
            } else {
                search_by_name(client, input, site, page).await
            }
        }
        "id" => {
            if !is_supported_site(site) {
                Err("抱歉不支持该网站".to_string())
            } else {
                get_song_by_id(client, input, site).await
            }
        }
        "url" => get_song_by_url(client, input).await,
        _ => Err("请求的数据错误".to_string()),
    };

    match result {
        Ok(items) if !items.is_empty() => json!({"code": "success", "data": items}),
        Ok(_) => error_response("抱歉没有找到相关信息"),
        Err(err) => error_response(err),
    }
}

async fn search_by_name(
    client: &Client,
    query: &str,
    site: &str,
    page: i64,
) -> Result<Vec<Value>, String> {
    match site {
        "netease" => netease_search(client, query, page).await,
        "kugou" => kugou_search(client, query, page).await,
        "qq" => qq_search(client, query, page).await,
        "kuwo" => kuwo_search(client, query, page).await,
        "migu" => migu_search(client, query, page).await,
        "xiami" => xiami_search(client, query, page).await,
        "kg" => kg_search(client, query, page).await,
        _ => Err("抱歉不支持该网站".to_string()),
    }
}

async fn get_song_by_id(client: &Client, songid: &str, site: &str) -> Result<Vec<Value>, String> {
    match site {
        "netease" => netease_get_song(client, songid).await,
        "kugou" => kugou_get_song(client, songid).await,
        "qq" => qq_get_song(client, songid).await,
        "kuwo" => kuwo_get_song(client, songid).await,
        "migu" => migu_get_song(client, songid).await,
        "xiami" => xiami_get_song(client, songid).await,
        "kg" => kg_get_song(client, songid).await,
        _ => Err("抱歉不支持该网站".to_string()),
    }
}

async fn get_song_by_url(client: &Client, input: &str) -> Result<Vec<Value>, String> {
    let Some((site, id, is_search)) = parse_music_url(input) else {
        return Err("请检查您的输入是否正确".to_string());
    };

    if is_search {
        search_by_name(client, &id, &site, 1).await
    } else {
        get_song_by_id(client, &id, &site).await
    }
}

async fn netease_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let offset = ((page - 1) * 10).to_string();
    let params = vec![
        ("s", query.to_string()),
        ("type", "1".to_string()),
        ("offset", offset),
        ("limit", "10".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "http://music.163.com/api/search/get/web",
        &params,
        Some("http://music.163.com/"),
        DESKTOP_UA,
    )
    .await?;

    let songs = body
        .pointer("/result/songs")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let mut out = Vec::new();
    for song in songs {
        if let Some(item) = netease_song_from_value(client, song, false).await {
            out.push(item);
        }
    }
    Ok(out)
}

async fn netease_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let ids = format!("[{songid}]");
    let params = vec![("id", songid.to_string()), ("ids", ids)];
    let body = http_json(
        client,
        "GET",
        "http://music.163.com/api/song/detail/",
        &params,
        Some("http://music.163.com/"),
        DESKTOP_UA,
    )
    .await?;
    let songs = body
        .get("songs")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;

    let mut out = Vec::new();
    for song in songs {
        if let Some(item) = netease_song_from_value(client, song, true).await {
            out.push(item);
        }
    }
    Ok(out)
}

async fn netease_song_from_value(
    client: &Client,
    song: &Value,
    detail_shape: bool,
) -> Option<Value> {
    let songid = value_to_string(song.get("id"))?;
    let title = value_to_string(song.get("name")).unwrap_or_default();
    let author = join_names(song.get(if detail_shape { "artists" } else { "artists" }))
        .or_else(|| join_names(song.get("ar")))
        .unwrap_or_default();
    let pic = first_string(song, &["/album/picUrl", "/album/blurPicUrl", "/al/picUrl"])
        .map(|p| format!("{p}?param=300x300"))
        .unwrap_or_default();
    let lrc = netease_lyric(client, &songid).await.unwrap_or_default();

    Some(song_json(
        "netease",
        format!("http://music.163.com/#/song?id={songid}"),
        songid.clone(),
        title,
        author,
        lrc,
        format!("http://music.163.com/song/media/outer/url?id={songid}.mp3"),
        pic,
    ))
}

async fn netease_lyric(client: &Client, songid: &str) -> Option<String> {
    let params = vec![
        ("os", "pc".to_string()),
        ("id", songid.to_string()),
        ("lv", "-1".to_string()),
        ("kv", "-1".to_string()),
        ("tv", "-1".to_string()),
    ];
    http_json(
        client,
        "GET",
        "http://music.163.com/api/song/lyric",
        &params,
        Some("http://music.163.com/"),
        DESKTOP_UA,
    )
    .await
    .ok()
    .and_then(|v| {
        v.pointer("/lrc/lyric")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

async fn kugou_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let params = vec![
        ("keyword", query.to_string()),
        ("platform", "WebFilter".to_string()),
        ("format", "json".to_string()),
        ("page", page.to_string()),
        ("pagesize", "10".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "http://songsearch.kugou.com/song_search_v2",
        &params,
        Some("http://www.kugou.com"),
        DESKTOP_UA,
    )
    .await?;

    let list = body
        .pointer("/data/lists")
        .and_then(Value::as_array)
        .or_else(|| body.pointer("/data/info").and_then(Value::as_array))
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;

    let mut out = Vec::new();
    for item in list {
        let hash = first_string(item, &["/HQFileHash", "/FileHash", "/320hash", "/hash"])
            .unwrap_or_default();
        if hash.is_empty() || hash.chars().all(|c| c == '0') {
            continue;
        }
        if let Ok(mut songs) = kugou_get_song(client, &hash).await {
            out.append(&mut songs);
        }
    }
    Ok(out)
}

async fn kugou_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let params = vec![
        ("cmd", "playInfo".to_string()),
        ("hash", songid.to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "http://m.kugou.com/app/i/getSongInfo.php",
        &params,
        Some(&format!("http://m.kugou.com/play/info/{songid}")),
        MOBILE_UA,
    )
    .await?;

    let url = value_to_string(body.get("url")).unwrap_or_default();
    if url.is_empty() {
        let paid = body
            .get("privilege")
            .and_then(Value::as_i64)
            .unwrap_or_default()
            != 0;
        return Err(if paid {
            "源站反馈此音频需要付费".to_string()
        } else {
            "找不到可用的播放地址".to_string()
        });
    }

    let hash = value_to_string(body.get("hash")).unwrap_or_else(|| songid.to_string());
    let album_img = value_to_string(body.get("album_img")).unwrap_or_default();
    let img = value_to_string(body.get("imgUrl")).unwrap_or_default();
    let pic = if album_img.is_empty() { img } else { album_img }.replace("{size}", "150");
    let lrc = kugou_lyric(client, &hash).await.unwrap_or_default();
    Ok(vec![song_json(
        "kugou",
        format!("http://www.kugou.com/song/#hash={hash}"),
        hash,
        value_to_string(body.get("songName")).unwrap_or_default(),
        value_to_string(body.get("singerName")).unwrap_or_default(),
        lrc,
        url,
        pic,
    )])
}

async fn kugou_lyric(client: &Client, songid: &str) -> Option<String> {
    let params = vec![
        ("cmd", "100".to_string()),
        ("timelength", "999999".to_string()),
        ("hash", songid.to_string()),
    ];
    http_text(
        client,
        "GET",
        "http://m.kugou.com/app/i/krc.php",
        &params,
        Some(&format!("http://m.kugou.com/play/info/{songid}")),
        MOBILE_UA,
    )
    .await
    .ok()
}

async fn qq_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let params = vec![
        ("w", query.to_string()),
        ("p", page.to_string()),
        ("n", "10".to_string()),
        ("format", "json".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "http://c.y.qq.com/soso/fcgi-bin/search_for_qq_cp",
        &params,
        Some("http://m.y.qq.com"),
        MOBILE_UA,
    )
    .await?;
    let list = body
        .pointer("/data/song/list")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;

    let mids: Vec<String> = list
        .iter()
        .filter(|song| song.pointer("/pay/payplay").and_then(Value::as_i64) != Some(1))
        .filter_map(|song| value_to_string(song.get("songmid")))
        .collect();
    let urls = qq_song_urls(client, &mids).await.unwrap_or_default();

    let mut out = Vec::new();
    let mut playable_index = 0usize;
    for song in list {
        if song.pointer("/pay/payplay").and_then(Value::as_i64) == Some(1) {
            continue;
        }
        let Some(mid) = value_to_string(song.get("songmid")) else {
            continue;
        };
        let author = join_names(song.get("singer")).unwrap_or_default();
        let lrc = qq_lyric(client, &mid).await.unwrap_or_default();
        let url = urls.get(playable_index).cloned().unwrap_or_default();
        playable_index += 1;
        let album_mid = value_to_string(song.get("albummid")).unwrap_or_default();
        out.push(song_json(
            "qq",
            format!("http://y.qq.com/n/yqq/song/{mid}.html"),
            mid,
            value_to_string(song.get("songname")).unwrap_or_default(),
            author,
            lrc,
            url,
            format!("http://y.gtimg.cn/music/photo_new/T002R300x300M000{album_mid}.jpg"),
        ));
    }
    Ok(out)
}

async fn qq_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let params = vec![
        ("songmid", songid.to_string()),
        ("format", "json".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "http://c.y.qq.com/v8/fcg-bin/fcg_play_single_song.fcg",
        &params,
        Some("http://m.y.qq.com"),
        MOBILE_UA,
    )
    .await?;
    let list = body
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let mids: Vec<String> = list
        .iter()
        .filter_map(|song| value_to_string(song.get("mid")))
        .collect();
    let urls = qq_song_urls(client, &mids).await.unwrap_or_default();

    let mut out = Vec::new();
    for (idx, song) in list.iter().enumerate() {
        if song.pointer("/pay/pay_play").and_then(Value::as_i64) == Some(1) {
            return Err("源站反馈此音频需要付费".to_string());
        }
        let mid = value_to_string(song.get("mid")).unwrap_or_else(|| songid.to_string());
        let author = join_names(song.get("singer")).unwrap_or_default();
        let lrc = qq_lyric(client, &mid).await.unwrap_or_default();
        let album_mid = first_string(song, &["/album/mid"]).unwrap_or_default();
        out.push(song_json(
            "qq",
            format!("http://y.qq.com/n/yqq/song/{mid}.html"),
            mid,
            value_to_string(song.get("title")).unwrap_or_default(),
            author,
            lrc,
            urls.get(idx).cloned().unwrap_or_default(),
            format!("http://y.gtimg.cn/music/photo_new/T002R300x300M000{album_mid}.jpg"),
        ));
    }
    Ok(out)
}

async fn qq_song_urls(client: &Client, songids: &[String]) -> Result<Vec<String>, String> {
    if songids.is_empty() {
        return Ok(Vec::new());
    }
    let guid = rand::thread_rng()
        .gen_range(111_111_111..=999_999_999)
        .to_string();
    let songtypes = vec![0; songids.len()];
    let req = json!({
        "req": {
            "module": "CDN.SrfCdnDispatchServer",
            "method": "GetCdnDispatch",
            "param": {"guid": guid, "calltype": 0, "userip": ""}
        },
        "req_0": {
            "module": "vkey.GetVkeyServer",
            "method": "CgiGetVkey",
            "param": {
                "guid": guid,
                "songmid": songids,
                "songtype": songtypes,
                "uin": "0",
                "loginflag": 1,
                "platform": "20"
            }
        },
        "comm": {"uin": 0, "format": "json", "ct": 24, "cv": 0}
    });
    let params = vec![("data", req.to_string())];
    let body = http_json(
        client,
        "GET",
        "https://u.y.qq.com/cgi-bin/musicu.fcg",
        &params,
        Some("https://y.qq.com/portal/player.html"),
        DESKTOP_UA,
    )
    .await?;
    let server = body
        .pointer("/req/data/sip/0")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let urls = body
        .pointer("/req_0/data/midurlinfo")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    let purl = value_to_string(item.get("purl")).unwrap_or_default();
                    if purl.is_empty() {
                        String::new()
                    } else {
                        format!("{server}{purl}")
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(urls)
}

async fn qq_lyric(client: &Client, songid: &str) -> Option<String> {
    let params = vec![
        ("songmid", songid.to_string()),
        ("format", "json".to_string()),
        ("nobase64", "1".to_string()),
        ("songtype", "0".to_string()),
        ("callback", "c".to_string()),
    ];
    let text = http_text(
        client,
        "GET",
        "http://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric.fcg",
        &params,
        Some("http://m.y.qq.com"),
        MOBILE_UA,
    )
    .await
    .ok()?;
    let json = parse_maybe_jsonp(&text).ok()?;
    json.get("lyric")
        .and_then(Value::as_str)
        .map(decode_qq_text)
}

async fn kuwo_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let params = vec![
        ("all", query.to_string()),
        ("ft", "music".to_string()),
        ("itemset", "web_2013".to_string()),
        ("pn", (page - 1).to_string()),
        ("rn", "10".to_string()),
        ("rformat", "json".to_string()),
        ("encoding", "utf8".to_string()),
    ];
    let text = http_text(
        client,
        "GET",
        "http://search.kuwo.cn/r.s",
        &params,
        Some("http://player.kuwo.cn/webmusic/play"),
        DESKTOP_UA,
    )
    .await?;
    let body: Value = serde_json::from_str(&text.replace('\'', "\""))
        .map_err(|_| "酷我音乐搜索结果解析失败".to_string())?;
    let list = body
        .get("abslist")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let mut out = Vec::new();
    for item in list {
        let id = value_to_string(item.get("MUSICRID"))
            .unwrap_or_default()
            .replace("MUSIC_", "");
        if id.is_empty() {
            continue;
        }
        if let Ok(mut songs) = kuwo_get_song(client, &id).await {
            out.append(&mut songs);
        }
    }
    Ok(out)
}

async fn kuwo_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let params = vec![("rid", format!("MUSIC_{songid}"))];
    let text = http_text(
        client,
        "GET",
        "http://player.kuwo.cn/webmusic/st/getNewMuiseByRid",
        &params,
        Some("http://player.kuwo.cn/webmusic/play"),
        DESKTOP_UA,
    )
    .await?;
    let id = extract_xml_tag(&text, "music_id").unwrap_or_else(|| songid.to_string());
    let title = extract_xml_tag(&text, "name").unwrap_or_default();
    let author = extract_xml_tag(&text, "singer").unwrap_or_default();
    let url = match (
        extract_xml_tag(&text, "mp3dl"),
        extract_xml_tag(&text, "mp3path"),
    ) {
        (Some(host), Some(path)) if !host.is_empty() && !path.is_empty() => {
            format!("http://{host}/resource/{path}")
        }
        _ => kuwo_song_url(client, &id).await.unwrap_or_default(),
    };
    let lrc = kuwo_lyric(client, &id).await.unwrap_or_default();
    Ok(vec![song_json(
        "kuwo",
        format!("http://www.kuwo.cn/yinyue/{id}"),
        id,
        title,
        author,
        lrc,
        url,
        extract_xml_tag(&text, "artist_pic").unwrap_or_default(),
    )])
}

async fn kuwo_song_url(client: &Client, songid: &str) -> Option<String> {
    let params = vec![
        ("type", "convert_url3".to_string()),
        ("rid", songid.to_string()),
        ("format", "mp3".to_string()),
        ("response", "url".to_string()),
    ];
    http_text(
        client,
        "GET",
        "https://antiserver.kuwo.cn/anti.s",
        &params,
        Some("http://www.kuwo.cn/"),
        DESKTOP_UA,
    )
    .await
    .ok()
    .map(|s| s.trim().to_string())
    .filter(|s| s.starts_with("http"))
}

async fn kuwo_lyric(client: &Client, songid: &str) -> Option<String> {
    let params = vec![("musicId", songid.to_string())];
    let body = http_json(
        client,
        "GET",
        "http://m.kuwo.cn/newh5/singles/songinfoandlrc",
        &params,
        Some(&format!("http://m.kuwo.cn/yinyue/{songid}")),
        MOBILE_UA,
    )
    .await
    .ok()?;
    let list = body.pointer("/data/lrclist")?.as_array()?;
    Some(generate_kuwo_lrc(list))
}

async fn migu_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let params = vec![
        ("keyword", query.to_string()),
        ("type", "2".to_string()),
        ("pgc", page.to_string()),
        ("rows", "10".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "http://m.music.migu.cn/migu/remoting/scr_search_tag",
        &params,
        Some("http://m.music.migu.cn"),
        MOBILE_UA,
    )
    .await?;
    let list = body
        .get("musics")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let mut out = Vec::new();
    for item in list {
        let id = value_to_string(item.get("copyrightId")).unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        let lrc = if item.get("lyrics").and_then(Value::as_bool).unwrap_or(false) {
            migu_lyric(client, &id).await.unwrap_or_default()
        } else {
            String::new()
        };
        out.push(song_json(
            "migu",
            format!("http://music.migu.cn/v3/music/song/{id}"),
            id,
            value_to_string(item.get("songName")).unwrap_or_default(),
            value_to_string(item.get("singerName")).unwrap_or_default(),
            lrc,
            value_to_string(item.get("mp3")).unwrap_or_default(),
            value_to_string(item.get("cover")).unwrap_or_default(),
        ));
    }
    Ok(out)
}

async fn migu_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let params = vec![("cpid", songid.to_string())];
    let body = http_json(
        client,
        "GET",
        "http://m.music.migu.cn/migu/remoting/cms_detail_tag",
        &params,
        Some(&format!("http://m.music.migu.cn/v3/music/song/{songid}")),
        MOBILE_UA,
    )
    .await?;
    let data = body
        .get("data")
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let id = value_to_string(data.get("copyrightId")).unwrap_or_else(|| songid.to_string());
    Ok(vec![song_json(
        "migu",
        format!("http://music.migu.cn/v3/music/song/{id}"),
        id,
        value_to_string(data.get("songName")).unwrap_or_default(),
        join_or_string(data.get("singerName")).unwrap_or_default(),
        value_to_string(data.get("lyricLrc")).unwrap_or_default(),
        first_string(data, &["/listenUrl", "/mp3"]).unwrap_or_default(),
        value_to_string(data.get("picS")).unwrap_or_default(),
    )])
}

async fn migu_lyric(client: &Client, songid: &str) -> Option<String> {
    let params = vec![("copyrightId", songid.to_string())];
    http_json(
        client,
        "GET",
        "http://music.migu.cn/v3/api/music/audioPlayer/getLyric",
        &params,
        Some("http://music.migu.cn/v3/music/player/audio"),
        DESKTOP_UA,
    )
    .await
    .ok()
    .and_then(|v| v.get("lyric").and_then(Value::as_str).map(str::to_string))
}

async fn xiami_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let params = vec![
        ("key", query.to_string()),
        ("v", "2.0".to_string()),
        ("app_key", "1".to_string()),
        ("r", "search/songs".to_string()),
        ("page", page.to_string()),
        ("limit", "10".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "https://api.xiami.com/web",
        &params,
        Some("https://m.xiami.com"),
        MOBILE_UA,
    )
    .await?;
    let list = body
        .pointer("/data/songs")
        .and_then(Value::as_array)
        .ok_or_else(|| "虾米音乐接口不可用或没有找到相关信息".to_string())?;
    let mut out = Vec::new();
    for item in list {
        let id = value_to_string(item.get("song_id")).unwrap_or_default();
        let lyric_url = value_to_string(item.get("lyric")).unwrap_or_default();
        let lrc = fetch_optional_text(client, &lyric_url)
            .await
            .unwrap_or_default();
        out.push(song_json(
            "xiami",
            format!("https://www.xiami.com/song/{id}"),
            id,
            value_to_string(item.get("song_name")).unwrap_or_default(),
            value_to_string(item.get("artist_name")).unwrap_or_default(),
            strip_tags(&lrc),
            value_to_string(item.get("listen_file")).unwrap_or_default(),
            format!(
                "{}@!c-400-400",
                value_to_string(item.get("album_logo")).unwrap_or_default()
            ),
        ));
    }
    Ok(out)
}

async fn xiami_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let params = vec![
        ("v", "2.0".to_string()),
        ("app_key", "1".to_string()),
        ("id", songid.to_string()),
        ("r", "song/detail".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "https://api.xiami.com/web",
        &params,
        Some("https://m.xiami.com"),
        MOBILE_UA,
    )
    .await?;
    let song = body.pointer("/data/song").ok_or_else(|| {
        value_to_string(body.get("message"))
            .unwrap_or_else(|| "虾米音乐接口不可用或没有找到相关信息".to_string())
    })?;
    let id = value_to_string(song.get("song_id")).unwrap_or_else(|| songid.to_string());
    let lyric_url = value_to_string(song.get("lyric")).unwrap_or_default();
    let lrc = fetch_optional_text(client, &lyric_url)
        .await
        .unwrap_or_default();
    Ok(vec![song_json(
        "xiami",
        format!("https://www.xiami.com/song/{id}"),
        id,
        value_to_string(song.get("song_name")).unwrap_or_default(),
        value_to_string(song.get("singers")).unwrap_or_default(),
        strip_tags(&lrc),
        value_to_string(song.get("listen_file")).unwrap_or_default(),
        format!(
            "{}@!c-400-400",
            value_to_string(song.get("logo")).unwrap_or_default()
        ),
    )])
}

async fn kg_search(client: &Client, query: &str, page: i64) -> Result<Vec<Value>, String> {
    let params = vec![
        ("format", "json".to_string()),
        ("type", "get_ugc".to_string()),
        ("inCharset", "utf8".to_string()),
        ("outCharset", "utf-8".to_string()),
        ("share_uid", query.to_string()),
        ("start", page.to_string()),
        ("num", "10".to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "https://kg.qq.com/cgi/kg_ugc_get_homepage",
        &params,
        Some("https://kg.qq.com"),
        DESKTOP_UA,
    )
    .await?;
    let list = body
        .pointer("/data/ugclist")
        .and_then(Value::as_array)
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let mut out = Vec::new();
    for item in list {
        let id = value_to_string(item.get("shareid")).unwrap_or_default();
        if id.is_empty() {
            continue;
        }
        if let Ok(mut songs) = kg_get_song(client, &id).await {
            out.append(&mut songs);
        }
    }
    Ok(out)
}

async fn kg_get_song(client: &Client, songid: &str) -> Result<Vec<Value>, String> {
    let params = vec![
        ("v", "4".to_string()),
        ("format", "json".to_string()),
        ("inCharset", "utf8".to_string()),
        ("outCharset", "utf-8".to_string()),
        ("shareid", songid.to_string()),
    ];
    let body = http_json(
        client,
        "GET",
        "https://kg.qq.com/cgi/kg_ugc_getdetail",
        &params,
        Some("https://kg.qq.com"),
        DESKTOP_UA,
    )
    .await?;
    let data = body
        .get("data")
        .ok_or_else(|| "抱歉没有找到相关信息".to_string())?;
    let id = songid.to_string();
    let uid = value_to_string(data.get("uid")).unwrap_or_default();
    let lrc = kg_lyric(client, &id).await.unwrap_or_default();
    Ok(vec![song_json(
        "kg",
        format!("https://kg.qq.com/node/play?s={id}&shareuid={uid}"),
        id,
        value_to_string(data.get("song_name")).unwrap_or_default(),
        value_to_string(data.get("nick")).unwrap_or_default(),
        lrc,
        value_to_string(data.get("playurl")).unwrap_or_default(),
        value_to_string(data.get("cover")).unwrap_or_default(),
    )])
}

async fn kg_lyric(client: &Client, songid: &str) -> Option<String> {
    let params = vec![
        ("format", "json".to_string()),
        ("inCharset", "utf8".to_string()),
        ("outCharset", "utf-8".to_string()),
        ("ksongmid", songid.to_string()),
    ];
    http_json(
        client,
        "GET",
        "https://kg.qq.com/cgi/fcg_lyric",
        &params,
        Some("https://kg.qq.com"),
        DESKTOP_UA,
    )
    .await
    .ok()
    .and_then(|v| {
        v.pointer("/data/lyric")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

async fn fetch_optional_text(client: &Client, url: &str) -> Option<String> {
    if !looks_like_url(url) {
        return None;
    }
    client
        .get(url)
        .header("User-Agent", DESKTOP_UA)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()
}

async fn http_json(
    client: &Client,
    method: &str,
    url: &str,
    params: &[(&str, String)],
    referer: Option<&str>,
    ua: &str,
) -> Result<Value, String> {
    let text = http_text(client, method, url, params, referer, ua).await?;
    parse_maybe_jsonp(&text)
}

async fn http_text(
    client: &Client,
    method: &str,
    url: &str,
    params: &[(&str, String)],
    referer: Option<&str>,
    ua: &str,
) -> Result<String, String> {
    let mut request = if method.eq_ignore_ascii_case("POST") {
        client.post(url).form(params)
    } else {
        client.get(url).query(params)
    }
    .header("User-Agent", ua)
    .header("X-Requested-With", "XMLHttpRequest");

    if let Some(referer) = referer {
        request = request.header("Referer", referer);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求源站失败: {e}"))?;
    response
        .text()
        .await
        .map_err(|e| format!("读取源站响应失败: {e}"))
}

fn parse_maybe_jsonp(text: &str) -> Result<Value, String> {
    let trimmed = text.trim();
    let json_text = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        trimmed
    } else if let (Some(start), Some(end)) = (trimmed.find('('), trimmed.rfind(')')) {
        trimmed[start + 1..end].trim()
    } else {
        trimmed
    };
    serde_json::from_str(json_text).map_err(|e| format!("源站响应解析失败: {e}"))
}

fn normalize_site(site: &str) -> &str {
    let site = site.trim();
    if site.eq_ignore_ascii_case("qmkg") {
        "kg"
    } else {
        site
    }
}

fn is_supported_site(site: &str) -> bool {
    matches!(
        site,
        "netease" | "kugou" | "qq" | "kuwo" | "migu" | "xiami" | "kg"
    )
}

fn looks_like_url(input: &str) -> bool {
    input.starts_with("http://") || input.starts_with("https://")
}

fn looks_like_provider_id(input: &str, site: &str) -> bool {
    let input = input.trim();
    if input.is_empty() || input.chars().any(char::is_whitespace) {
        return false;
    }
    match site {
        "netease" | "kuwo" | "xiami" => input.chars().all(|c| c.is_ascii_digit()),
        "migu" => input.len() >= 8 && input.chars().all(|c| c.is_ascii_alphanumeric()),
        "kugou" => input.len() >= 24 && input.chars().all(|c| c.is_ascii_hexdigit()),
        "qq" => input.len() >= 10 && input.chars().all(|c| c.is_ascii_alphanumeric()),
        "kg" => {
            input.len() >= 6
                && input
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        }
        _ => false,
    }
}

fn parse_music_url(input: &str) -> Option<(String, String, bool)> {
    let url = Url::parse(input).ok()?;
    let host = url.host_str()?.to_ascii_lowercase();
    let path = url.path();
    let fragment = url.fragment().unwrap_or_default();

    if host.contains("music.163.com") {
        if let Some(id) = url
            .query_pairs()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.to_string())
        {
            return Some(("netease".to_string(), id, false));
        }
        if let Some(id) = query_param_from_text(fragment, "id") {
            return Some(("netease".to_string(), id, false));
        }
        if let Some(id) = path_id_after(path, "song") {
            return Some(("netease".to_string(), id, false));
        }
    }
    if host.contains("kugou.com") {
        if let Some(hash) = url
            .query_pairs()
            .find(|(k, _)| k == "hash")
            .map(|(_, v)| v.to_string())
        {
            return Some(("kugou".to_string(), hash, false));
        }
        if let Some(hash) = query_param_from_text(fragment, "hash") {
            return Some(("kugou".to_string(), hash, false));
        }
        if let Some(id) = path_id_after(path, "info") {
            return Some(("kugou".to_string(), id, false));
        }
    }
    if host.contains("kuwo.cn") {
        if let Some(id) = path_id_after(path, "yinyue").or_else(|| path_id_after(path, "my")) {
            return Some(("kuwo".to_string(), id, false));
        }
    }
    if host.contains("y.qq.com") || host.contains("data.music.qq.com") {
        if let Some(mid) = url
            .query_pairs()
            .find(|(k, _)| k == "songmid")
            .map(|(_, v)| v.to_string())
        {
            return Some(("qq".to_string(), mid, false));
        }
        if let Some(id) = path_id_after(path, "song") {
            return Some((
                "qq".to_string(),
                id.trim_end_matches(".html").to_string(),
                false,
            ));
        }
        if let Some(id) = path_id_after(fragment, "song") {
            return Some((
                "qq".to_string(),
                id.trim_end_matches(".html").to_string(),
                false,
            ));
        }
    }
    if host.contains("xiami.com") {
        if let Some(id) = path_id_after(path, "song") {
            return Some(("xiami".to_string(), id, false));
        }
    }
    if host.contains("music.migu.cn") {
        if let Some(id) = path_id_after(path, "song") {
            return Some(("migu".to_string(), id, false));
        }
    }
    if host.contains("kg.qq.com") {
        if path.contains("personal") {
            if let Some(uid) = url
                .query_pairs()
                .find(|(k, _)| k == "uid")
                .map(|(_, v)| v.to_string())
            {
                return Some(("kg".to_string(), uid, true));
            }
        }
        if let Some(id) = url
            .query_pairs()
            .find(|(k, _)| k == "s")
            .map(|(_, v)| v.to_string())
        {
            return Some(("kg".to_string(), id, false));
        }
    }

    None
}

fn query_param_from_text(text: &str, key: &str) -> Option<String> {
    text.split(['?', '#', '&'])
        .find_map(|part| part.strip_prefix(&format!("{key}=")))
        .map(|value| value.split('&').next().unwrap_or(value).to_string())
        .filter(|value| !value.is_empty())
}

fn path_id_after(path: &str, marker: &str) -> Option<String> {
    let mut segments = path.split('/').filter(|s| !s.is_empty());
    while let Some(segment) = segments.next() {
        if segment.eq_ignore_ascii_case(marker) {
            return segments.next().map(|s| s.to_string());
        }
    }
    None
}

fn song_json(
    site: &str,
    link: String,
    songid: String,
    title: String,
    author: String,
    lrc: String,
    url: String,
    pic: String,
) -> Value {
    json!({
        "type": site,
        "link": link,
        "songid": songid,
        "title": title,
        "author": author,
        "lrc": lrc,
        "url": url,
        "pic": pic,
    })
}

fn error_response(message: impl Into<String>) -> Value {
    json!({"code": "error", "error": message.into()})
}

fn value_to_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "1" } else { "0" }.to_string()),
        _ => None,
    }
}

fn first_string(value: &Value, paths: &[&str]) -> Option<String> {
    paths.iter().find_map(|path| {
        value
            .pointer(path)
            .and_then(|v| value_to_string(Some(v)))
            .filter(|s| !s.is_empty())
    })
}

fn join_names(value: Option<&Value>) -> Option<String> {
    let names = value?
        .as_array()?
        .iter()
        .filter_map(|item| {
            first_string(item, &["/name", "/title"])
                .or_else(|| value_to_string(Some(item)))
                .filter(|s| !s.is_empty())
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        None
    } else {
        Some(names.join(","))
    }
}

fn join_or_string(value: Option<&Value>) -> Option<String> {
    if let Some(joined) = join_names(value) {
        return Some(joined);
    }
    match value? {
        Value::Array(items) => {
            let values = items
                .iter()
                .filter_map(|item| value_to_string(Some(item)))
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join(","))
            }
        }
        other => value_to_string(Some(other)),
    }
}

fn extract_xml_tag(text: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");
    let start = text.find(&start_tag)? + start_tag.len();
    let end = text[start..].find(&end_tag)? + start;
    Some(text[start..end].to_string())
}

fn generate_kuwo_lrc(list: &[Value]) -> String {
    let mut lrc = String::new();
    for item in list {
        let time = item
            .get("time")
            .and_then(Value::as_str)
            .and_then(|s| s.parse::<f64>().ok())
            .or_else(|| item.get("time").and_then(Value::as_f64))
            .unwrap_or_default();
        let minutes = (time / 60.0).floor() as u64;
        let seconds = time - (minutes * 60) as f64;
        let line = value_to_string(item.get("lineLyric")).unwrap_or_default();
        lrc.push_str(&format!("[{minutes:02}:{seconds:05.2}]{line}\n"));
    }
    lrc
}

fn decode_qq_text(input: &str) -> String {
    input
        .replace("&#13;", "")
        .replace("&#10;", "\n")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn strip_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}
