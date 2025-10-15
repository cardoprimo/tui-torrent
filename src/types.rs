#[derive(Debug, Clone)]
pub struct Download {
    pub id: String,
    pub name: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub download_dir: String,
}

#[derive(Debug, Clone)]
pub enum DownloadStatus {
    Active,
    Paused,
    Completed,
    Error(String),
}
