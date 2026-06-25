use chrono::Utc;
use rusqlite::{Connection, Result};

use crate::models::*;

/// Fields that can be updated on a todo. All optional — only `Some` fields are applied.
#[derive(Debug, Clone, Default)]
pub struct TodoUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<Priority>,
    pub due_date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub project: Option<String>,
}

/// Fields that can be updated on an idea.
#[derive(Debug, Clone, Default)]
pub struct IdeaUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// Fields that can be updated on a log.
#[derive(Debug, Clone, Default)]
pub struct LogUpdate {
    pub content: Option<String>,
    pub mood: Option<String>,
    pub tags: Option<Vec<String>>,
}

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

pub fn get_todo(conn: &Connection, id: &str) -> Result<Todo> {
    conn.query_row("SELECT * FROM todos WHERE id = ?1", rusqlite::params![id], todo_from_row)
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

pub fn update_todo(conn: &Connection, id: &str, update: &TodoUpdate) -> Result<()> {
    let existing = get_todo(conn, id)?;
    let now = Utc::now();

    let updated = Todo {
        title: update.title.clone().unwrap_or(existing.title),
        description: update.description.clone().or(existing.description),
        priority: update.priority.clone().unwrap_or(existing.priority),
        due_date: update.due_date.clone().or(existing.due_date),
        tags: update.tags.clone().unwrap_or(existing.tags),
        project: update.project.clone().or(existing.project),
        status: existing.status,
        updated_at: now,
        ..existing
    };

    let tags_json = serde_json::to_string(&updated.tags).unwrap_or_default();
    conn.execute(
        "UPDATE todos SET title=?1, description=?2, priority=?3, due_date=?4, tags=?5, project=?6, updated_at=?7 WHERE id=?8",
        rusqlite::params![
            updated.title,
            updated.description,
            updated.priority.to_string(),
            updated.due_date,
            tags_json,
            updated.project,
            updated.updated_at.to_rfc3339(),
            id,
        ],
    )?;
    Ok(())
}

pub fn update_todo_status(conn: &Connection, id: &str, status: &TodoStatus) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE todos SET status = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![status.to_string(), now, id],
    )?;
    Ok(())
}

