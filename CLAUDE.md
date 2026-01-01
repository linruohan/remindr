# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Remindr is an open-source, self-hostable alternative to Notion built with Rust and GPUI (Zed's GPU-accelerated UI framework).

## Build & Run Commands

```bash
# Run the application
cargo run --bin remindr_gpui

# Build in release mode
cargo build --release --bin remindr_gpui

# Run tests
cargo test

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy lints
cargo clippy
```

## Architecture

The project follows **hexagonal architecture** (ports and adapters) with a single crate in `crates/gpui/`.

### Layer Structure

- **`app/`** - UI layer using GPUI framework
  - `components/` - Reusable UI components (sidebar, node renderers, slash menu)
  - `screens/` - Application screens (home, document, login)
  - `states/` - GPUI global state management (app, document, node, repository, settings states)
  - `remindr.rs` - Core application class handling config/database initialization

- **`domain/`** - Business logic layer
  - `database/` - Domain models (e.g., `DocumentModel`)
  - `entities/` - Domain entities (e.g., settings)
  - `ports.rs` - Repository trait definitions (e.g., `DocumentRepositoryPort`)

- **`infrastructure/`** - Data access layer
  - `repositories/` - SQLite implementations of domain ports
  - `entities/` - Database entities

### Key Patterns

- **Node System**: Documents are composed of typed nodes (`RemindrNode`) with elements (`RemindrElement`) that can be Text, Heading, or Divider. Each node type has a data struct and a renderable component.

- **Navigation**: Uses `gpui-router` and `gpui-nav` for screen navigation via `Navigator` in `AppState`.

- **Database**: SQLite via `sqlx` with migrations in `crates/gpui/migrations/`. Database file stored in user config directory (`~/.config/remindr/database.sqlite`).

- **Settings**: JSON-based settings stored in `~/.config/remindr/settings.json`.

- **UUID Generation**: Uses UUID v7 (time-ordered) via `Utils::generate_uuid()`.

### Dependencies

Key external crates:
- `gpui` (0.2.2) - GPU-accelerated UI framework from Zed
- `gpui-component` - Pre-built GPUI components with theming
- `gpui-router` / `gpui-nav` - Navigation system
- `sqlx` with SQLite - Async database access
- `tokio` - Async runtime
