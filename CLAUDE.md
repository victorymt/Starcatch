# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Rust CLI build
cargo build
cargo run

# Run tests (uses dev-dependency: tempfile)
cargo test

# Lint
cargo clippy

# Install CLI to ~/.cargo/bin/
cargo install --path .
```

### Qt GUI (C++)

```bash
cd qt
cmake -B build
cmake --build build
./build/starcatch-qt
```

**Important:** The Rust binary is CLI-only. The GUI is a separate C++ Qt 6 project in `qt/`. Both share the same SQLite database at `~/.local/share/starcatch/starcatch.db`.

## Architecture

Starcatch is a Wayland-native idea/todo/log capture tool. The CLI is written in Rust (~1,200 lines), and the GUI is a separate C++ Qt 6 application (~900 lines in `qt/`). Both share the same SQLite database.

### Project Layout

```
src/                # Rust CLI
├── main.rs         # Entry point + CLI handlers
├── cli.rs          # clap argument definitions
├── db.rs           # SQLite via rusqlite (WAL, migrations, CRUD)
└── models/         # Todo, Idea, Log structs + enums

qt/                 # C++ Qt 6 GUI
├── CMakeLists.txt
└── src/
    ├── main.cpp          # QApplication entry point
    ├── models.h          # C++ mirrors of Rust models
    ├── database.h/.cpp   # SQLite via QSqlDatabase
    ├── inputparser.h/.cpp # Quick input parsing (P0-P3, due:, #tags)
    ├── mainwindow.h/.cpp  # Top-level window + layout
    ├── todopanel.h/.cpp   # Todo tab (filters, checkboxes, actions)
    ├── ideapanel.h/.cpp   # Idea tab (days slider, list)
    ├── logpanel.h/.cpp    # Log tab (days slider, mood icons)
    ├── quickinputbar.h/.cpp # Bottom input bar
    └── toastwidget.h/.cpp  # Toast overlay notifications
```

### No Feature Gating (since Qt migration)

The GUI was migrated from egui to C++ Qt 6 for superior CJK text support. The Rust crate is now CLI-only — no feature flags needed. The Qt GUI is a completely separate project that directly accesses the same SQLite database via Qt's QSqlDatabase.

### Entry Point & Dispatch (`src/main.rs`)

`main()` parses CLI args via `clap::Parser`, then dispatches to one of four handler functions:

```
Args::parse() → match args.command
  ├── TodoCommands  → handle_todo()
  ├── IdeaCommands  → handle_idea()
  ├── LogCommands   → handle_log()
  ├── PipeArgs      → handle_pipe()     (reads stdin)
  └── None          → print help message
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

### GUI (`qt/`) — C++ Qt 6

Qt 6 application with a three-tab floating panel (Todo/Idea/Log). Key patterns:

- **Quick input bar** at the bottom — type text, select kind (Todo/Idea/Log via QComboBox), press Enter or click `➕` to capture
- **Enter key capture** works natively — `QLineEdit::returnPressed()` only fires after IME composition is committed (no manual IME gating needed, unlike egui)
- **Escape** closes the window via QShortcut
- **Data refresh** happens inline after every mutation (insert/update/delete → refresh panel immediately)
- **Toast notifications** display for 2.5 seconds via QTimer::singleShot
- **CJK text**: Qt uses HarfBuzz + fontconfig natively — no special font loading needed
- **Database**: Uses Qt's QSqlDatabase with QSQLITE driver, same schema as the Rust CLI
- **Tags**: Stored as JSON arrays in TEXT columns, identical format to Rust's serde_json output

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
