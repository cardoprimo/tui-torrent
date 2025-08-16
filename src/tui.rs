use crate::app::{App, AppMode};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
use std::io::{Result, stdout};

pub fn render_ui(app: &App) -> Result<()> {
    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main content
                Constraint::Length(3), // Status bar
            ].as_ref())
            .split(f.size());

        // Render header/search bar
        match app.mode {
            AppMode::Normal => {
                let instructions = Paragraph::new(Line::from(vec![
                    Span::raw("Press "),
                    Span::styled("s", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
                    Span::raw(" to search, "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                    Span::raw(" to quit"),
                ]))
                .block(Block::default().title("🏴‍☠️ TUI Torrent").borders(Borders::ALL));
                f.render_widget(instructions, chunks[0]);
            },
            AppMode::Search => {
                let search_text = format!("Search: {}", app.search_query);
                let search_bar = Paragraph::new(search_text)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().title("🔍 Enter Search Query (Press Enter to search, Esc to cancel)").borders(Borders::ALL));
                f.render_widget(search_bar, chunks[0]);
            },
            AppMode::Searching => {
                let searching_text = format!("Searching for: {}", app.search_query);
                let search_bar = Paragraph::new(searching_text)
                    .style(Style::default().fg(Color::Blue))
                    .block(Block::default().title("🔄 Searching Multiple Sources...").borders(Borders::ALL));
                f.render_widget(search_bar, chunks[0]);
            },
            AppMode::Results => {
                let result_info = Paragraph::new(Line::from(vec![
                    Span::raw("Press "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD).fg(Color::Green)),
                    Span::raw(" to download, "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD).fg(Color::Red)),
                    Span::raw(" to go back, "),
                    Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
                    Span::raw(" to navigate"),
                ]))
                .block(Block::default().title(format!("📋 Search Results ({})", app.search_results.len())).borders(Borders::ALL));
                f.render_widget(result_info, chunks[0]);
            }
        }

        // Status bar
        let status_color = if app.search_in_progress {
            Color::Blue
        } else if app.status_message.contains("failed") || app.status_message.contains("error") {
            Color::Red
        } else {
            Color::Green
        };

        let status_bar = Paragraph::new(app.status_message.clone())
            .style(Style::default().fg(status_color))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status_bar, chunks[2]);

        // Render main content based on app mode
        match app.mode {
            AppMode::Normal | AppMode::Search => {
                if app.active_downloads.is_empty() {
                    let empty_msg = Paragraph::new("No active downloads. Press 's' to search for torrents.")
                        .style(Style::default().fg(Color::Gray))
                        .alignment(Alignment::Center)
                        .block(Block::default().title("📥 Active Downloads").borders(Borders::ALL));
                    f.render_widget(empty_msg, chunks[1]);
                } else {
                    let items: Vec<ListItem> = app.active_downloads
                        .iter()
                        .enumerate()
                        .map(|(i, t)| {
                            let progress = if let (Ok(completed), Ok(total)) = (t.completedLength.parse::<u64>(), t.totalLength.parse::<u64>()) {
                                if total > 0 {
                                    format!(" ({:.1}%)", (completed as f64 / total as f64) * 100.0)
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            let title = format!(
                                "📁 {} - {}/{} bytes{} @ {} B/s",
                                t.status, t.completedLength, t.totalLength, progress, t.downloadSpeed
                            );
                            let style = if i == app.selected_index && app.mode == AppMode::Normal {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                            };
                            ListItem::new(title).style(style)
                        })
                        .collect();

                    let downloads = List::new(items).block(
                        Block::default()
                            .title("📥 Active Downloads")
                            .borders(Borders::ALL),
                    );
                    f.render_widget(downloads, chunks[1]);
                }
            },
            AppMode::Searching => {
                // Show loading animation
                let loading_msg = Paragraph::new("🔄 Searching YTS, PirateBay, and 1337x...\n\nThis may take a few seconds...")
                    .style(Style::default().fg(Color::Blue))
                    .alignment(Alignment::Center)
                    .block(Block::default().title("🔍 Searching").borders(Borders::ALL));
                f.render_widget(loading_msg, chunks[1]);
            },
            AppMode::Results => {
                if app.search_results.is_empty() {
                    let empty_msg = Paragraph::new("No results found. Try a different search term.")
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center)
                        .block(Block::default().title("📋 Search Results").borders(Borders::ALL));
                    f.render_widget(empty_msg, chunks[1]);
                } else {
                    let items: Vec<ListItem> = app.search_results
                        .iter()
                        .enumerate()
                        .map(|(i, result)| {
                            // Color code by source
                            let source_color = match result.source.as_str() {
                                "YTS" => Color::Green,
                                "PirateBay" => Color::Blue,
                                "1337x" => Color::Magenta,
                                _ => Color::Gray,
                            };

                            // Truncate long names
                            let display_name = if result.name.len() > 60 {
                                format!("{}...", &result.name[..57])
                            } else {
                                result.name.clone()
                            };

                            let title = Line::from(vec![
                                Span::styled(format!("[{}] ", result.source), Style::default().fg(source_color).add_modifier(Modifier::BOLD)),
                                Span::raw(display_name),
                                Span::styled(format!(" | {} | ", result.size), Style::default().fg(Color::Cyan)),
                                Span::styled(format!("S:{}", result.seeders), Style::default().fg(Color::Green)),
                                Span::raw(" "),
                                Span::styled(format!("L:{}", result.leechers), Style::default().fg(Color::Red)),
                            ]);

                            let style = if i == app.selected_index {
                                Style::default().bg(Color::DarkGray)
                            } else {
                                Style::default()
                            };
                            ListItem::new(title).style(style)
                        })
                        .collect();

                    let results_list = List::new(items).block(
                        Block::default()
                            .title("📋 Torrent Results")
                            .borders(Borders::ALL),
                    );
                    f.render_widget(results_list, chunks[1]);
                }
            }
        }
    })?;

    Ok(())
}
