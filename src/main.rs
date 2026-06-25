mod cli;
mod db;
mod models;

use std::io::Read;
use std::sync::LazyLock;

use chrono::{Datelike, Duration, Utc, Weekday};
use clap::Parser;
use regex::Regex;

use cli::*;
use models::*;

fn default_db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = format!("{}/.local/share/starcatch", home);
    std::fs::create_dir_all(&dir).ok();
    format!("{}/starcatch.db", dir)
}

fn open_db(db_path: Option<&str>) -> rusqlite::Result<rusqlite::Connection> {
    let default_path = default_db_path();
    let path = db_path.unwrap_or(&default_path);
    let conn = db::open(path)?;
    db::migrate(&conn)?;
    Ok(conn)
}

fn parse_tags(tag_opt: Option<&str>) -> Vec<String> {
    tag_opt
        .map(|s| s.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default()
}

fn main() {
    let args = Args::parse();
    let json = args.json;

    let result = match &args.command {
        Some(Commands::Todo(cmd)) => handle_todo(cmd, args.db.as_deref(), json),
        Some(Commands::Idea(cmd)) => handle_idea(cmd, args.db.as_deref(), json),
        Some(Commands::Log(cmd)) => handle_log(cmd, args.db.as_deref(), json),
        Some(Commands::Pipe(cmd)) => handle_pipe(cmd, args.db.as_deref()),
        None => {
            eprintln!("🌙 Starcatch 星捕 — No command given.");
            eprintln!("   Run `starcatch --help` to see available commands.");
            eprintln!("   Or use `starcatch-qt` (in the qt/ directory) for the GUI.");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

// ─── Pipe input parser ───

fn is_trailing_punct(c: char) -> bool {
    c.is_ascii_punctuation()
        || c == '\u{3001}' // 、
        || c == '\u{3002}' // 。
        || c == '\u{FF0C}' // ，
        || c == '\u{FF1B}' // ；
        || c == '\u{FF1A}' // ：
        || c == '\u{FF01}' // ！
        || c == '\u{FF1F}' // ？
        || c == '\u{FF09}' // ）
        || c == '\u{FF3D}' // 】
        || c == '\u{300D}' // 》
}

fn trim_trailing_punct(s: &str) -> &str {
    let mut end = s.len();
    for (byte_offset, c) in s.char_indices().rev() {
        if is_trailing_punct(c) {
            end = byte_offset;
        } else {
            break;
        }
    }
    &s[..end]
}

struct ParsedPipeTodo {
    title: String,
    priority: Priority,
    due_date: Option<String>,
    tags: Vec<String>,
}

/// Parse a quick-input string for pipe mode, mirroring the GUI's parseTodoInput().
/// Extracts P0-P3 priority, due:/due： dates, #tags — the rest becomes the title.
fn parse_pipe_todo(raw: &str) -> ParsedPipeTodo {
    let mut priority = Priority::P2;
    let mut due_date = None;
    let mut tags: Vec<String> = Vec::new();
    let mut title_parts: Vec<&str> = Vec::new();

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let token = tokens[i];

        // Priority keywords
        if token == "P0" {
            priority = Priority::P0;
        } else if token == "P1" {
            priority = Priority::P1;
        } else if token == "P3" {
            priority = Priority::P3;
        } else if token == "P2" {
            priority = Priority::P2;
        }
        // due: / due： prefix — value may be in same token or the next
        else if let Some(val) = token.strip_prefix("due:").or_else(|| token.strip_prefix("due：")) {
            let val = val.trim();
            if !val.is_empty() {
                due_date = Some(parse_natural_date(val).unwrap_or_else(|| val.to_string()));
            } else if i + 1 < tokens.len() {
                i += 1;
                let next = tokens[i];
                due_date = Some(parse_natural_date(next).unwrap_or_else(|| next.to_string()));
            }
        }
        // #tag — strip leading # then trim trailing punctuation
        else if let Some(tag) = token.strip_prefix('#') {
            let tag = trim_trailing_punct(tag.trim());
            if !tag.is_empty() {
                tags.push(tag.to_string());
            }
        }
        // Plain title word
        else {
            title_parts.push(token);
        }

        i += 1;
    }

    let title = if title_parts.is_empty() {
        raw.to_string()
    } else {
        title_parts.join(" ")
    };

    ParsedPipeTodo { title, priority, due_date, tags }
}

// ─── Natural date parser ───

fn parse_natural_date(text: &str) -> Option<String> {
    static DATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
    static NUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\s*(天|d|day|days)?(后|後| later)?$").unwrap());
    static NEXT_EN_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)^next\s+(\w+)").unwrap());
    static NEXT_ZH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^下(?:周|星期|礼拜)?(.)").unwrap());
    static THIS_ZH_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(?:这|本|这周|本周|这星期|本星期)(?:周|星期|礼拜)?(.)").unwrap());

    let today = Utc::now().date_naive();
    let t = text.trim();

    // Already yyyy-MM-dd
    if DATE_RE.is_match(t) {
        return Some(t.to_string());
    }

    // Numeric: N (days later)
    if let Some(cap) = NUM_RE.captures(t) {
        let n: i64 = cap[1].parse().unwrap_or(0);
        return Some((today + Duration::days(n)).format("%Y-%m-%d").to_string());
    }

    // Day-of-week maps
    let dow_en: Vec<(&str, Weekday)> = vec![
        ("mon", Weekday::Mon), ("monday", Weekday::Mon),
        ("tue", Weekday::Tue), ("tuesday", Weekday::Tue),
        ("wed", Weekday::Wed), ("wednesday", Weekday::Wed),
        ("thu", Weekday::Thu), ("thursday", Weekday::Thu),
        ("fri", Weekday::Fri), ("friday", Weekday::Fri),
        ("sat", Weekday::Sat), ("saturday", Weekday::Sat),
        ("sun", Weekday::Sun), ("sunday", Weekday::Sun),
    ];
    let dow_zh: Vec<(&str, Weekday)> = vec![
        ("一", Weekday::Mon), ("二", Weekday::Tue), ("三", Weekday::Wed),
        ("四", Weekday::Thu), ("五", Weekday::Fri), ("六", Weekday::Sat),
        ("日", Weekday::Sun), ("天", Weekday::Sun),
    ];

    // Absolute keywords
    match t {
        "今天" | "today" => return Some(today.format("%Y-%m-%d").to_string()),
        "明天" | "tomorrow" => return Some((today + Duration::days(1)).format("%Y-%m-%d").to_string()),
        "后天" | "後天" => return Some((today + Duration::days(2)).format("%Y-%m-%d").to_string()),
        "大后天" | "大後天" => return Some((today + Duration::days(3)).format("%Y-%m-%d").to_string()),
        "昨天" | "yesterday" => return Some((today + Duration::days(-1)).format("%Y-%m-%d").to_string()),
        "下周" | "下週" | "next week" => {
            let delta = 7 - today.weekday().num_days_from_monday() as i64;
            if delta <= 0 { return Some((today + Duration::days(delta + 7)).format("%Y-%m-%d").to_string()); }
            return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
        }
        _ => {}
    }

    // "next <weekday>"
    if let Some(cap) = NEXT_EN_RE.captures(t) {
        let w = cap[1].to_lowercase();
        for (key, wd) in &dow_en {
            if w == *key {
                let target = wd.num_days_from_monday() as i64;
                let cur = today.weekday().num_days_from_monday() as i64;
                let mut delta = target - cur;
                if delta <= 0 { delta += 7; }
                return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
            }
        }
    }

    // "下<星期X>" / "下周<X>"
    if let Some(cap) = NEXT_ZH_RE.captures(t) {
        let ch = &cap[1];
        for (key, wd) in &dow_zh {
            if ch == *key {
                let target = wd.num_days_from_monday() as i64;
                let cur = today.weekday().num_days_from_monday() as i64;
                let mut delta = target - cur;
                if delta <= 0 { delta += 7; }
                return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
            }
        }
    }

    // "本周X" / "这周X"
    if let Some(cap) = THIS_ZH_RE.captures(t) {
        let ch = &cap[1];
        for (key, wd) in &dow_zh {
            if ch == *key {
                let target = wd.num_days_from_monday() as i64;
                let cur = today.weekday().num_days_from_monday() as i64;
                let delta = target - cur;
                return Some((today + Duration::days(delta)).format("%Y-%m-%d").to_string());
            }
        }
    }

    None
}

// ─── Todo ───

fn handle_todo(cmd: &TodoCommands, db_path: Option<&str>, json: bool) -> rusqlite::Result<()> {
    let conn = open_db(db_path)?;

    match cmd {
        TodoCommands::Add(args) => handle_todo_add(args, &conn),
        TodoCommands::List(args) => handle_todo_list(args, &conn, json),
        TodoCommands::Done { id } => {
            db::update_todo_status(&conn, id, &TodoStatus::Done)?;
            println!("✅ Todo marked as done: {}", id);
            Ok(())
        }
        TodoCommands::Archive { id } => {
            db::update_todo_status(&conn, id, &TodoStatus::Archived)?;
            println!("📦 Todo archived: {}", id);
            Ok(())
        }
        TodoCommands::Reopen { id } => {
            db::update_todo_status(&conn, id, &TodoStatus::Pending)?;
            println!("🔄 Todo reopened: {}", id);
            Ok(())
        }
    }
}

fn handle_todo_add(args: &TodoAddArgs, conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let priority = match args.priority.to_uppercase().as_str() {
        "P0" => Priority::P0,
        "P1" => Priority::P1,
        "P3" => Priority::P3,
        _ => Priority::P2,
    };
    let icon = priority.icon();

    let due_date = args.due.as_deref().and_then(parse_natural_date);

    let todo = Todo {
        id: uuid::Uuid::new_v4().to_string(),
        title: args.title.clone(),
        description: args.desc.clone(),
        priority,
        status: TodoStatus::Pending,
        due_date,
        tags: parse_tags(args.tag.as_deref()),
        project: args.project.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    db::insert_todo(conn, &todo)?;
    println!("✅ Todo added: {} {}", icon, todo.title);
    Ok(())
}

fn handle_todo_list(args: &TodoListArgs, conn: &rusqlite::Connection, json: bool) -> rusqlite::Result<()> {
    let show_statuses = list_visible_statuses(args);
    let mut todos = fetch_todos_by_statuses(conn, &show_statuses)?;
    todos.sort_by_key(|t| (t.priority.order(), std::cmp::Reverse(t.created_at)));

    let filtered: Vec<&Todo> = if let Some(tag) = &args.tag {
        todos.iter().filter(|t| t.tags.iter().any(|t2| t2 == tag)).collect()
    } else {
        todos.iter().collect()
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered).unwrap_or_default());
    } else {
        render_todo_list(&filtered);
    }
    Ok(())
}

fn list_visible_statuses(args: &TodoListArgs) -> Vec<&'static str> {
    if args.all {
        vec!["pending", "done", "archived"]
    } else if args.archived {
        vec!["archived"]
    } else if args.done {
        vec!["done"]
    } else if args.pending {
        vec!["pending"]
    } else {
        vec!["pending", "done"]
    }
}

