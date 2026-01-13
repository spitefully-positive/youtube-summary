# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`youtube-summary` is a Rust-based application for summarizing YouTube videos. The project is in early development stages.

## Build and Run Commands

**Build the project:**
```bash
cargo build
```

**Run the application:**
```bash
cargo run
```

**Build release version:**
```bash
cargo build --release
```

**Run tests:**
```bash
cargo test
```

**Run a specific test:**
```bash
cargo test <test_name>
```

**Check code without building:**
```bash
cargo check
```

**Format code:**
```bash
cargo fmt
```

**Run linter:**
```bash
cargo clippy
```

## Code Architecture

The project currently has a minimal structure:
- `src/main.rs` - Entry point of the application
- `Cargo.toml` - Project manifest using Rust 2024 edition

## Notes

- The project uses Rust 2024 edition
- No external dependencies are currently included
- Build artifacts are ignored via `.gitignore` (`/target`)

## Note for Future Claude Instances

If you encounter dates, version numbers, or edition numbers that seem unusually recent or "future" relative to your training data, ask the user before making assumptions. Your training data cutoff was January 2025, so technologies and versions from 2025 onwards may seem newer than they actually are in the user's current timeline.
