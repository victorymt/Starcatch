# CLAUDE.md

### Qt GUI (C++)
```bash
cd qt
cmake -B build
cmake --build build
./build/starcatch-qt
```

**Important:** The Rust binary is CLI-only. The GUI is a separate C++ Qt 6 project in `qt/`. Both share the same SQLite database at `~/.local/share/starcatch/starcatch.db` (override with `-D <path>`).

## Architecture

Starcatch is a Wayland-native idea/todo/log capture tool. The CLI is written in Rust, and the GUI is a separate C++ Qt 6 application. Both share the same SQLite database.

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

### Database Layer (`src/db.rs`)

- `open(path)` — opens connection, enables WAL + foreign keys
- `migrate(conn)` — idempotent schema creation (3 tables + indexes on status, priority, created_at)
- CRUD per entity: `insert_*`, `list_*`, `update_todo_status` (no update/delete for idea/log beyond the GUI's direct SQL)
- Query pattern: `list_*` functions take an optional filter parameter, build SQL dynamically, return `Result<Vec<T>>`. Row mapping uses private `*_from_row` functions.

### Pipe Mode

Reads entire stdin to a string, then creates the appropriate entity with defaults (P2 priority for todos, no tags/mood) and inserts into the database. Route by the required positional `type` argument: `todo`, `idea`, or `log`.

### GUI (`qt/`) — C++ Qt 6

Qt 6 application with a three-tab floating panel (Todo/Idea/Log). Key patterns:

- **Quick input bar** at the bottom — type text, select kind (Todo/Idea/Log via QComboBox), press Enter or click `➕` to capture
- **Enter key capture** — `QLineEdit::returnPressed()` fires after IME composition is committed (no manual gating needed)
- **Escape** closes the window via QShortcut
- **Data refresh** happens inline after every mutation (insert/update/delete → refresh panel immediately)
- **Toast notifications** display for 2.5 seconds via QTimer::singleShot
- **Database**: Uses Qt's QSqlDatabase with QSQLITE driver, same schema as the Rust CLI

## Code Patterns

- **No ORM** — raw SQL via `rusqlite`, with parameterized queries. Tags stored as JSON strings in TEXT columns.
- **Immutable DB path** — built once in `default_db_path()`, never mutated.
- **Error handling**: `main()` prints errors to stderr and exits with code 1. Handler functions propagate `rusqlite::Result`.
- **CLI args** defined in `src/cli.rs` using clap derive macros (`#[derive(Parser)]`, `#[derive(Subcommand)]`).
