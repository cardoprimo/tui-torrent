# Agent Guidelines for tui-torrent

## Commands
- **Build**: `cargo build`
- **Test all**: `cargo test`
- **Test single**: `cargo test <test_name>`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`
- **Run**: `cargo run`

## Code Style
- **Edition**: Rust 2024
- **Naming**: snake_case for functions/variables, PascalCase for types/enums
- **Imports**: std → external crates → local modules
- **Error handling**: Use `TorrentError` enum with `Result<T>` alias
- **Async**: Use tokio runtime with async/await
- **Derives**: Include `Debug` on structs, `Display`/`Error` on error types
- **Formatting**: Run `cargo fmt` before commits
- **Linting**: Fix all `cargo clippy` warnings
- **Testing**: Write unit tests for new functionality, run tests before commits