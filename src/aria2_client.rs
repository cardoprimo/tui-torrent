use serde::Deserialize;
use serde_json::json;
use tokio::time::{timeout, Duration};
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct TorrentStatus {
    pub gid: String,
    pub status: String,
    #[serde(rename = "totalLength")]
    pub total_length: String,
    #[serde(rename = "completedLength")]
    pub completed_length: String,
    #[serde(rename = "downloadSpeed")]
    pub download_speed: String,
    #[serde(rename = "infoHash")]
    pub info_hash: Option<String>,
    pub file_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Aria2File {
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct Aria2StatusResponse {
    pub files: Option<Vec<Aria2File>>,
}

// Static client for reuse across calls
static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::new()
    })
}

pub async fn get_active_downloads() -> Result<Vec<TorrentStatus>, reqwest::Error> {
    let client = get_client();

    // First, get active downloads
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "aria2.tellActive",
        "id": "active"
    });

    let res = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    let basic_torrents: Vec<TorrentStatus> = serde_json::from_value(json["result"].clone()).unwrap_or(vec![]);

    // Process all downloads in parallel to get file names
    let enhanced_torrents = futures::future::join_all(
        basic_torrents.into_iter().map(|torrent| async move {
            let gid = torrent.gid.clone();

            // Get detailed status for this download
            let status_payload = json!({
                "jsonrpc": "2.0",
                "method": "aria2.tellStatus",
                "id": format!("status_{}", gid),
                "params": [gid]
            });

            let file_name = match timeout(Duration::from_secs(2), async {
                let client = get_client();
                let status_res = client
                    .post("http://localhost:6800/jsonrpc")
                    .json(&status_payload)
                    .send()
                    .await;

                match status_res {
                    Ok(response) => {
                        match response.json::<serde_json::Value>().await {
                            Ok(status_json) => {
                                let status_response: Result<Aria2StatusResponse, _> = serde_json::from_value(status_json["result"].clone());

                                match status_response {
                                    Ok(response) => {
                                        // Extract file name from the first file
                                        if let Some(files) = response.files {
                                            if let Some(first_file) = files.first() {
                                                // Get just the filename from the path
                                                std::path::Path::new(&first_file.path)
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .map(|s| s.to_string())
                                            } else {
                                                None
                                            }
                                        } else {
                                            None
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse aria2 status response for {}: {}", gid, e);
                                        None
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to parse JSON response for {}: {}", gid, e);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get status for {}: {}", gid, e);
                        None
                    }
                }
            }).await {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("Timeout getting status for {}", gid);
                    None
                }
            };

            // Create enhanced torrent with file name
            TorrentStatus {
                file_name,
                ..torrent
            }
        })
    ).await;

    Ok(enhanced_torrents)
}
