# AGENTS.md

Guidelines for AI coding agents working in this repository.

## Project Overview

`youtube-summary` is a Rust CLI application that fetches YouTube video transcripts and summarizes them using the OpenRouter API. OpenRouter provides access to 300+ AI models including Claude, GPT-4, Llama, and more through a unified API. Uses Rust 2024 edition with async/await via tokio.

## Build Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build

# Check without building (fast)
cargo check
cargo check --all              # Check all targets
```

## Test Commands

```bash
# Run all tests
cargo test
cargo test --all               # All workspace tests

# Run a single test by name
cargo test <test_name>

# Run tests matching a pattern
cargo test extract_video       # Runs tests containing "extract_video"

# Run tests in a specific module
cargo test transcript::        # Tests in transcript module

# Show output from passing tests
cargo test -- --nocapture
```

## Lint and Format Commands

```bash
# Linting - WARNINGS ARE ERRORS (enforced by pre-commit hook)
cargo clippy --all -- -D warnings

# Format check (doesn't modify files)
cargo fmt --all -- --check

# Auto-format (modifies files)
cargo fmt
```

## Pre-commit Hooks

Git hooks (via cargo-husky) run on every commit and push:
1. `cargo test --all`
2. `cargo check --all`
3. `cargo clippy --all -- -D warnings`
4. `cargo fmt --all -- --check`

**All checks must pass before committing.** Fix clippy warnings before committing.

## Project Structure

```
src/
├── main.rs        # Entry point, async runtime setup, orchestration
├── cli.rs         # Argument parsing, Args struct
├── config.rs      # Configuration loading (env, files, precedence)
├── error.rs       # Custom Error enum and Result type alias
├── transcript.rs  # YouTube transcript fetching, URL parsing
└── openrouter.rs  # OpenRouter API client, request/response types, model listing
```

## Code Style Guidelines

### Imports

Order imports in groups separated by blank lines:
1. Standard library (`std::`)
2. External crates
3. Local modules (`crate::`)

```rust
use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::Args;
use crate::error::{Error, Result};
```

### Error Handling

Use the custom error types defined in `src/error.rs`:

```rust
use crate::error::{Error, Result};

// Use map_err to convert external errors
let content = fs::read_to_string(&path)
    .map_err(|e| Error::Config(format!("Failed to read: {}", e)))?;

// Return descriptive error variants
Err(Error::InvalidYoutubeUrl(format!(
    "Could not extract video ID from: {}", url
)))
```

Error variants:
- `InvalidYoutubeUrl(String)` - Bad YouTube URL format
- `TranscriptFetch(String)` - Failed to get transcript
- `ApiRequest(String)` - OpenRouter API errors
- `Config(String)` - Configuration/credential issues

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Functions | snake_case | `fetch_transcript`, `extract_video_id` |
| Variables | snake_case | `video_id`, `api_key` |
| Types/Structs | PascalCase | `Config`, `Args`, `Choice` |
| Enums | PascalCase | `Error` |
| Enum variants | PascalCase | `Error::ApiRequest` |
| Constants | SCREAMING_SNAKE_CASE | `API_URL`, `DEFAULT_MODEL` |
| Modules | snake_case | `transcript`, `openrouter` |

### Struct Definitions

Always derive `Debug`. Add other derives as needed:

```rust
#[derive(Debug)]                              // Minimum for all structs
#[derive(Debug, Clone, Copy, PartialEq)]      // For simple enums
#[derive(Debug, Default)]                     // For config with defaults
#[derive(Serialize)]                          // For API request bodies
#[derive(Deserialize)]                        // For API responses
```

### Async Code

Use `tokio` runtime with `#[tokio::main]`:

```rust
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Pattern Matching

Use `if let` with `let` chains (Rust 2024 feature):

```rust
if url.contains("youtube.com/watch")
    && let Some(v_pos) = url.find("v=")
{
    // Extract video ID
}
```

### Function Design

- Keep functions focused on a single task
- Use `Result<T>` for fallible operations
- Prefer returning `Result` over panicking
- Use `Option` for values that may not exist

### Documentation Comments

Use `///` for public items:

```rust
/// Summarize a transcript using OpenRouter API
/// Sends the transcript with the configured prompt to the selected model
pub async fn summarize(config: &Config, transcript: &str) -> Result<String> {
```

## Configuration Precedence

API key lookup order (first found wins):
1. `--api-key` CLI argument
2. `OPENROUTER_API_KEY` environment variable
3. `~/.config/youtube-summary/credentials` file (`OPENROUTER_API_KEY=...`)
4. Config file (`api_key=` field)

Model selection order (first found wins):
1. `--model` CLI argument
2. Config file (`default_model=` field)
3. Default: `anthropic/claude-haiku-4.5`

## Dependencies

Key crates used:
- `tokio` - Async runtime
- `reqwest` - HTTP client with JSON support
- `serde` / `serde_json` - Serialization
- `yt-transcript-rs` - YouTube transcript fetching

## Common Tasks

### Adding a new CLI flag

1. Add field to `Args` struct in `src/cli.rs`
2. Add parsing logic in `Args::parse()`
3. Update help text in `Args::usage()`
4. Use the flag in `main.rs` or pass through `Config`

### Adding a new error type

1. Add variant to `Error` enum in `src/error.rs`
2. Add match arm in `Display` impl with human-readable message

### Adding a new API field

1. Add field to request/response struct in `src/openrouter.rs`
2. Use `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields

### Model Selection

Models are specified using OpenRouter's model ID format: `provider/model-name`

Examples:
- `anthropic/claude-haiku-4.5` (default)
- `anthropic/claude-sonnet-4`
- `openai/gpt-4o`
- `openai/gpt-4o-mini`
- `meta-llama/llama-3.1-70b-instruct`

Use `--list-models` to see all available models, optionally filtered:
```bash
youtube-summary --list-models           # List all models
youtube-summary --list-models claude    # Filter by "claude"
youtube-summary -l gpt                  # Filter by "gpt"
```
