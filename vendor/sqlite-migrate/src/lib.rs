use rusqlite::Connection;

/// A schema definition for `migrate()`.
///
/// # Fields
/// - `create_tables`: full `CREATE TABLE IF NOT EXISTS …` statements that
///    define the current schema. Idempotent — run on every startup.
/// - `add_columns`: `(table, column, sql_type)` tuples for columns added
///    after the initial release. Each becomes
///    `ALTER TABLE {table} ADD COLUMN {column} {sql_type}`.
///    Errors (e.g. column already exists) are silently ignored so both fresh
///    and already-upgraded databases work without a version table.
pub struct Schema {
    pub create_tables: &'static [&'static str],
    pub add_columns: &'static [(&'static str, &'static str, &'static str)],
}

/// Apply the schema to a SQLite connection.
///
/// 1. Run every `CREATE TABLE IF NOT EXISTS` statement — fresh databases get
///    the full schema; existing databases are no-ops.
/// 2. For each `(table, column, type)` tuple, run
///    `ALTER TABLE … ADD COLUMN …` and silently ignore errors.
///    This handles upgrades for databases created before the column existed,
///    while not failing on fresh databases (where the column was already
///    created in step 1) or previously-upgraded ones.
///
/// # Example
///
/// ```rust
/// use sqlite_migrate::{migrate, Schema};
///
/// static SCHEMA: Schema = Schema {
///     create_tables: &[
///         "CREATE TABLE IF NOT EXISTS users (id TEXT PRIMARY KEY, name TEXT NOT NULL)",
///     ],
///     add_columns: &[
///         ("users", "email", "TEXT"),
///         ("users", "age",  "INTEGER"),
///     ],
/// };
///
/// let conn = rusqlite::Connection::open_in_memory().unwrap();
/// migrate(&conn, &SCHEMA).unwrap();
/// ```
pub fn migrate(conn: &Connection, schema: &Schema) -> rusqlite::Result<()> {
    for sql in schema.create_tables {
        conn.execute(sql, [])?;
    }

    for (table, column, col_type) in schema.add_columns {
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {col_type}");
        let _ = conn.execute(&sql, []);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_SCHEMA: Schema = Schema {
        create_tables: &[
            "CREATE TABLE IF NOT EXISTS items (id TEXT PRIMARY KEY, name TEXT NOT NULL, tags TEXT NOT NULL DEFAULT '[]')",
        ],
        add_columns: &[
            ("items", "extra", "TEXT"),
        ],
    };

    #[test]
    fn fresh_db_creates_table() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn, &TEST_SCHEMA).unwrap();

        conn.execute("INSERT INTO items (id, name, extra) VALUES ('1', 'hello', 'x')", [])
            .unwrap();
        let name: String = conn
            .query_row("SELECT name FROM items WHERE id='1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(name, "hello");
    }

    #[test]
    fn existing_db_adds_column() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE items (id TEXT PRIMARY KEY, name TEXT NOT NULL)", [])
            .unwrap();
        conn.execute("INSERT INTO items (id, name) VALUES ('1', 'old')", [])
            .unwrap();

        migrate(&conn, &TEST_SCHEMA).unwrap();

        conn.execute("UPDATE items SET extra = 'new' WHERE id='1'", [])
            .unwrap();
        let extra: String = conn
            .query_row("SELECT extra FROM items WHERE id='1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(extra, "new");
    }

    #[test]
    fn migrate_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn, &TEST_SCHEMA).unwrap();
        migrate(&conn, &TEST_SCHEMA).unwrap(); // second call
    }
}
