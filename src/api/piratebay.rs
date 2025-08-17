use crate::torrent_search::TorrentSearchResult;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PirateBayClient {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct PbSearchResult {
    name: String,
    info_hash: String,
    leechers: String,
    seeders: String,
    #[allow(dead_code)]
    num_files: String,
    size: String,
    #[allow(dead_code)]
    username: String,
    #[allow(dead_code)]
    added: String,
    #[allow(dead_code)]
    status: String,
    #[allow(dead_code)]
    category: String,
    #[allow(dead_code)]
    imdb: String,
}

impl PirateBayClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("TUI-Torrent/1.0")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

    Self { client }
    }

    pub async fn search(&self, query: &str, _category: Option<&str>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Use apibay.org API (unofficial but more reliable)
        let search_url = format!(
            "https://apibay.org/q.php?q={}&cat=0",
            urlencoding::encode(query)
        );

        let response = self.client
            .get(&search_url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let pb_results: Vec<PbSearchResult> = response.json().await?;
        let mut results = Vec::new();

        for pb_result in pb_results.into_iter().take(20) {
            // Skip if no seeders
            let seeders = pb_result.seeders.parse::<u32>().unwrap_or(0);
            let leechers = pb_result.leechers.parse::<u32>().unwrap_or(0);
            
            if seeders == 0 {
                continue;
            }

            let magnet_link = format!(
                "magnet:?xt=urn:btih:{}&dn={}&tr=udp://tracker.coppersurfer.tk:6969/announce&tr=udp://9.rarbg.to:2920/announce&tr=udp://tracker.opentrackr.org:1337&tr=udp://tracker.internetwarriors.net:1337/announce&tr=udp://tracker.leechers-paradise.org:6969/announce&tr=udp://tracker.coppersurfer.tk:6969/announce&tr=udp://tracker.pirateparty.gr:6969/announce&tr=udp://tracker.cyberia.is:6969/announce",
                pb_result.info_hash,
                urlencoding::encode(&pb_result.name)
            );

            results.push(TorrentSearchResult {
                name: pb_result.name,
                size: self.format_size(&pb_result.size),
                seeders,
                leechers,
                magnet_link,
                source: "PirateBay".to_string(),
            });
        }

        // Sort by seeders (descending)
        results.sort_by(|a, b| b.seeders.cmp(&a.seeders));

        Ok(results)
    }

    fn format_size(&self, size_bytes: &str) -> String {
        if let Ok(bytes) = size_bytes.parse::<u64>() {
            if bytes >= 1_073_741_824 {
                format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
            } else if bytes >= 1_048_576 {
                format!("{:.1} MB", bytes as f64 / 1_048_576.0)
            } else if bytes >= 1024 {
                format!("{:.1} KB", bytes as f64 / 1024.0)
            } else {
                format!("{} B", bytes)
            }
        } else {
            size_bytes.to_string()
        }
    }
}

impl Default for PirateBayClient {
    fn default() -> Self {
        Self::new()
    }
}