pub fn delete_todo(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM todos WHERE id = ?1", rusqlite::params![id])?;
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

pub fn get_idea(conn: &Connection, id: &str) -> Result<Idea> {
    conn.query_row("SELECT * FROM ideas WHERE id = ?1", rusqlite::params![id], idea_from_row)
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

pub fn update_idea(conn: &Connection, id: &str, update: &IdeaUpdate) -> Result<()> {
    let existing = get_idea(conn, id)?;

    let updated = Idea {
        title: update.title.clone().unwrap_or(existing.title),
        content: update.content.clone().or(existing.content),
        source: update.source.clone().or(existing.source),
        tags: update.tags.clone().unwrap_or(existing.tags),
        ..existing
    };

    let tags_json = serde_json::to_string(&updated.tags).unwrap_or_default();
    conn.execute(
        "UPDATE ideas SET title=?1, content=?2, source=?3, tags=?4 WHERE id=?5",
        rusqlite::params![updated.title, updated.content, updated.source, tags_json, id],
    )?;
    Ok(())
}

pub fn delete_idea(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM ideas WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
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

pub fn get_log(conn: &Connection, id: &str) -> Result<Log> {
    conn.query_row("SELECT * FROM logs WHERE id = ?1", rusqlite::params![id], log_from_row)
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

pub fn update_log(conn: &Connection, id: &str, update: &LogUpdate) -> Result<()> {
    let existing = get_log(conn, id)?;
    let now = Some(Utc::now());

    let updated = Log {
        content: update.content.clone().unwrap_or(existing.content),
        mood: update.mood.clone().or(existing.mood),
        tags: update.tags.clone().unwrap_or(existing.tags),
        updated_at: now,
        ..existing
    };

    let tags_json = serde_json::to_string(&updated.tags).unwrap_or_default();
    conn.execute(
        "UPDATE logs SET content=?1, mood=?2, tags=?3, updated_at=?4 WHERE id=?5",
        rusqlite::params![
            updated.content,
            updated.mood,
            tags_json,
            updated.updated_at.map(|t| t.to_rfc3339()),
            id,
        ],
    )?;
    Ok(())
}

pub fn delete_log(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM logs WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
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

// ─── Global search ───

#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub entity_type: String,
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub created_at: String,
}

pub fn search_all(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let pattern = format!("%{}%", query);
    let mut results = Vec::new();

    // Search todos
    {
        let mut stmt = conn.prepare(
            "SELECT id, title, COALESCE(description, ''), created_at FROM todos
             WHERE title LIKE ?1 OR description LIKE ?1
             ORDER BY created_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern], |row| {
            Ok(SearchResult {
                entity_type: "todo".to_string(),
                id: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get::<_, String>(2)?,
                created_at: row.get(3)?,
            })
        })?;
        for row in rows {
            results.push(row?);
        }
    }

    // Search ideas
    {
        let mut stmt = conn.prepare(
            "SELECT id, title, COALESCE(content, ''), created_at FROM ideas
             WHERE title LIKE ?1 OR content LIKE ?1
             ORDER BY created_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern], |row| {
            Ok(SearchResult {
                entity_type: "idea".to_string(),
                id: row.get(0)?,
                title: row.get(1)?,
                subtitle: row.get::<_, String>(2)?,
                created_at: row.get(3)?,
            })
        })?;
        for row in rows {
            results.push(row?);
        }
    }

    // Search logs
    {
        let mut stmt = conn.prepare(
            "SELECT id, content, COALESCE(mood, ''), created_at FROM logs
             WHERE content LIKE ?1 OR mood LIKE ?1
             ORDER BY created_at DESC LIMIT 20",
        )?;
        let rows = stmt.query_map(rusqlite::params![pattern], |row| {
            Ok(SearchResult {
                entity_type: "log".to_string(),
                id: row.get(0)?,
                title: row.get::<_, String>(1)?,
                subtitle: row.get::<_, String>(2)?,
                created_at: row.get(3)?,
            })
        })?;
        for row in rows {
            results.push(row?);
        }
    }

    Ok(results)
}

// ─── Stats ───

#[derive(Debug, Clone, serde::Serialize)]
pub struct Stats {
    pub pending_todos: i64,
    pub done_today: i64,
    pub total_todos: i64,
    pub ideas_7d: i64,
    pub logs_7d: i64,
}

pub fn get_stats(conn: &Connection) -> Result<Stats> {
    let pending_todos: i64 = conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;

    let done_today: i64 = conn.query_row(
        "SELECT COUNT(*) FROM todos WHERE status = 'done' AND date(updated_at) = date('now')",
        [],
        |row| row.get(0),
    )?;

    let total_todos: i64 = conn.query_row(
        "SELECT COUNT(*) FROM todos",
        [],
        |row| row.get(0),
    )?;

    let ideas_7d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM ideas WHERE created_at >= datetime('now', '-7 days')",
        [],
        |row| row.get(0),
    )?;

    let logs_7d: i64 = conn.query_row(
        "SELECT COUNT(*) FROM logs WHERE created_at >= datetime('now', '-7 days')",
        [],
        |row| row.get(0),
    )?;

    Ok(Stats {
        pending_todos,
        done_today,
        total_todos,
        ideas_7d,
        logs_7d,
    })
}

// ─── Export ───

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExportData {
    pub todos: Vec<Todo>,
    pub ideas: Vec<Idea>,
    pub logs: Vec<Log>,
}

pub fn export_all(conn: &Connection) -> Result<ExportData> {
    let todos = list_todos(conn, None)?;
    let ideas = list_ideas(conn, None)?;
    let logs = list_logs(conn, None)?;
    Ok(ExportData { todos, ideas, logs })
}

pub fn export_csv(conn: &Connection) -> Result<String> {
    let mut wtr = csv::Writer::from_writer(Vec::new());

    // Todos
    let todos = list_todos(conn, None)?;
    for t in &todos {
        wtr.write_record(&[
            "todo",
            &t.id,
            &t.title,
            &t.description.clone().unwrap_or_default(),
            &t.priority.to_string(),
            &t.status.to_string(),
            &t.due_date.clone().unwrap_or_default(),
            &t.tags.join(","),
            &t.project.clone().unwrap_or_default(),
            &t.created_at.to_rfc3339(),
            &t.updated_at.to_rfc3339(),
        ]).ok();
    }

    // Ideas
    let ideas = list_ideas(conn, None)?;
    for item in &ideas {
        wtr.write_record(&[
            "idea",
            &item.id,
            &item.title,
            &item.content.clone().unwrap_or_default(),
            &item.source.clone().unwrap_or_default(),
            &item.tags.join(","),
            &item.created_at.to_rfc3339(),
        ]).ok();
    }

    // Logs
    let logs = list_logs(conn, None)?;
    for l in &logs {
        wtr.write_record(&[
            "log",
            &l.id,
            &l.content,
            &l.mood.clone().unwrap_or_default(),
            &l.tags.join(","),
            &l.created_at.to_rfc3339(),
        ]).ok();
    }

    let data = wtr.into_inner().map_err(|e| {
        rusqlite::Error::InvalidParameterName(format!("CSV write error: {}", e))
    })?;
    String::from_utf8(data).map_err(|e| {
        rusqlite::Error::InvalidParameterName(format!("UTF-8 error: {}", e))
    })
}

// ═══════════════════════════════════════════════════════════
// ─── Tests ───
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusqlite::Connection;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    fn sample_todo(conn: &Connection, title: &str) -> Todo {
        let todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: Some("test desc".to_string()),
            priority: Priority::P1,
            status: TodoStatus::Pending,
            due_date: Some("2026-12-31".to_string()),
            tags: vec!["test".to_string(), "dev".to_string()],
            project: Some("myproject".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        insert_todo(conn, &todo).unwrap();
        todo
    }

    fn sample_idea(conn: &Connection, title: &str) -> Idea {
        let idea = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            content: Some("test content".to_string()),
            source: Some("twitter".to_string()),
            context_window: None,
            tags: vec!["idea-tag".to_string()],
            created_at: Utc::now(),
        };
        insert_idea(conn, &idea).unwrap();
        idea
    }

    fn sample_log(conn: &Connection, content: &str) -> Log {
        let log = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: content.to_string(),
            mood: Some("happy".to_string()),
            tags: vec!["log-tag".to_string()],
            created_at: Utc::now(),
            updated_at: None,
        };
        insert_log(conn, &log).unwrap();
        log
    }

    // ─── Todo CRUD ───

    #[test]
    fn test_insert_and_get_todo() {
        let conn = setup();
        let todo = sample_todo(&conn, "my todo");
        let fetched = get_todo(&conn, &todo.id).unwrap();
        assert_eq!(fetched.title, "my todo");
        assert_eq!(fetched.priority, Priority::P1);
        assert_eq!(fetched.due_date.as_deref(), Some("2026-12-31"));
        assert_eq!(fetched.tags, vec!["test", "dev"]);
        assert_eq!(fetched.project.as_deref(), Some("myproject"));
    }

    #[test]
    fn test_get_nonexistent_todo() {
        let conn = setup();
        let result = get_todo(&conn, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_update_todo_all_fields() {
        let conn = setup();
        let todo = sample_todo(&conn, "original");

        let update = TodoUpdate {
            title: Some("updated title".to_string()),
            description: Some("updated desc".to_string()),
            priority: Some(Priority::P0),
            due_date: Some("2027-01-01".to_string()),
            tags: Some(vec!["urgent".to_string()]),
            project: Some("newproject".to_string()),
        };
        update_todo(&conn, &todo.id, &update).unwrap();

        let fetched = get_todo(&conn, &todo.id).unwrap();
        assert_eq!(fetched.title, "updated title");
        assert_eq!(fetched.description.as_deref(), Some("updated desc"));
        assert_eq!(fetched.priority, Priority::P0);
        assert_eq!(fetched.due_date.as_deref(), Some("2027-01-01"));
        assert_eq!(fetched.tags, vec!["urgent"]);
        assert_eq!(fetched.project.as_deref(), Some("newproject"));
    }

    #[test]
    fn test_update_todo_partial() {
        let conn = setup();
        let todo = sample_todo(&conn, "original");

        let update = TodoUpdate {
            title: Some("only title".to_string()),
            ..Default::default()
        };
        update_todo(&conn, &todo.id, &update).unwrap();

        let fetched = get_todo(&conn, &todo.id).unwrap();
        assert_eq!(fetched.title, "only title");
        // Other fields unchanged
        assert_eq!(fetched.priority, Priority::P1);
        assert_eq!(fetched.due_date.as_deref(), Some("2026-12-31"));
        assert_eq!(fetched.tags, vec!["test", "dev"]);
    }

    #[test]
    fn test_update_todo_nonexistent() {
        let conn = setup();
        let update = TodoUpdate::default();
        let result = update_todo(&conn, "nonexistent", &update);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_todo_status() {
        let conn = setup();
        let todo = sample_todo(&conn, "task");
        update_todo_status(&conn, &todo.id, &TodoStatus::Done).unwrap();
        let fetched = get_todo(&conn, &todo.id).unwrap();
        assert_eq!(fetched.status, TodoStatus::Done);
    }

    #[test]
    fn test_delete_todo() {
        let conn = setup();
        let todo = sample_todo(&conn, "to delete");
        delete_todo(&conn, &todo.id).unwrap();
        assert!(get_todo(&conn, &todo.id).is_err());
    }

    #[test]
    fn test_delete_nonexistent_todo() {
        let conn = setup();
        // Deleting non-existent row is not an error in SQLite
        let result = delete_todo(&conn, "nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_todos_by_status() {
        let conn = setup();
        let t1 = sample_todo(&conn, "pending task");
        let t2 = sample_todo(&conn, "done task");
        update_todo_status(&conn, &t2.id, &TodoStatus::Done).unwrap();

        let pending = list_todos(&conn, Some("pending")).unwrap();
        assert!(pending.iter().any(|t| t.id == t1.id));
        assert!(!pending.iter().any(|t| t.id == t2.id));

        let done = list_todos(&conn, Some("done")).unwrap();
        assert!(done.iter().any(|t| t.id == t2.id));
    }

    #[test]
    fn test_list_todos_priority_order() {
        let conn = setup();
        let t_p0 = sample_todo(&conn, "p0");
        update_todo(&conn, &t_p0.id, &TodoUpdate { priority: Some(Priority::P0), ..Default::default() }).unwrap();
        let t_p3 = sample_todo(&conn, "p3");
        update_todo(&conn, &t_p3.id, &TodoUpdate { priority: Some(Priority::P3), ..Default::default() }).unwrap();

        let all = list_todos(&conn, None).unwrap();
        let positions: Vec<_> = all.iter().map(|t| t.priority.order()).collect();
        // Should be sorted ascending: P0(0) < P1(1) < P2(2) < P3(3)
        assert!(positions.windows(2).all(|w| w[0] <= w[1]));
    }

    // ─── Idea CRUD ───

    #[test]
    fn test_insert_and_get_idea() {
        let conn = setup();
        let idea = sample_idea(&conn, "my idea");
        let fetched = get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "my idea");
        assert_eq!(fetched.content.as_deref(), Some("test content"));
        assert_eq!(fetched.source.as_deref(), Some("twitter"));
        assert_eq!(fetched.tags, vec!["idea-tag"]);
    }

    #[test]
    fn test_get_nonexistent_idea() {
        let conn = setup();
        assert!(get_idea(&conn, "nonexistent").is_err());
    }

    #[test]
    fn test_update_idea() {
        let conn = setup();
        let idea = sample_idea(&conn, "original");

        let update = IdeaUpdate {
            title: Some("updated idea".to_string()),
            content: Some("new content".to_string()),
            source: Some("github".to_string()),
            tags: Some(vec!["newtag".to_string()]),
        };
        update_idea(&conn, &idea.id, &update).unwrap();

        let fetched = get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "updated idea");
        assert_eq!(fetched.content.as_deref(), Some("new content"));
        assert_eq!(fetched.source.as_deref(), Some("github"));
        assert_eq!(fetched.tags, vec!["newtag"]);
    }

    #[test]
    fn test_update_idea_partial() {
        let conn = setup();
        let idea = sample_idea(&conn, "original");

        let update = IdeaUpdate {
            title: Some("only title".to_string()),
            ..Default::default()
        };
        update_idea(&conn, &idea.id, &update).unwrap();

        let fetched = get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "only title");
        assert_eq!(fetched.source.as_deref(), Some("twitter")); // unchanged
    }

    #[test]
    fn test_delete_idea() {
        let conn = setup();
        let idea = sample_idea(&conn, "to delete");
        delete_idea(&conn, &idea.id).unwrap();
        assert!(get_idea(&conn, &idea.id).is_err());
    }

    #[test]
    fn test_list_ideas_by_days() {
        let conn = setup();
        sample_idea(&conn, "recent");
        // Within 7 days: should find it
        let recent = list_ideas(&conn, Some(7)).unwrap();
        assert!(!recent.is_empty());
        // Within 365 days: should still find it
        let year = list_ideas(&conn, Some(365)).unwrap();
        assert!(!year.is_empty());
    }

    // ─── Log CRUD ───

    #[test]
    fn test_insert_and_get_log() {
        let conn = setup();
        let log = sample_log(&conn, "my log");
        let fetched = get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "my log");
        assert_eq!(fetched.mood.as_deref(), Some("happy"));
        assert_eq!(fetched.tags, vec!["log-tag"]);
    }

    #[test]
    fn test_get_nonexistent_log() {
        let conn = setup();
        assert!(get_log(&conn, "nonexistent").is_err());
    }

    #[test]
    fn test_update_log() {
        let conn = setup();
        let log = sample_log(&conn, "original");

        let update = LogUpdate {
            content: Some("updated content".to_string()),
            mood: Some("productive".to_string()),
            tags: Some(vec!["updated-tag".to_string()]),
        };
        update_log(&conn, &log.id, &update).unwrap();

        let fetched = get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "updated content");
        assert_eq!(fetched.mood.as_deref(), Some("productive"));
        assert_eq!(fetched.tags, vec!["updated-tag"]);
        assert!(fetched.updated_at.is_some()); // timestamp set
    }

    #[test]
    fn test_update_log_partial() {
        let conn = setup();
        let log = sample_log(&conn, "original");

        let update = LogUpdate {
            content: Some("only content".to_string()),
            ..Default::default()
        };
        update_log(&conn, &log.id, &update).unwrap();

        let fetched = get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "only content");
        assert_eq!(fetched.mood.as_deref(), Some("happy")); // unchanged
    }

    #[test]
    fn test_delete_log() {
        let conn = setup();
        let log = sample_log(&conn, "to delete");
        delete_log(&conn, &log.id).unwrap();
        assert!(get_log(&conn, &log.id).is_err());
    }

    // ─── Search ───

    #[test]
    fn test_search_finds_todo_by_title() {
        let conn = setup();
        sample_todo(&conn, "fix login bug");
        let results = search_all(&conn, "login").unwrap();
        assert!(results.iter().any(|r| r.entity_type == "todo" && r.title.contains("login")));
    }

    #[test]
    fn test_search_finds_idea_by_content() {
        let conn = setup();
        let idea = sample_idea(&conn, "my idea");
        // content is "test content"
        let results = search_all(&conn, "content").unwrap();
        assert!(results.iter().any(|r| r.entity_type == "idea" && r.id == idea.id));
    }

    #[test]
    fn test_search_finds_log_by_content() {
        let conn = setup();
        let log = sample_log(&conn, "deploy v2.0");
        let results = search_all(&conn, "deploy").unwrap();
        assert!(results.iter().any(|r| r.entity_type == "log" && r.id == log.id));
    }

    #[test]
    fn test_search_across_all_tables() {
        let conn = setup();
        sample_todo(&conn, "api design");
        sample_idea(&conn, "api v2 ideas");
        sample_log(&conn, "worked on api");

        let results = search_all(&conn, "api").unwrap();
        let types: Vec<&str> = results.iter().map(|r| r.entity_type.as_str()).collect();
        assert!(types.contains(&"todo"));
        assert!(types.contains(&"idea"));
        assert!(types.contains(&"log"));
    }

    #[test]
    fn test_search_no_results() {
        let conn = setup();
        sample_todo(&conn, "task");
        let results = search_all(&conn, "zzz_nonexistent_zzz").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_empty_query() {
        let conn = setup();
        sample_todo(&conn, "task");
        // Empty query matches everything (LIKE '%%')
        let results = search_all(&conn, "").unwrap();
        assert!(!results.is_empty());
    }

    // ─── Stats ───

    #[test]
    fn test_stats_empty_db() {
        let conn = setup();
        let stats = get_stats(&conn).unwrap();
        assert_eq!(stats.pending_todos, 0);
        assert_eq!(stats.done_today, 0);
        assert_eq!(stats.total_todos, 0);
        assert_eq!(stats.ideas_7d, 0);
        assert_eq!(stats.logs_7d, 0);
    }

    #[test]
    fn test_stats_with_data() {
        let conn = setup();
        sample_todo(&conn, "task 1");
        sample_todo(&conn, "task 2");
        sample_idea(&conn, "idea 1");
        sample_log(&conn, "log 1");

        let stats = get_stats(&conn).unwrap();
        assert_eq!(stats.pending_todos, 2);
        assert_eq!(stats.total_todos, 2);
        assert!(stats.ideas_7d >= 1);
        assert!(stats.logs_7d >= 1);
    }

    #[test]
    fn test_stats_done_today() {
        let conn = setup();
        let todo = sample_todo(&conn, "task");
        update_todo_status(&conn, &todo.id, &TodoStatus::Done).unwrap();

        let stats = get_stats(&conn).unwrap();
        assert_eq!(stats.pending_todos, 0);
        assert_eq!(stats.done_today, 1);
    }

    // ─── Export ───

    #[test]
    fn test_export_all_json() {
        let conn = setup();
        sample_todo(&conn, "t1");
        sample_idea(&conn, "i1");
        sample_log(&conn, "l1");

        let data = export_all(&conn).unwrap();
        assert_eq!(data.todos.len(), 1);
        assert_eq!(data.ideas.len(), 1);
        assert_eq!(data.logs.len(), 1);
    }

    #[test]
    fn test_export_all_empty() {
        let conn = setup();
        let data = export_all(&conn).unwrap();
        assert!(data.todos.is_empty());
        assert!(data.ideas.is_empty());
        assert!(data.logs.is_empty());
    }

    #[test]
    fn test_export_csv_has_records() {
        let conn = setup();
        let t = sample_todo(&conn, "csv todo");
        let i = sample_idea(&conn, "csv idea");
        let l = sample_log(&conn, "csv log");

        let csv_str = export_csv(&conn).unwrap();
        // Each entity type and ID should appear
        assert!(csv_str.contains("todo,"), "missing todo: {}", csv_str);
        assert!(csv_str.contains(&t.id), "missing todo id: {}", csv_str);
        assert!(csv_str.contains("idea,"), "missing idea: {}", csv_str);
        assert!(csv_str.contains(&i.id), "missing idea id: {}", csv_str);
        assert!(csv_str.contains("log,"), "missing log: {}", csv_str);
        assert!(csv_str.contains(&l.id), "missing log id: {}", csv_str);
    }

    #[test]
    fn test_export_csv_empty() {
        let conn = setup();
        let csv = export_csv(&conn).unwrap();
        assert!(csv.is_empty());
    }
}
