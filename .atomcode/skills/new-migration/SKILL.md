---
name: new-migration
description: Generate a new SQLite migration file for Starcatch. Uses the custom sqlite-migrate library.
user_invocable: true
disable_model_invocation: false
---

# new-migration

Generate a new migration file for the Starcatch SQLite database.

## When to use

When you need to:
- Add a new table
- Add/modify/remove columns on an existing table
- Add indexes
- Seed initial data

## Usage

```
/new-migration add_tags_to_ideas
```

## Migration template

Create a new file `src/db/migrations/<timestamp>_<name>.rs` following the existing pattern:

```rust
use rusqlite::Connection;

pub fn up(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        -- Your SQL here
    ")?;
    Ok(())
}

pub fn down(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        -- Rollback SQL here
    ")?;
    Ok(())
}
```

Then register the migration in `src/db/migrations/mod.rs`.
