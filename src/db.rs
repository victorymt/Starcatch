use rusqlite::{Connection, Result};

use crate::models::*;

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS todos (
            id          TEXT PRIMARY KEY,
            title       TEXT NOT NULL,
            description TEXT,
            priority    TEXT NOT NULL DEFAULT 'P2',
            status      TEXT NOT NULL DEFAULT 'pending',
            due_date    TEXT,
            tags        TEXT NOT NULL DEFAULT '[]',
            project     TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS ideas (
            id              TEXT PRIMARY KEY,
            title           TEXT NOT NULL,
            content         TEXT,
            source          TEXT,
            context_window  TEXT,
            tags            TEXT NOT NULL DEFAULT '[]',
            created_at      TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS logs (
            id          TEXT PRIMARY KEY,
            content     TEXT NOT NULL,
            mood        TEXT,
            tags        TEXT NOT NULL DEFAULT '[]',
            created_at  TEXT NOT NULL,
            updated_at  TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_todos_status ON todos(status);
        CREATE INDEX IF NOT EXISTS idx_todos_priority ON todos(priority);
        CREATE INDEX IF NOT EXISTS idx_ideas_created ON ideas(created_at);
        CREATE INDEX IF NOT EXISTS idx_logs_created ON logs(created_at);
        ",
    )?;
    Ok(())
}

// ─── Todo helpers ───

pub fn insert_todo(conn: &Connection, todo: &Todo) -> Result<()> {
    let tags_json = serde_json::to_string(&todo.tags).unwrap_or_default();
    conn.execute(
        "INSERT INTO todos (id, title, description, priority, status, due_date, tags, project, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![
            todo.id,
            todo.title,
            todo.description,
            todo.priority.to_string(),
            todo.status.to_string(),
            todo.due_date,
            tags_json,
            todo.project,
            todo.created_at.to_rfc3339(),
            todo.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn list_todos(conn: &Connection, status: Option<&str>) -> Result<Vec<Todo>> {
    let sql = match status {
        Some(_) => "SELECT * FROM todos WHERE status = ?1 ORDER BY
                     CASE priority WHEN 'P0' THEN 0 WHEN 'P1' THEN 1 WHEN 'P2' THEN 2 WHEN 'P3' THEN 3 END,
                     created_at DESC",
        None => "SELECT * FROM todos ORDER BY
                 CASE priority WHEN 'P0' THEN 0 WHEN 'P1' THEN 1 WHEN 'P2' THEN 2 WHEN 'P3' THEN 3 END,
                 created_at DESC",
    };

    let mut stmt = conn.prepare(sql)?;

    let rows = if let Some(s) = status {
        stmt.query_map(rusqlite::params![s], todo_from_row)?
    } else {
        stmt.query_map([], todo_from_row)?
    };

    rows.collect()
}

pub fn update_todo_status(conn: &Connection, id: &str, status: &TodoStatus) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE todos SET status = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![status.to_string(), now, id],
    )?;
    Ok(())
}

fn todo_from_row(row: &rusqlite::Row) -> rusqlite::Result<Todo> {
    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    let priority_str: String = row.get("priority")?;
    let priority = match priority_str.as_str() {
        "P0" => Priority::P0,
        "P1" => Priority::P1,
        "P3" => Priority::P3,
        _ => Priority::P2,
    };

    let status_str: String = row.get("status")?;
    let status = match status_str.as_str() {
        "done" => TodoStatus::Done,
        "archived" => TodoStatus::Archived,
        _ => TodoStatus::Pending,
    };

    Ok(Todo {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        priority,
        status,
        due_date: row.get("due_date")?,
        tags,
        project: row.get("project")?,
        created_at: row.get::<_, String>("created_at")?.parse().unwrap_or_default(),
        updated_at: row.get::<_, String>("updated_at")?.parse().unwrap_or_default(),
    })
}

// ─── Idea helpers ───

pub fn insert_idea(conn: &Connection, idea: &Idea) -> Result<()> {
    let tags_json = serde_json::to_string(&idea.tags).unwrap_or_default();
    conn.execute(
        "INSERT INTO ideas (id, title, content, source, context_window, tags, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            idea.id,
            idea.title,
            idea.content,
            idea.source,
            idea.context_window,
            tags_json,
            idea.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn list_ideas(conn: &Connection, days: Option<i64>) -> Result<Vec<Idea>> {
    let sql = match days {
        Some(_) => "SELECT * FROM ideas WHERE created_at >= datetime('now', ?1) ORDER BY created_at DESC",
        None => "SELECT * FROM ideas ORDER BY created_at DESC",
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(d) = days {
        stmt.query_map(rusqlite::params![format!("-{} days", d)], idea_from_row)?
    } else {
        stmt.query_map([], idea_from_row)?
    };

    rows.collect()
}

fn idea_from_row(row: &rusqlite::Row) -> rusqlite::Result<Idea> {
    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    Ok(Idea {
        id: row.get("id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        source: row.get("source")?,
        context_window: row.get("context_window")?,
        tags,
        created_at: row.get::<_, String>("created_at")?.parse().unwrap_or_default(),
    })
}

// ─── Log helpers ───

pub fn insert_log(conn: &Connection, log: &Log) -> Result<()> {
    let tags_json = serde_json::to_string(&log.tags).unwrap_or_default();
    conn.execute(
        "INSERT INTO logs (id, content, mood, tags, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            log.id,
            log.content,
            log.mood,
            tags_json,
            log.created_at.to_rfc3339(),
            log.updated_at.map(|t| t.to_rfc3339()),
        ],
    )?;
    Ok(())
}

pub fn list_logs(conn: &Connection, days: Option<i64>) -> Result<Vec<Log>> {
    let sql = match days {
        Some(_) => "SELECT * FROM logs WHERE created_at >= datetime('now', ?1) ORDER BY created_at DESC",
        None => "SELECT * FROM logs ORDER BY created_at DESC",
    };

    let mut stmt = conn.prepare(sql)?;
    let rows = if let Some(d) = days {
        stmt.query_map(rusqlite::params![format!("-{} days", d)], log_from_row)?
    } else {
        stmt.query_map([], log_from_row)?
    };

    rows.collect()
}

fn log_from_row(row: &rusqlite::Row) -> rusqlite::Result<Log> {
    let tags_str: String = row.get("tags")?;
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    let updated: Option<String> = row.get("updated_at")?;

    Ok(Log {
        id: row.get("id")?,
        content: row.get("content")?,
        mood: row.get("mood")?,
        tags,
        created_at: row.get::<_, String>("created_at")?.parse().unwrap_or_default(),
        updated_at: updated.map(|s| s.parse().unwrap_or_default()),
    })
}