fn fetch_todos_by_statuses(
    conn: &rusqlite::Connection,
    statuses: &[&str],
) -> rusqlite::Result<Vec<Todo>> {
    let mut todos = Vec::new();
    for s in statuses {
        if let Ok(mut batch) = db::list_todos(conn, Some(s)) {
            todos.append(&mut batch);
        }
    }
    Ok(todos)
}

fn render_todo_list(todos: &[&Todo]) {
    if todos.is_empty() {
        println!("📋 No todos found.");
        return;
    }

    println!("📋 Todos:");
    let mut current_section = "";
    for todo in todos {
        let section = match todo.status {
            TodoStatus::Pending => "📋 待办",
            TodoStatus::Done => "✅ 已完成",
            TodoStatus::Archived => "📦 已归档",
        };
        if section != current_section {
            println!("  {}:", section);
            current_section = section;
        }

        let due = todo.due_date.as_deref().unwrap_or("-");
        let tags = todo.tags.join(", ");
        let tag_str = if tags.is_empty() { "".to_string() } else { format!(" [{}]", tags) };
        let status_icon = match todo.status {
            TodoStatus::Pending => "⬜",
            TodoStatus::Done => "✅",
            TodoStatus::Archived => "📦",
        };
        println!(
            "  {} {} {}{} | due: {}",
            todo.priority.icon(),
            status_icon,
            todo.title,
            tag_str,
            due,
        );
    }
}

