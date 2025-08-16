use tui_torrent::torrent_search::TorrentSearchEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing multi-source torrent search integration...");
    
    let engine = TorrentSearchEngine::new();
    
    // Test search
    let query = "ubuntu";
    println!("Searching for: {} (this may take a few seconds...)", query);
    
    match engine.search_torrents(query, None).await {
        Ok(results) => {
            println!("Found {} results from multiple sources:", results.len());
            for (i, result) in results.iter().enumerate().take(10) {
                println!("{}. [{}] {} - {} (S:{} L:{})", 
                    i + 1, 
                    result.source,
                    result.name, 
                    result.size, 
                    result.seeders, 
                    result.leechers
                );
            }
            
            if results.len() > 10 {
                println!("... and {} more results", results.len() - 10);
            }
        }
        Err(e) => {
            println!("Search failed: {}", e);
        }
    }
    
    // Test movie search
    println!("\n{}", "=".repeat(50));
    let movie_query = "inception";
    println!("Searching for movies: {}", movie_query);
    
    match engine.search_torrents(movie_query, Some("Movies")).await {
        Ok(results) => {
            println!("Found {} movie results:", results.len());
            for (i, result) in results.iter().enumerate().take(5) {
                println!("{}. [{}] {} - {} (S:{} L:{})", 
                    i + 1, 
                    result.source,
                    result.name, 
                    result.size, 
                    result.seeders, 
                    result.leechers
                );
            }
        }
        Err(e) => {
            println!("Movie search failed: {}", e);
        }
    }
    
    // Test categories
    println!("\nAvailable categories:");
    let categories = TorrentSearchEngine::get_available_categories();
    for (key, value) in categories {
        println!("- {}: {}", key, value);
    }
    
    Ok(())
}