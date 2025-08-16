use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct TorrentStatus {
    pub gid: String,
    pub status: String,
    pub totalLength: String,
    pub completedLength: String,
    pub downloadSpeed: String,
    pub infoHash: Option<String>,
}

pub async fn get_active_downloads() -> Result<Vec<TorrentStatus>, reqwest::Error> {
    let client = reqwest::Client::new();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "aria2.tellActive",
        "id": "1"
    });

    let res = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    let torrents = serde_json::from_value(json["result"].clone()).unwrap_or(vec![]);
    Ok(torrents)
}
