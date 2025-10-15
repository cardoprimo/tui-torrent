use tui_torrent::app::{App, AppMode};
use tui_torrent::torrent_search::TorrentSearchResult;

fn sample_results() -> Vec<TorrentSearchResult> {
    vec![TorrentSearchResult {
        name: "Test Torrent".into(),
        size: "1 GB".into(),
        seeders: 100,
        leechers: 10,
        magnet_link: "magnet:?xt=urn:btih:TEST".into(),
        source: "YTS".into(),
    }]
}

#[test]
fn start_search_sets_state() {
    let mut app = App::new();
    app.search_query = "ubuntu".into();
    app.start_search();
    assert_eq!(app.mode, AppMode::Searching);
    assert!(app.search_in_progress);
    assert!(app.search_results.is_empty());
    assert_eq!(app.loading_frame, 0);
}

#[test]
fn finish_search_moves_to_results() {
    let mut app = App::new();
    app.start_search();
    app.finish_search(sample_results());
    assert_eq!(app.mode, AppMode::Results);
    assert!(!app.search_in_progress);
    assert_eq!(app.search_results.len(), 1);
    assert!(app.status_message.contains("Found 1"));
}

#[test]
fn search_error_returns_to_normal() {
    let mut app = App::new();
    app.start_search();
    app.search_error("boom".into());
    assert_eq!(app.mode, AppMode::Normal);
    assert!(!app.search_in_progress);
    assert!(app.status_message.contains("boom"));
}

#[test]
fn loading_animation_cycles() {
    let mut app = App::new();
    app.start_search();
    let initial = app.loading_frame;
    for _ in 0..20 { app.update_loading_animation(); }
    assert!(app.loading_frame < 8); // stays within frame count
    assert_ne!(initial, app.loading_frame); // progressed
    assert!(!app.search_progress.is_empty());
}

#[test]
fn add_to_search_history() {
    let mut app = App::new();
    app.add_to_search_history("ubuntu".to_string());
    assert_eq!(app.search_history, vec!["ubuntu"]);
    app.add_to_search_history("debian".to_string());
    assert_eq!(app.search_history, vec!["ubuntu", "debian"]);
    app.add_to_search_history("ubuntu".to_string()); // duplicate
    assert_eq!(app.search_history, vec!["ubuntu", "debian"]); // no duplicate
}

#[test]
fn filter_search_history() {
    let mut app = App::new();
    app.search_history = vec!["ubuntu".to_string(), "debian".to_string(), "fedora".to_string()];
    app.search_query = "deb".to_string();
    app.filter_recents();
    assert_eq!(app.filtered_recents, vec!["debian"]);
    app.search_query = "ora".to_string();
    app.filter_recents();
    assert_eq!(app.filtered_recents, vec!["fedora"]);
    app.search_query = "".to_string();
    app.filter_recents();
    assert_eq!(app.filtered_recents, vec!["ubuntu", "debian", "fedora"]);
}


