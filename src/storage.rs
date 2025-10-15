use crate::error::TorrentError;
use crate::types::{Download, DownloadStatus};
use rusqlite::{Connection, Result as SqliteResult};
use std::path::Path;

pub trait HistoryStorage: std::fmt::Debug {
    fn save_search(&self, term: &str) -> Result<(), TorrentError>;
    fn load_searches(&self) -> Result<Vec<String>, TorrentError>;
    fn save_download(&self, download: &Download) -> Result<(), TorrentError>;
    fn load_downloads(&self) -> Result<Vec<Download>, TorrentError>;
    fn update_download_status(&self, id: &str, status: DownloadStatus) -> Result<(), TorrentError>;
}

pub struct SqliteStorage {
    conn: Connection,
}

impl std::fmt::Debug for SqliteStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteStorage").finish()
    }
}

impl SqliteStorage {
    pub fn new(db_path: &Path) -> Result<Self, TorrentError> {
        let conn = Connection::open(db_path)?;
        Self::init_tables(&conn)?;
        Ok(Self { conn })
    }

    fn init_tables(conn: &Connection) -> SqliteResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS search_history (
                id INTEGER PRIMARY KEY,
                term TEXT UNIQUE NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL NOT NULL,
                download_dir TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        Ok(())
    }
}

impl HistoryStorage for SqliteStorage {
    fn save_search(&self, term: &str) -> Result<(), TorrentError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO search_history (term) VALUES (?1)",
            rusqlite::params![term],
        )?;
        Ok(())
    }

    fn load_searches(&self) -> Result<Vec<String>, TorrentError> {
        let mut stmt = self
            .conn
            .prepare("SELECT term FROM search_history ORDER BY timestamp DESC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        let mut searches = Vec::new();
        for term_result in rows {
            searches.push(term_result?);
        }
        Ok(searches)
    }

    fn save_download(&self, download: &Download) -> Result<(), TorrentError> {
        let status_str = match &download.status {
            DownloadStatus::Active => "active",
            DownloadStatus::Paused => "paused",
            DownloadStatus::Completed => "completed",
            DownloadStatus::Error(_) => "error",
        };
        self.conn.execute(
            "INSERT OR REPLACE INTO downloads (id, name, status, progress, download_dir) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![download.id, download.name, status_str, download.progress, download.download_dir],
        )?;
        Ok(())
    }

    fn load_downloads(&self) -> Result<Vec<Download>, TorrentError> {
        let mut stmt = self.conn.prepare("SELECT id, name, status, progress, download_dir FROM downloads ORDER BY timestamp DESC")?;
        let rows = stmt.query_map([], |row| {
            let status_str: String = row.get(2)?;
            let progress: f64 = row.get(3)?;
            let status = match status_str.as_str() {
                "active" => DownloadStatus::Active,
                "paused" => DownloadStatus::Paused,
                "completed" => DownloadStatus::Completed,
                "error" => DownloadStatus::Error("Unknown error".to_string()), // TODO: store error message
                _ => DownloadStatus::Error("Invalid status".to_string()),
            };
            Ok(Download {
                id: row.get(0)?,
                name: row.get(1)?,
                status,
                progress,
                download_dir: row.get(4)?,
            })
        })?;
        let mut downloads = Vec::new();
        for download_result in rows {
            downloads.push(download_result?);
        }
        Ok(downloads)
    }

    fn update_download_status(&self, id: &str, status: DownloadStatus) -> Result<(), TorrentError> {
        let status_str = match &status {
            DownloadStatus::Active => "active",
            DownloadStatus::Paused => "paused",
            DownloadStatus::Completed => "completed",
            DownloadStatus::Error(_) => "error",
        };
        self.conn.execute(
            "UPDATE downloads SET status = ?1 WHERE id = ?2",
            rusqlite::params![status_str, id],
        )?;
        Ok(())
    }
}
