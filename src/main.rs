pub mod api;
pub mod aria2_client;
pub mod aria2_manager;
pub mod app;
pub mod error;
pub mod torrent_search;
pub mod tui;

use app::{App, AppMode};
use aria2_manager::Aria2Manager;
use torrent_search::TorrentSearchEngine;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tokio::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏴‍☠️ Starting TUI Torrent...");
    
    // Initialize and start aria2 manager
    let mut aria2_manager = Aria2Manager::new();
    
    match aria2_manager.ensure_aria2_running().await {
        Ok(()) => {
            if let Ok(version) = aria2_manager.get_version().await {
                println!("📡 Connected to aria2 version: {}", version);
            }
        }
        Err(e) => {
            eprintln!("⚠️  Warning: {}", e);
            eprintln!("💡 Downloads will not work without aria2. Install it with:");
            eprintln!("   macOS: brew install aria2");
            eprintln!("   Ubuntu: sudo apt install aria2");
            eprintln!("   Or download from: https://aria2.github.io/");
            eprintln!();
            eprintln!("🔄 Continuing anyway... (search will still work)");
        }
    }

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Create app state
    let mut app = App::new();
    let search_engine = TorrentSearchEngine::new();
    
    // Update status based on aria2 availability
    if aria2_manager.is_aria2_running().await {
        app.status_message = "Ready - aria2 RPC connected".to_string();
    } else {
        app.status_message = "Ready - aria2 not available (downloads disabled)".to_string();
    }
    
    // Main loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();
    let mut last_update = Instant::now();

    loop {
        // Handle input
        app.handle_input()?;
        
        if app.should_quit {
            break;
        }

        // Handle search state
        if app.mode == AppMode::Searching && app.search_in_progress {
            let query = app.search_query.clone();
            let category = app.selected_category.clone();
            
            match search_engine.search_torrents(&query, category.as_deref()).await {
                Ok(results) => {
                    app.finish_search(results);
                }
                Err(e) => {
                    app.search_error(e.to_string());
                }
            }
        }
        
        // Handle torrent download request
        if app.download_requested && !app.search_results.is_empty() {
            if let Some(selected) = app.search_results.get(app.selected_index) {
                match torrent_search::add_torrent(&selected.magnet_link).await {
                    Ok(gid) => {
                        app.status_message = format!("Added torrent: {} (GID: {})", selected.name, gid);
                        app.mode = AppMode::Normal;
                    }
                    Err(e) => {
                        app.status_message = format!("Failed to add torrent: {}", e);
                    }
                }
            }
            app.download_requested = false;
        }

        // Update downloads list every 2 seconds
        if last_update.elapsed() >= Duration::from_secs(2) {
            app.active_downloads = aria2_client::get_active_downloads().await?;
            last_update = Instant::now();
        }

        // Render UI
        if last_tick.elapsed() >= tick_rate {
            tui::render_ui(&app)?;
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    // Clean up aria2 process
    aria2_manager.stop();
    println!("👋 Goodbye!");

    Ok(())
}
