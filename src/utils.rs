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