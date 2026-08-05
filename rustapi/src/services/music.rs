use reqwest::Client;
use serde_json::{json, Value};

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

pub async fn music_search(_client: &Client, _input: &str, _site: &str, _page: &str) -> Value {
    json!({"error": "music provider unavailable", "data": []})
}