// ─── Idea ───

fn handle_idea(cmd: &IdeaCommands, db_path: Option<&str>, json: bool) -> rusqlite::Result<()> {
    let conn = open_db(db_path)?;

    match cmd {
        IdeaCommands::Add(args) => {
            let idea = Idea {
                id: uuid::Uuid::new_v4().to_string(),
                title: args.title.clone(),
                content: args.content.clone(),
                source: args.source.clone(),
                context_window: None,
                tags: parse_tags(args.tag.as_deref()),
                created_at: Utc::now(),
            };

            db::insert_idea(&conn, &idea)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&idea).unwrap_or_default());
            } else {
                println!("💡 Idea captured: {}", idea.title);
            }
        }

        IdeaCommands::List(args) => {
            let ideas = db::list_ideas(&conn, Some(args.days))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&ideas).unwrap_or_default());
            } else if ideas.is_empty() {
                println!("💭 No ideas in the last {} days.", args.days);
            } else {
                println!("💭 Ideas (last {} days):", args.days);
                for idea in &ideas {
                    let source = idea.source.as_deref().unwrap_or("?");
                    let tags = idea.tags.join(", ");
                    let tag_str = if tags.is_empty() { "".to_string() } else { format!(" [{}]", tags) };
                    println!(
                        "  💡 {} {}{} | from: {}",
                        idea.created_at.format("%m-%d %H:%M"),
                        idea.title,
                        tag_str,
                        source,
                    );
                }
            }
        }
    }

    Ok(())
}

