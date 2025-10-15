pub mod api;
pub mod app;
pub mod aria2_client;
pub mod aria2_manager;
pub mod ascii_art;
pub mod error;
pub mod storage;
pub mod torrent_search;
pub mod tui;
pub mod types;
pub mod utils;

use app::{App, AppMode};
use aria2_manager::Aria2Manager;
use ascii_art::TUI_BIRD_TITLE;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use tokio::time::{Duration, Instant};
use torrent_search::TorrentSearchEngine;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", TUI_BIRD_TITLE);
    println!("🏴‍☠️ Starting TUI Torrent...");

    let mut aria2_manager = Aria2Manager::new();

    let aria2_available = match aria2_manager.ensure_aria2_running().await {
        Ok(()) => {
            if let Ok(version) = aria2_manager.get_version().await {
                println!("📡 Connected to aria2 version: {}", version);
            } else {
                println!("📡 Connected to aria2");
            }
            println!(
                "📁 Downloads will be saved to: {}",
                aria2_manager.get_download_dir()
            );
            true
        }
        Err(e) => {
            eprintln!("⚠️  Warning: {}", e);
            eprintln!("💡 Downloads will not work without aria2. Install it with:");
            eprintln!("   macOS: brew install aria2");
            eprintln!("   Ubuntu: sudo apt install aria2");
            eprintln!();
            eprintln!("🔄 Continuing anyway... (search will still work)");
            false
        }
    };

    println!("🚀 Starting TUI interface...");

    // Small delay to let user see the startup messages
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Setup terminal
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    // Create terminal once and reuse
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Create app state
    let mut app = App::new();
    let search_engine = TorrentSearchEngine::new();

    // Track if we've already rendered the initial searching frame
    let mut initial_search_frame_rendered = false;

    // Update status based on aria2 availability
    if aria2_available {
        let download_dir = aria2_manager.get_download_dir();
        let short_path = if download_dir.len() > 40 {
            format!("...{}", &download_dir[download_dir.len() - 37..])
        } else {
            download_dir
        };
        app.status_message = format!("Ready - Downloads: {} - Press 's' to search", short_path);
    } else {
        app.status_message = "Ready - Search only (aria2 not available)".to_string();
    }

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let loading_tick_rate = Duration::from_millis(150); // Faster animation during search
    let mut last_tick = Instant::now();
    let mut last_update = Instant::now();

    loop {
        app.handle_input()?;

        if app.should_quit {
            break;
        }

        // Perform search (blocking) but ensure searching frame displayed first
        if app.mode == AppMode::Searching && app.search_in_progress {
            if !initial_search_frame_rendered {
                // Immediate render to show user the searching state before network calls
                tui::render_ui(&mut terminal, &app)?;
                initial_search_frame_rendered = true;
            } else {
                // Execute the search now
                let query = app.search_query.clone();
                let category = app.selected_category.clone();
                match search_engine
                    .search_torrents(&query, category.as_deref())
                    .await
                {
                    Ok(results) => app.finish_search(results),
                    Err(e) => app.search_error(e.to_string()),
                }
                initial_search_frame_rendered = false; // reset for next time
            }
        } else {
            initial_search_frame_rendered = false; // reset if we leave searching mode
        }

        // Handle torrent download request
        if app.download_requested && !app.search_results.is_empty() {
            if let Some(selected) = app.search_results.get(app.selected_index) {
                match torrent_search::add_torrent(&selected.magnet_link).await {
                    Ok(gid) => {
                        app.status_message =
                            format!("Added torrent: {} (GID: {})", selected.name, gid);
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

        // Update loading animation and render UI
        let current_tick_rate = if app.search_in_progress {
            loading_tick_rate
        } else {
            tick_rate
        };
        if last_tick.elapsed() >= current_tick_rate {
            app.update_loading_animation();
            tui::render_ui(&mut terminal, &app)?;
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;

    // Clean up aria2 process
    aria2_manager.stop();

    Ok(())
}
