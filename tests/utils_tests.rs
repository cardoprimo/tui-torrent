use std::path::PathBuf;
use tempfile::tempdir;
use tui_torrent::utils::{ensure_download_dir_exists, format_bytes, format_speed};

#[test]
fn ensure_download_dir_creates_path() {
    let dir = tempdir().expect("tempdir");
    let nested = dir.path().join("nested_downloads");
    assert!(!nested.exists());
    ensure_download_dir_exists(&PathBuf::from(&nested)).expect("create");
    assert!(nested.exists());
}

#[test]
fn test_format_bytes() {
    assert_eq!(format_bytes(0), "0");
    assert_eq!(format_bytes(512), "512");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
    assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    assert_eq!(format_bytes(1024 * 1024 * 1024 * 1024), "1.0 TB");
    assert_eq!(format_bytes(2147483648), "2.0 GB"); // 2GB
}

#[test]
fn test_format_speed() {
    assert_eq!(format_speed("0"), "0B/s");
    assert_eq!(format_speed("512"), "512B/s");
    assert_eq!(format_speed("1024"), "1.0 KB/s");
    assert_eq!(format_speed("1048576"), "1.0 MB/s");
    assert_eq!(format_speed("1073741824"), "1.0 GB/s");
    assert_eq!(format_speed("invalid"), "invalidB/s");
}
