# CLAUDE.md

## Build & Run

### CLI (Rust)
```bash
cargo build
./target/debug/starcatch [OPTIONS] <COMMAND>
# Release: cargo build --release → ./target/release/starcatch
```

### TUI (Rust, ratatui)
```bash
cargo build -p starcatch-tui
./target/debug/starcatch-tui [db_path]
```

### Qt GUI (C++)
```bash
cd qt
cmake -B build
cmake --build build
./build/starcatch-qt
```

**Database:** All three share `~/.local/share/starcatch/starcatch.db` (SQLite, WAL mode).
- CLI: override with `-D <path>` (global flag, see `src/cli.rs:11`)
- TUI: override via positional `db_path` argument
- Qt GUI: hardcoded `~/.local/share/starcatch/starcatch.db` (no `-D` support)

## Architecture

Starcatch is a Wayland-native idea/todo/log capture tool with three frontends sharing one SQLite database:

- **CLI** (`src/`) — clap-based command-line interface
- **TUI** (`tui/`) — ratatui + crossterm terminal interface
- **Qt GUI** (`qt/`) — C++ Qt 6 application (legacy, still maintained)
- **starcatch-core** (`starcatch-core/`) — shared library (models, db, parser)
- **vendor/sqlite-migrate** (`vendor/sqlite-migrate/`) — vendored schema migration lib

### Project Layout

```
src/                        # Rust CLI (thin handlers)
├── main.rs                 # Entry point + CLI handlers (dispatch to starcatch-core)
└── cli.rs                  # clap argument definitions (Parser/Subcommand derive)

starcatch-core/             # Shared core library
└── src/
    ├── lib.rs              # Re-exports
    ├── db.rs               # SQLite via rusqlite (WAL, migrations, CRUD)
    ├── parser.rs           # Pipe input parsing (P0-P3, due:, #tags, project:, mood:, source:)
    └── models/             # Todo, Idea, Log structs + Priority/TodoStatus enums
        ├── mod.rs
        ├── todo.rs
        ├── idea.rs
        └── log.rs

tui/                        # Ratatui TUI
└── src/
    ├── main.rs             # Entry point (positional db_path arg)
    ├── app.rs              # App state, status messages (2.5s tick-based clear)
    ├── handler.rs          # Key event handling
    ├── ui.rs               # Main layout
    ├── event.rs            # crossterm poll (250ms timeout)
    ├── styles.rs           # Theme colors
    └── components/         # sidebar, todo_list, idea_list, log_list, quick_input

qt/                         # C++ Qt 6 GUI (legacy)
├── CMakeLists.txt
└── src/
    ├── main.cpp            # QApplication entry point
    ├── models.h            # C++ mirrors of Rust models (Todo/Idea/LogEntry + enums)
    ├── database.h/.cpp     # SQLite via QSqlDatabase (same schema, addColumnIfMissing migration)
    ├── inputparser.h/.cpp  # Quick input parsing (P0-P3, due:, #tags, project:)
    ├── mainwindow.h/.cpp   # Top-level window + layout
    ├── todopanel.h/.cpp    # Todo tab (filters, checkboxes, actions)
    ├── ideapanel.h/.cpp    # Idea tab (days slider, list)
    ├── logpanel.h/.cpp     # Log tab (days slider, mood icons)
    ├── allpanel.h/.cpp     # "All" tab (mixed entries)
    ├── quickinputbar.h/.cpp # Bottom input bar
    ├── toastwidget.h/.cpp  # Toast overlay notifications
    ├── theme.h/.cpp        # Qt theme colors
    ├── command_plugin.h/.cpp # Command plugin system
    └── commands/           # Slash commands (export, help, search, stats, test-delete, theme)

vendor/sqlite-migrate/      # Vendored schema migration (CREATE TABLE + ALTER TABLE ADD COLUMN)
```

### Database Layer (`starcatch-core/src/db.rs`)

- `open(path)` — opens connection, enables WAL + foreign keys
- `migrate(conn)` — idempotent schema creation via `sqlite-migrate` (3 tables: todos/ideas/logs, each with `project TEXT` column + indexes on status, priority, created_at)
- **Full CRUD per entity**: `insert_*`, `list_*`, `get_*`, `update_*`, `update_todo_status`, `delete_*`
  - `update_*` and `delete_*` call `get_*` first to error on nonexistent IDs (no silent success)
- Query pattern: `list_*` functions take an optional filter parameter (status / days), build SQL dynamically, return `Result<Vec<T>>`. Row mapping uses private `*_from_row` functions.
- **Timestamps**: stored as RFC3339 (`to_rfc3339()`); `parse_ts()` helper robustly parses RFC3339, Qt ISODate (`Z` suffix), and strptime formats to avoid epoch fallback.
- **Date filtering**: `WHERE datetime(created_at) >= datetime('now', ?1)` — wraps `created_at` in `datetime()` to normalize RFC3339 vs SQLite format comparison.

### Pipe Mode

Reads entire stdin to a string, parses metadata tokens, then inserts with defaults. Route by positional `type` argument: `todo`, `idea`, or `log`.
- Todo tokens: `P0-P3` (priority), `due:YYYY-MM-DD` (date), `#tag`, `project:NAME`
- Idea tokens: `#tag`, `source:NAME`, `project:NAME`
- Log tokens: `#tag`, `mood:NAME`, `project:NAME`
- Fullwidth colon (`：`) supported for all `key:` tokens.

### TUI (`tui/`) — Ratatui + Crossterm

Terminal interface with sidebar navigation, three views (Todo/Idea/Log), quick input, and edit/delete/archive actions.
- **Status messages** display for ~2.5s (10 ticks × 250ms `POLL_TIMEOUT`), matching Qt toast duration
- **Ctrl+T/Ctrl+L** switch input type in command mode; Ctrl+I conflicts with Tab
- Loads all data on startup (no day filter); `App::new` in `tui/src/app.rs`

### GUI (`qt/`) — C++ Qt 6 (Legacy)

Three-tab floating panel (Todo/Idea/Log + "All" tab). Still maintained for parity with CLI.
- **Quick input bar** — type text, select kind (Todo/Idea/Log), press Enter to capture
- **project:** keyword supported in quick input parser (mirrors Rust parser)
- **Overdue boundary**: `due < today` (today is NOT overdue, matches CLI)
- **Timestamps**: written as `yyyy-MM-ddTHH:mm:ss+00:00` (matches Rust `to_rfc3339`)
- **Schema migration**: `addColumnIfMissing()` handles old DBs missing `project` column
- **Database**: QSqlDatabase with QSQLITE driver, same schema as Rust CLI

## Code Patterns

- **No ORM** — raw SQL via `rusqlite` (Rust) / QSqlQuery (Qt), with parameterized queries. Tags stored as JSON strings in TEXT columns.
- **Immutable DB path** — built once in `default_db_path()`, never mutated.
- **Error handling**: `main()` prints errors to stderr and exits with code 1. Handler functions propagate `rusqlite::Result`. No `if let Ok` silent swallowing.
- **CLI args** defined in `src/cli.rs` using clap derive macros (`#[derive(Parser)]`, `#[derive(Subcommand)]`). `-D` is a global flag.
- **sqlite-migrate** vendored at `vendor/sqlite-migrate/` (workspace member) so `git clone` alone is sufficient to build.