// ─── Log ───

fn handle_log(cmd: &LogCommands, db_path: Option<&str>, json: bool) -> rusqlite::Result<()> {
    let conn = open_db(db_path)?;

    match cmd {
        LogCommands::Add(args) => {
            let log = Log {
                id: uuid::Uuid::new_v4().to_string(),
                content: args.content.clone(),
                mood: args.mood.clone(),
                tags: parse_tags(args.tag.as_deref()),
                created_at: Utc::now(),
                updated_at: None,
            };

            db::insert_log(&conn, &log)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&log).unwrap_or_default());
            } else {
                let mood_icon = log.mood.as_deref().unwrap_or("");
                println!("📓 Log saved {}{}", mood_icon, if !mood_icon.is_empty() { " " } else { "" });
            }
        }

        LogCommands::List(args) => {
            let logs = db::list_logs(&conn, Some(args.days))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&logs).unwrap_or_default());
            } else if logs.is_empty() {
                println!("📓 No logs in the last {} days.", args.days);
            } else {
                println!("📓 Logs (last {} days):", args.days);
                for log in &logs {
                    let mood = log.mood.as_deref().unwrap_or("");
                    println!("  📝 [{}] {} {}", log.created_at.format("%m-%d %H:%M"), mood, log.content);
                }
            }
        }
    }

    Ok(())
}

// ─── Pipe ───

