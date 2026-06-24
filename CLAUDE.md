# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# CLI-only build (default, no GUI dependency)
cargo build

# With GUI (Wayland via egui/eframe)
cargo build --features gui
cargo run --features gui

# Release build (opt-level=2, LTO, stripped)
cargo build --release --features gui

# Run tests (uses dev-dependency: tempfile)
cargo test

# Lint
cargo clippy
cargo clippy --features gui

# Install to ~/.cargo/bin/
cargo install --path . --features gui
```

**Important:** The default feature set is empty — `cargo build` without `--features gui` produces a CLI-only binary. Always pass `--features gui` when working on or testing GUI code.

## Architecture

Starcatch is a Wayland-native idea/todo/log capture tool. Rust 2024 edition, single crate, ~650 lines of source.

### Feature Gating

| Feature | Crate | Purpose |
|---------|-------|---------|
| _(default)_ | `clap`, `rusqlite`, `serde`, `chrono`, `uuid` | CLI + database |
| `gui` | `eframe` (Wayland) | Floating panel GUI |

`src/gui.rs` is **entirely** `#[cfg(feature = "gui")]` gated — the module is conditionally compiled, and `main.rs` uses `#[cfg]` blocks at the call site to branch between GUI launch and a help message.

### Entry Point & Dispatch (`src/main.rs`)

`main()` parses CLI args via `clap::Parser`, then dispatches to one of four handler functions:

```
Args::parse() → match args.command
  ├── TodoCommands  → handle_todo()
  ├── IdeaCommands  → handle_idea()
  ├── LogCommands   → handle_log()
  ├── PipeArgs      → handle_pipe()     (reads stdin)
  └── None          → launch_gui() or print help
```

Database path: `~/.local/share/starcatch/starcatch.db` (created on first use). Override with `-D <path>`.

### Data Models (`src/models/`)

Three entity types, each with a UUID v4 primary key and JSON-serialized `Vec<String>` tags stored in SQLite text columns:

- **Todo** — title, priority (P0/P1/P2/P3), status (Pending→Done↔Archived), due date, project, tags
- **Idea** — title, optional content/source/context_window, tags
- **Log** — content, optional mood (stored as string), tags

`Priority`, `TodoStatus` enums implement `Display` (for DB serialization) and row mapping uses `match` for deserialization. Tags use `serde_json` for round-trip through the `TEXT` column.

### Database Layer (`src/db.rs`)

- `open(path)` — opens connection, enables WAL + foreign keys
- `migrate(conn)` — idempotent schema creation (3 tables + indexes on status, priority, created_at)
- CRUD per entity: `insert_*`, `list_*`, `update_todo_status` (no update/delete for idea/log beyond the GUI's direct SQL)

Query pattern: `list_*` functions take an optional filter parameter, build the SQL string dynamically (not via a query builder), and return `Result<Vec<T>>`. Row mapping uses private `*_from_row` functions.

### Pipe Mode

Reads entire stdin to a string, then creates the appropriate entity with defaults (P2 priority for todos, no tags/mood) and inserts into the database. Route by the required positional `type` argument: `todo`, `idea`, or `log`.

### GUI (`src/gui.rs`) — feature-gated

egui/eframe app with a three-tab floating panel (Todo/Idea/Log). Key patterns:

- **Quick input bar** at the bottom — type text, select kind (Todo/Idea/Log), press Enter or click `➕` to capture
- **Enter key capture** is handled at the **top level** of `update()` (not inside any panel) for IME compatibility
- **Escape** closes the window via `ViewportCommand::Close`
- **Data refresh** uses a `needs_refresh` flag — set after mutations, checked at top of frame
- **Toast notifications** display for 2.5 seconds after actions
- **CJK font**: loads `/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc` at startup, prepends to Proportional and Monospace font families

### Release Profile

```toml
[profile.release]
opt-level = 2   # balanced (not 3/s)
strip = true    # strip symbols
lto = true      # link-time optimization
```

## Code Patterns

- **No ORM** — raw SQL via `rusqlite`, with parameterized queries. Tags stored as JSON strings.
- **Immutable DB path** — built once in `default_db_path()`, never mutated.
- **Error handling**: `main()` prints errors to stderr and exits with code 1. Handler functions propagate `rusqlite::Result`.
- **CLI args** defined in `src/cli.rs` using clap derive macros (`#[derive(Parser)]`, `#[derive(Subcommand)]`).
- **GUI state** lives entirely in `GuiApp` struct fields — no separate state management.
