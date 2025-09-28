use std::path::PathBuf;

/// Get the default download directory for the current OS
pub fn get_default_download_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            return home.join("Downloads");
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        // Try XDG user dirs first
        if let Some(download_dir) = dirs::download_dir() {
            return download_dir;
        }
        // Fallback to ~/Downloads
        if let Some(home) = dirs::home_dir() {
            return home.join("Downloads");
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        if let Some(download_dir) = dirs::download_dir() {
            return download_dir;
        }
    }
    
    // Ultimate fallback - current directory
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Ensure the download directory exists
pub fn ensure_download_dir_exists(path: &PathBuf) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Format bytes into human-readable format (B, KB, MB, GB, TB)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{}", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format download speed in human-readable format
pub fn format_speed(speed: &str) -> String {
    if let Ok(bytes_per_sec) = speed.parse::<u64>() {
        if bytes_per_sec == 0 {
            "0B/s".to_string()
        } else {
            let formatted = format_bytes(bytes_per_sec);
            if formatted.contains(' ') {
                format!("{}/s", formatted)
            } else {
                format!("{}B/s", formatted)
            }
        }
    } else {
        format!("{}B/s", speed)
    }
}