fn handle_pipe(args: &PipeArgs, db_path: Option<&str>) -> rusqlite::Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| rusqlite::Error::InvalidParameterName(format!("stdin read error: {}", e)))?;

    let input = input.trim().to_string();
    if input.is_empty() {
        eprintln!("⚠️  No input from pipe.");
        return Ok(());
    }

    let conn = open_db(db_path)?;

    match args.r#type.as_str() {
        "todo" => {
            let parsed = parse_pipe_todo(&input);
            let todo = Todo {
                id: uuid::Uuid::new_v4().to_string(),
                title: parsed.title,
                description: None,
                priority: parsed.priority,
                status: TodoStatus::Pending,
                due_date: parsed.due_date,
                tags: parsed.tags,
                project: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            db::insert_todo(&conn, &todo)?;
            println!("✅ Todo (from pipe): {} {}", todo.priority.icon(), todo.title);
        }
        "idea" => {
            let idea = Idea::new(input);
            db::insert_idea(&conn, &idea)?;
            println!("💡 Idea (from pipe): {}", idea.title);
        }
        "log" => {
            let log = Log::new(input);
            db::insert_log(&conn, &log)?;
            println!("📓 Log (from pipe): {}", log.content);
        }
        other => {
            eprintln!("⚠️  Unknown pipe type: {}. Use: todo, idea, log", other);
        }
    }

    Ok(())
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pipe_todo_plain_text() {
        let p = parse_pipe_todo("buy milk");
        assert_eq!(p.title, "buy milk");
        assert_eq!(p.priority, Priority::P2);
        assert!(p.due_date.is_none());
        assert!(p.tags.is_empty());
    }

    #[test]
    fn parse_pipe_todo_with_priority() {
        let p = parse_pipe_todo("P0 fix login bug");
        assert_eq!(p.title, "fix login bug");
        assert_eq!(p.priority, Priority::P0);
    }

    #[test]
    fn parse_pipe_todo_with_due_date() {
        let p = parse_pipe_todo("submit report due:tomorrow");
        assert_eq!(p.title, "submit report");
        assert!(p.due_date.is_some());
    }

    #[test]
    fn parse_pipe_todo_with_due_separate_token() {
        let p = parse_pipe_todo("submit report due: tomorrow");
        assert_eq!(p.title, "submit report");
        assert!(p.due_date.is_some());
    }

    #[test]
    fn parse_pipe_todo_with_tags() {
        let p = parse_pipe_todo("fix login #urgent #backend");
        assert_eq!(p.title, "fix login");
        assert_eq!(p.tags, vec!["urgent", "backend"]);
    }

    #[test]
    fn parse_pipe_todo_tag_trailing_punct() {
        let p = parse_pipe_todo("fix login #urgent.");
        assert_eq!(p.tags, vec!["urgent"]);
    }

    #[test]
    fn parse_pipe_todo_all_features() {
        let p = parse_pipe_todo("P1 fix login bug #urgent due:tomorrow");
        assert_eq!(p.title, "fix login bug");
        assert_eq!(p.priority, Priority::P1);
        assert!(p.due_date.is_some());
        assert_eq!(p.tags, vec!["urgent"]);
    }

    #[test]
    fn parse_pipe_todo_only_keywords_uses_raw() {
        let p = parse_pipe_todo("P0 #urgent due:tomorrow");
        // All tokens consumed as keywords → title falls back to raw
        assert_eq!(p.title, "P0 #urgent due:tomorrow");
        assert_eq!(p.priority, Priority::P0);
        assert!(p.due_date.is_some());
        assert_eq!(p.tags, vec!["urgent"]);
    }

    #[test]
    fn trim_trailing_punct_ascii() {
        assert_eq!(trim_trailing_punct("hello."), "hello");
        assert_eq!(trim_trailing_punct("hello!"), "hello");
    }

    #[test]
    fn trim_trailing_punct_cjk() {
        assert_eq!(trim_trailing_punct("标签。"), "标签");
        assert_eq!(trim_trailing_punct("标签，"), "标签");
    }

    #[test]
    fn trim_trailing_punct_none() {
        assert_eq!(trim_trailing_punct("hello"), "hello");
    }

    #[test]
    fn parse_pipe_todo_due_fullwidth_colon() {
        let p = parse_pipe_todo("报告 due：tomorrow");
        assert_eq!(p.title, "报告");
        assert!(p.due_date.is_some());
    }
}
