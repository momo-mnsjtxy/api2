use reqwest::Client;
use serde_json::Value;

pub async fn get_json(client: &Client, url: &str) -> Option<Value> {
    let resp = client.post(url).send().await.ok()?;
    let text = resp.text().await.ok()?;
    let trimmed = text.trim_start_matches('\u{feff}');
    serde_json::from_str(trimmed).ok()
}

pub async fn get_text(client: &Client, url: &str) -> Option<String> {
    let resp = client.get(url).send().await.ok()?;
    resp.text().await.ok()
}

pub async fn get_bytes(client: &Client, url: &str) -> Option<Vec<u8>> {
    let resp = client.get(url).send().await.ok()?;
    resp.bytes().await.ok().map(|b| b.to_vec())
}
