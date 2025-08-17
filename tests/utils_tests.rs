use std::path::PathBuf;
use tempfile::tempdir;
use tui_torrent::utils::ensure_download_dir_exists;

#[test]
fn ensure_download_dir_creates_path() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested_downloads");
    assert!(!nested.exists());
    ensure_download_dir_exists(&PathBuf::from(&nested)).expect("create");
    assert!(nested.exists());
}
