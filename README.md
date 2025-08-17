# TUI Torrent

A Terminal User Interface (TUI) application for searching and downloading torrents using multiple sources.

## Features

- 🔍 **Multi-source search**: Searches YTS, PirateBay, and 1337x simultaneously
- 🏴‍☠️ **Real torrent results**: Gets actual magnet links and torrent information
- 📊 **Rich TUI interface**: Beautiful terminal interface with colors and navigation
- ⚡ **Fast and responsive**: Concurrent searches with timeout handling
- 🔄 **Aria2 integration**: Downloads torrents using aria2 RPC

## Supported Sources

- **YTS**: High-quality movie torrents
- **PirateBay**: General torrent search via API
- **1337x**: Popular torrent site (with fallback mock data due to anti-bot protection)

## Usage

### Running the TUI

```bash
cargo run
```

### Controls

- **s**: Start a new search
- **Enter**:
  - In search mode: Execute search
  - In results mode: Download selected torrent
- **↑/↓** or **j/k**: Navigate through results or downloads (vim-style)
- **Esc**: Go back/cancel current action
- **q**: Quit application

### Testing Search Integration

```bash
cargo run --bin test_search
```

## Requirements

- Rust 1.70+
- aria2 running on localhost:6800 (for downloads)

## Installation

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run: `cargo run`

## Architecture

- `src/main.rs`: Main application loop
- `src/app.rs`: Application state management
- `src/tui.rs`: Terminal UI rendering
- `src/api/`: Torrent source integrations
  - `yts.rs`: YTS movie API client
  - `piratebay.rs`: PirateBay API client
  - `x1337.rs`: 1337x scraper with fallbacks
- `src/torrent_search.rs`: Multi-source search engine
- `src/aria2_client.rs`: Aria2 RPC client for downloads

## Demo

1. Start the application: `cargo run`
2. Press `s` to search
3. Type a search term (e.g., "ubuntu" or "inception")
4. Press Enter to search multiple sources
5. Use ↑/↓ to navigate results
6. Press Enter to download a torrent
7. Press `q` to quit

The application will show:

- Source attribution for each result
- Seeder/leecher counts
- File sizes
- Real-time search progress
- Download status
