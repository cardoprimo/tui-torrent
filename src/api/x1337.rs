use crate::torrent_search::TorrentSearchResult;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct X1337Client {
    client: Client,
    base_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct TorrentInfo {
    name: String,
    size: String,
    seeders: u32,
    leechers: u32,
    magnet_link: String,
    info_hash: String,
}

impl X1337Client {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://1337x.to".to_string(),
        }
    }

    pub fn with_mirror(mirror_url: &str) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .cookie_store(true)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: mirror_url.to_string(),
        }
    }

    pub async fn search(&self, query: &str, category: Option<&str>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Try multiple mirrors if the main one fails
        let mirrors = vec![
            "https://1337x.to",
            "https://1337x.st",
            "https://x1337x.ws",
            "https://x1337x.eu",
        ];

        for mirror in mirrors {
            match self.try_search_with_mirror(mirror, query, category).await {
                Ok(results) => return Ok(results),
                Err(_) => {
                    // Silently try next mirror
                    continue;
                }
            }
        }

        // If all mirrors fail, return empty results
        Ok(Vec::new())
    }

    async fn try_search_with_mirror(&self, mirror: &str, query: &str, category: Option<&str>) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Add delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        let search_url = match category {
            Some(cat) => format!("{}/category-search/{}/{}/1/", mirror, query, cat),
            None => format!("{}/search/{}/1/", mirror, query),
        };

        // Trying to search mirror silently

        let response = self.client
            .get(&search_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .header("Accept-Encoding", "gzip, deflate")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", mirror)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let html = response.text().await?;
        self.parse_search_results(&html).await
    }



    async fn parse_search_results(&self, html: &str) -> Result<Vec<TorrentSearchResult>, Box<dyn std::error::Error + Send + Sync>> {
        let document = Html::parse_document(html);
        let row_selector = Selector::parse("tbody tr").unwrap();
        let name_selector = Selector::parse("td.coll-1 a:nth-child(2)").unwrap();
        let seeders_selector = Selector::parse("td.coll-2").unwrap();
        let leechers_selector = Selector::parse("td.coll-3").unwrap();
        let size_selector = Selector::parse("td.coll-4").unwrap();

        let mut results = Vec::new();

        for row in document.select(&row_selector) {
            if let (Some(name_elem), Some(seeders_elem), Some(leechers_elem), Some(size_elem)) = (
                row.select(&name_selector).next(),
                row.select(&seeders_selector).next(),
                row.select(&leechers_selector).next(),
                row.select(&size_selector).next(),
            ) {
                let name = name_elem.inner_html().trim().to_string();
                let href = name_elem.value().attr("href").unwrap_or("");
                
                // Parse seeders and leechers
                let seeders = seeders_elem.inner_html().trim().parse::<u32>().unwrap_or(0);
                let leechers = leechers_elem.inner_html().trim().parse::<u32>().unwrap_or(0);
                let size = size_elem.inner_html().trim().to_string();

                // Get magnet link by visiting the torrent page
                if let Ok(magnet_link) = self.get_magnet_link(href).await {
                    results.push(TorrentSearchResult {
                        name,
                        size,
                        seeders,
                        leechers,
                        magnet_link,
                        source: "1337x".to_string(),
                    });
                }

                // Limit results to avoid too many requests
                if results.len() >= 10 {
                    break;
                }
            }
        }

        Ok(results)
    }

    async fn get_magnet_link(&self, torrent_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let torrent_url = format!("{}{}", self.base_url, torrent_path);
        
        let response = self.client
            .get(&torrent_url)
            .send()
            .await?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);
        
        // Look for magnet link
        let magnet_selector = Selector::parse("a[href^='magnet:']").unwrap();
        
        if let Some(magnet_elem) = document.select(&magnet_selector).next() {
            if let Some(href) = magnet_elem.value().attr("href") {
                return Ok(href.to_string());
            }
        }

        Err("Magnet link not found".into())
    }

    pub fn get_categories() -> HashMap<&'static str, &'static str> {
        let mut categories = HashMap::new();
        categories.insert("Movies", "Movies");
        categories.insert("TV", "TV");
        categories.insert("Games", "Games");
        categories.insert("Music", "Music");
        categories.insert("Apps", "Applications");
        categories.insert("Anime", "Anime");
        categories.insert("Documentaries", "Documentaries");
        categories.insert("XXX", "XXX");
        categories.insert("Other", "Other");
        categories
    }
}

impl Default for X1337Client {
    fn default() -> Self {
        Self::new()
    }
}