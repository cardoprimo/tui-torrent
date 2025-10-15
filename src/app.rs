use crate::aria2_client::TorrentStatus;
use crate::storage::{HistoryStorage, SqliteStorage};
use crate::torrent_search::TorrentSearchResult;
use crate::utils::get_db_path;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::io;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Search,
    Results,
    Searching,
}

pub struct App {
    pub mode: AppMode,
    pub search_query: String,
    pub search_results: Vec<TorrentSearchResult>,
    pub active_downloads: Vec<TorrentStatus>,
    pub selected_index: usize,
    pub search_history: Vec<String>,
    pub recents_index: usize,
    pub recents_offset: usize,
    pub filtered_recents: Vec<String>,
    pub should_quit: bool,
    pub search_in_progress: bool,
    pub status_message: String,
    pub selected_category: Option<String>,
    pub download_requested: bool,
    pub loading_frame: usize,
    pub search_progress: String,
    storage: Box<dyn HistoryStorage>,
}

impl App {
    pub fn new() -> Self {
        let storage = Box::new(SqliteStorage::new(&get_db_path()).unwrap_or_else(|e| {
            eprintln!("Failed to initialize storage: {}", e);
            std::process::exit(1);
        }));
        let search_history = storage.load_searches().unwrap_or_else(|e| {
            eprintln!("Failed to load search history: {}", e);
            Vec::new()
        });
        let filtered_recents = search_history.clone();

        App {
            mode: AppMode::Normal,
            search_query: String::new(),
            search_results: Vec::new(),
            active_downloads: Vec::new(),
            selected_index: 0,
            search_history,
            recents_index: 0,
            recents_offset: 0,
            filtered_recents,
            should_quit: false,
            search_in_progress: false,
            status_message: "Starting up...".to_string(),
            selected_category: None,
            download_requested: false,
            loading_frame: 0,
            search_progress: String::new(),
            storage,
        }
    }

    pub fn start_search(&mut self) {
        self.mode = AppMode::Searching;
        self.search_in_progress = true;
        self.status_message = "Initializing search across multiple sources...".to_string();
        self.search_progress = "Starting search...".to_string();
        self.search_results.clear();
        self.selected_index = 0;
        self.loading_frame = 0;
    }

    pub fn finish_search(&mut self, results: Vec<TorrentSearchResult>) {
        self.search_in_progress = false;
        self.search_results = results;
        self.mode = AppMode::Results;
        self.status_message = format!("Found {} results", self.search_results.len());
        self.selected_index = 0;
        // Add the search term to history
        if !self.search_query.is_empty() {
            self.add_to_search_history(self.search_query.clone());
        }
    }

    pub fn search_error(&mut self, error: String) {
        self.search_in_progress = false;
        self.mode = AppMode::Normal;
        self.status_message = format!("Search failed: {}", error);
    }

    pub fn update_loading_animation(&mut self) {
        if self.search_in_progress {
            self.loading_frame = (self.loading_frame + 1) % 8;

            // Update search progress message with different states
            let progress_messages = [
                "Connecting to YTS movie database...",
                "Searching YTS for movies...",
                "Connecting to PirateBay API...",
                "Searching PirateBay torrents...",
                "Checking 1337x mirrors...",
                "Searching 1337x database...",
                "Sorting results by seeders...",
                "Finalizing search results...",
            ];

            let message_index = (self.loading_frame / 4) % progress_messages.len();
            self.search_progress = progress_messages[message_index].to_string();
        }
    }

    pub fn get_loading_indicator(&self) -> &'static str {
        const LOADING_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
        LOADING_FRAMES[self.loading_frame]
    }

    pub fn handle_input(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match self.mode {
                    AppMode::Normal => self.handle_normal_mode(key),
                    AppMode::Search => self.handle_search_mode(key),
                    AppMode::Results => self.handle_results_mode(key),
                    AppMode::Searching => self.handle_searching_mode(key),
                }
            }
        }
        Ok(())
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('s') => {
                self.mode = AppMode::Search;
                self.filter_recents();
                self.recents_index = 0;
                self.recents_offset = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.active_downloads.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.active_downloads.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.active_downloads.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.active_downloads.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            _ => {}
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_query.clear();
                self.filtered_recents.clear();
            }
            KeyCode::Enter => {
                if self.recents_index < self.filtered_recents.len() {
                    // Select from recents
                    self.search_query = self.filtered_recents[self.recents_index].clone();
                }
                if !self.search_query.is_empty() {
                    self.start_search();
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.filter_recents();
            }
            KeyCode::Up => {
                if self.recents_index > 0 {
                    self.recents_index -= 1;
                    if self.recents_index < self.recents_offset {
                        self.recents_offset = self.recents_index;
                    }
                }
            }
            KeyCode::Down => {
                if self.recents_index < self.filtered_recents.len() - 1 {
                    self.recents_index += 1;
                    if self.recents_index >= self.recents_offset + 5 {
                        self.recents_offset = self.recents_index - 4;
                    }
                }
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.filter_recents();
            }
            _ => {}
        }
    }

    fn handle_results_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_results.clear();
                self.selected_index = 0;
            }
            KeyCode::Enter => {
                if !self.search_results.is_empty() {
                    self.download_requested = true;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.search_results.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.search_results.len();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.search_results.is_empty() {
                    self.selected_index = if self.selected_index == 0 {
                        self.search_results.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }
            _ => {}
        }
    }

    fn handle_searching_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_in_progress = false;
                self.mode = AppMode::Normal;
                self.status_message = "Search cancelled".to_string();
            }
            _ => {
                // Ignore other keys while searching
            }
        }
    }

    pub fn add_to_search_history(&mut self, term: String) {
        if !self.search_history.contains(&term) {
            self.search_history.push(term.clone());
            if let Err(e) = self.storage.save_search(&term) {
                eprintln!("Failed to save search term: {}", e);
            }
        }
    }

    pub fn filter_recents(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_recents = self.search_history.clone();
        } else {
            self.filtered_recents = self
                .search_history
                .iter()
                .filter(|term| term.contains(&self.search_query))
                .cloned()
                .collect();
        }
        // Reset index and offset if out of bounds
        if self.recents_index >= self.filtered_recents.len() {
            self.recents_index = self.filtered_recents.len().saturating_sub(1);
        }
        if self.recents_offset > self.recents_index {
            self.recents_offset = self.recents_index.saturating_sub(4);
        }
    }
}
