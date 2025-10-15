use tempfile::NamedTempFile;
use tui_torrent::storage::{HistoryStorage, SqliteStorage};
use tui_torrent::types::{Download, DownloadStatus};

#[test]
fn test_save_and_load_searches() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage = SqliteStorage::new(temp_file.path()).unwrap();

    // Save some searches
    storage.save_search("ubuntu").unwrap();
    storage.save_search("debian").unwrap();
    storage.save_search("ubuntu").unwrap(); // duplicate, should be ignored

    // Load searches
    let searches = storage.load_searches().unwrap();
    assert_eq!(searches.len(), 2);
    assert!(searches.contains(&"ubuntu".to_string()));
    assert!(searches.contains(&"debian".to_string()));
}

#[test]
fn test_save_and_load_downloads() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage = SqliteStorage::new(temp_file.path()).unwrap();

    let download = Download {
        id: "gid123".to_string(),
        name: "Ubuntu ISO".to_string(),
        status: DownloadStatus::Active,
        progress: 50.0,
        download_dir: "/tmp".to_string(),
    };

    // Save download
    storage.save_download(&download).unwrap();

    // Load downloads
    let downloads = storage.load_downloads().unwrap();
    assert_eq!(downloads.len(), 1);
    let loaded = &downloads[0];
    assert_eq!(loaded.id, "gid123");
    assert_eq!(loaded.name, "Ubuntu ISO");
    assert!(matches!(loaded.status, DownloadStatus::Active));
    assert_eq!(loaded.progress, 50.0);
    assert_eq!(loaded.download_dir, "/tmp");
}

#[test]
fn test_update_download_status() {
    let temp_file = NamedTempFile::new().unwrap();
    let storage = SqliteStorage::new(temp_file.path()).unwrap();

    let download = Download {
        id: "gid123".to_string(),
        name: "Ubuntu ISO".to_string(),
        status: DownloadStatus::Active,
        progress: 50.0,
        download_dir: "/tmp".to_string(),
    };

    storage.save_download(&download).unwrap();

    // Update status
    storage
        .update_download_status("gid123", DownloadStatus::Completed)
        .unwrap();

    // Load and check
    let downloads = storage.load_downloads().unwrap();
    assert_eq!(downloads.len(), 1);
    assert!(matches!(downloads[0].status, DownloadStatus::Completed));
}
