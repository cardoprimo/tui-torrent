use crate::api::{X1337Client, YtsClient, PirateBayClient};
use serde::{Deserialize, Serialize};
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TorrentSearchResult {
    pub name: String,
    pub size: String,
    pub seeders: u32,
    pub leechers: u32,
    pub magnet_link: String,
    pub source: String,
}

#[derive(Clone)]
pub struct TorrentSearchEngine {
    x1337_client: X1337Client,
    yts_client: YtsClient,
    piratebay_client: PirateBayClient,
}

impl TorrentSearchEngine {
    pub fn new() -> Self {
        Self {
            x1337_client: X1337Client::new(),
            yts_client: YtsClient::new(),
            piratebay_client: PirateBayClient::new(),
        }
    }

    pub async fn search_torrents(&self, query: &str, category: Option<&str>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let mut all_results = Vec::new();

        // Search YTS (movies)
        if let Ok(Ok(mut results)) = timeout(Duration::from_secs(15), self.search_yts(query)).await {
            all_results.append(&mut results);
        }

        // Search PirateBay
        if let Ok(Ok(mut results)) = timeout(Duration::from_secs(15), self.search_piratebay(query, category)).await {
            all_results.append(&mut results);
        }

        // Search 1337x
        if let Ok(Ok(mut results)) = timeout(Duration::from_secs(15), self.search_1337x(query, category)).await {
            all_results.append(&mut results);
        }

        // Sort by seeders (descending) and limit results
        all_results.sort_by(|a, b| b.seeders.cmp(&a.seeders));
        all_results.truncate(50);
        
        Ok(all_results)
    }

    async fn search_yts(&self, query: &str) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        self.yts_client.search(query, Some(20)).await
    }

    async fn search_piratebay(&self, query: &str, _category: Option<&str>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        self.piratebay_client.search(query, None).await
    }

    async fn search_1337x(&self, query: &str, category: Option<&str>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        self.x1337_client.search(query, category).await
    }

    pub fn get_available_categories() -> std::collections::HashMap<&'static str, &'static str> {
        X1337Client::get_categories()
    }
}

impl Default for TorrentSearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Legacy function for backward compatibility
pub async fn search_torrents(query: &str) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
    let engine = TorrentSearchEngine::new();
    engine.search_torrents(query, None).await
}

// Function to add a torrent to aria2 via the RPC API
pub async fn add_torrent(magnet_link: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "aria2.addUri",
        "id": "1",
        "params": [[magnet_link]]
    });

    let res = client
        .post("http://localhost:6800/jsonrpc")
        .json(&payload)
        .send()
        .await?;

    let json: serde_json::Value = res.json().await?;
    let gid = json["result"].as_str().unwrap_or("unknown").to_string();
    
    Ok(gid)
}
