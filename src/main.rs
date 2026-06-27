mod cli;

use std::io::Read;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;

use cli::*;
use starcatch_core::{IdeaUpdate, LogUpdate, TodoUpdate};
use starcatch_core::*;

fn default_db_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = format!("{}/.local/share/starcatch", home);
    std::fs::create_dir_all(&dir).ok();
    format!("{}/starcatch.db", dir)
}

fn open_db(db_path: Option<&str>) -> Result<rusqlite::Connection> {
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
        Some(Commands::Search(search_args)) => handle_search(search_args, args.db.as_deref(), json),
        Some(Commands::Stats) => handle_stats(args.db.as_deref(), json),
        Some(Commands::Export(export_args)) => handle_export(export_args, args.db.as_deref()),
        Some(Commands::Completions(comp_args)) => handle_completions(comp_args),
        None => {
            eprintln!("🌙 Starcatch 星捕 — No command given.");
            eprintln!("   Run `starcatch --help` to see available commands.");
            eprintln!("   Or run `starcatch-tui` for the terminal interface.");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}


// ═══════════════════════════════════════════════════════════
// ─── Todo ───
// ═══════════════════════════════════════════════════════════

fn handle_todo(cmd: &TodoCommands, db_path: Option<&str>, json: bool) -> Result<()> {
    let conn = open_db(db_path)?;

    match cmd {
        TodoCommands::Add(args) => handle_todo_add(args, &conn),
        TodoCommands::List(args) => handle_todo_list(args, &conn, json),
        TodoCommands::Edit(args) => handle_todo_edit(args, &conn, json),
        TodoCommands::Show { id } => handle_todo_show(id, &conn, json),
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
        TodoCommands::Delete { id } => {
            db::delete_todo(&conn, id)?;
            println!("🗑️  Todo deleted: {}", id);
            Ok(())
        }
    }
}

fn handle_todo_add(args: &TodoAddArgs, conn: &rusqlite::Connection) -> Result<()> {
    let priority = match args.priority.to_uppercase().as_str() {
        "P0" => Priority::P0,
        "P1" => Priority::P1,
        "P3" => Priority::P3,
        _ => Priority::P2,
    };
    let icon = priority.icon();

    let due_date = args.due.as_deref().and_then(starcatch_core::parser::parse_natural_date);

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

fn handle_todo_list(args: &TodoListArgs, conn: &rusqlite::Connection, json: bool) -> Result<()> {
    let show_statuses = list_visible_statuses(args);
    let mut todos = fetch_todos_by_statuses(conn, &show_statuses)?;
    todos.sort_by_key(|t| (t.priority.order(), std::cmp::Reverse(t.created_at)));

    let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let filtered: Vec<&Todo> = todos.iter().filter(|t| {
        if let Some(ref tag) = args.tag {
            if !t.tags.iter().any(|t2| t2 == tag) {
                return false;
            }
        }
        if let Some(ref proj) = args.project {
            if t.project.as_deref() != Some(proj.as_str()) {
                return false;
            }
        }
        if args.overdue {
            if t.status == TodoStatus::Done {
                return false;
            }
            match &t.due_date {
                Some(due) if due.as_str() < today.as_str() => {}
                _ => return false,
            }
        }
        if args.today {
            match &t.due_date {
                Some(due) if due.as_str() == today.as_str() => {}
                _ => return false,
            }
        }
        true
    }).collect();

    if json {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
    } else {
        render_todo_list(&filtered);
    }
    Ok(())
}

fn handle_todo_edit(args: &TodoEditArgs, conn: &rusqlite::Connection, json: bool) -> Result<()> {
    let priority = args.priority.as_deref().map(|p| match p.to_uppercase().as_str() {
        "P0" => Priority::P0,
        "P1" => Priority::P1,
        "P3" => Priority::P3,
        _ => Priority::P2,
    });

    let due_date = args.due.as_deref().and_then(starcatch_core::parser::parse_natural_date);

    let update = TodoUpdate {
        title: args.title.clone(),
        description: args.desc.clone(),
        priority,
        due_date,
        tags: args.tag.as_deref().map(|s| parse_tags(Some(s))),
        project: args.project.clone(),
    };

    db::update_todo(conn, &args.id, &update)?;

    if json {
        let updated = db::get_todo(conn, &args.id)?;
        println!("{}", serde_json::to_string_pretty(&updated)?);
    } else {
        println!("✏️  Todo updated: {}", args.id);
    }
    Ok(())
}

fn handle_todo_show(id: &str, conn: &rusqlite::Connection, json: bool) -> Result<()> {
    let todo = db::get_todo(conn, id)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&todo)?);
    } else {
        println!("📋 Todo: {}", todo.id);
        println!("   Title:       {}", todo.title);
        println!("   Priority:    {} {}", todo.priority.icon(), todo.priority);
        println!("   Status:      {}", todo.status);
        println!("   Due:         {}", todo.due_date.as_deref().unwrap_or("-"));
        println!("   Tags:        {}", if todo.tags.is_empty() { "-".to_string() } else { todo.tags.join(", ") });
        println!("   Project:     {}", todo.project.as_deref().unwrap_or("-"));
        println!("   Description: {}", todo.description.as_deref().unwrap_or("-"));
        println!("   Created:     {}", todo.created_at.format("%Y-%m-%d %H:%M"));
        println!("   Updated:     {}", todo.updated_at.format("%Y-%m-%d %H:%M"));
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
) -> Result<Vec<Todo>> {
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

// ═══════════════════════════════════════════════════════════
// ─── Idea ───
// ═══════════════════════════════════════════════════════════

fn handle_idea(cmd: &IdeaCommands, db_path: Option<&str>, json: bool) -> Result<()> {
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
                project: args.project.clone(),
                created_at: Utc::now(),
            };

            db::insert_idea(&conn, &idea)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&idea)?);
            } else {
                println!("💡 Idea captured: {}", idea.title);
            }
            Ok(())
        }

        IdeaCommands::List(args) => {
            let ideas = db::list_ideas(&conn, Some(args.days))?;

            let filtered: Vec<&Idea> = ideas.iter().filter(|i| {
                if let Some(ref tag) = args.tag {
                    if !i.tags.iter().any(|t| t == tag) {
                        return false;
                    }
                }
                if let Some(ref proj) = args.project {
                    if i.project.as_deref() != Some(proj.as_str()) {
                        return false;
                    }
                }
                true
            }).collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&filtered)?);
            } else if filtered.is_empty() {
                println!("💭 No ideas in the last {} days.", args.days);
            } else {
                println!("💭 Ideas (last {} days):", args.days);
                for idea in &filtered {
                    let source = idea.source.as_deref().unwrap_or("?");
                    let tags = idea.tags.join(", ");
                    let tag_str = if tags.is_empty() { "".to_string() } else { format!(" [{}]", tags) };
                    let proj_str = idea.project.as_deref().map(|p| format!(" | project: {}", p)).unwrap_or_default();
                    println!(
                        "  💡 {} {}{}{} | from: {}",
                        idea.created_at.format("%m-%d %H:%M"),
                        idea.title,
                        tag_str,
                        proj_str,
                        source,
                    );
                }
            }
            Ok(())
        }

        IdeaCommands::Edit(args) => {
            let update = IdeaUpdate {
                title: args.title.clone(),
                content: args.content.clone(),
                source: args.source.clone(),
                tags: args.tag.as_deref().map(|s| parse_tags(Some(s))),
                project: args.project.clone(),
            };
            db::update_idea(&conn, &args.id, &update)?;
            if json {
                let updated = db::get_idea(&conn, &args.id)?;
                println!("{}", serde_json::to_string_pretty(&updated)?);
            } else {
                println!("✏️  Idea updated: {}", args.id);
            }
            Ok(())
        }

        IdeaCommands::Show { id } => {
            let idea = db::get_idea(&conn, id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&idea)?);
            } else {
                println!("💡 Idea: {}", idea.id);
                println!("   Title:   {}", idea.title);
                println!("   Content: {}", idea.content.as_deref().unwrap_or("-"));
                println!("   Source:  {}", idea.source.as_deref().unwrap_or("-"));
                println!("   Tags:    {}", if idea.tags.is_empty() { "-".to_string() } else { idea.tags.join(", ") });
                println!("   Project: {}", idea.project.as_deref().unwrap_or("-"));
                println!("   Created: {}", idea.created_at.format("%Y-%m-%d %H:%M"));
            }
            Ok(())
        }

        IdeaCommands::Delete { id } => {
            db::delete_idea(&conn, id)?;
            println!("🗑️  Idea deleted: {}", id);
            Ok(())
        }
    }
}

// ═══════════════════════════════════════════════════════════
// ─── Log ───
// ═══════════════════════════════════════════════════════════

fn handle_log(cmd: &LogCommands, db_path: Option<&str>, json: bool) -> Result<()> {
    let conn = open_db(db_path)?;

    match cmd {
        LogCommands::Add(args) => {
            let log = Log {
                id: uuid::Uuid::new_v4().to_string(),
                content: args.content.clone(),
                mood: args.mood.clone(),
                tags: parse_tags(args.tag.as_deref()),
                project: args.project.clone(),
                created_at: Utc::now(),
                updated_at: None,
            };

            db::insert_log(&conn, &log)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&log)?);
            } else {
                let preview = starcatch_core::safe_truncate_bytes(&log.content, 60);
                println!("📓 Log saved: {}", preview);
            }
            Ok(())
        }

        LogCommands::List(args) => {
            let logs = db::list_logs(&conn, Some(args.days))?;

            let filtered: Vec<&Log> = logs.iter().filter(|l| {
                if let Some(ref tag) = args.tag {
                    if !l.tags.iter().any(|t| t == tag) {
                        return false;
                    }
                }
                if let Some(ref mood) = args.mood {
                    if l.mood.as_deref() != Some(mood.as_str()) {
                        return false;
                    }
                }
                if let Some(ref proj) = args.project {
                    if l.project.as_deref() != Some(proj.as_str()) {
                        return false;
                    }
                }
                true
            }).collect();

            if json {
                println!("{}", serde_json::to_string_pretty(&filtered)?);
            } else if filtered.is_empty() {
                println!("📓 No logs in the last {} days.", args.days);
            } else {
                println!("📓 Logs (last {} days):", args.days);
                for log in &filtered {
                    let mood = log.mood.as_deref().unwrap_or("");
                    let tags = log.tags.join(", ");
                    let tag_str = if tags.is_empty() { "".to_string() } else { format!(" [{}]", tags) };
                    let proj_str = log.project.as_deref().map(|p| format!(" | project: {}", p)).unwrap_or_default();
                    println!("  📝 [{}] {} {}{}{}", log.created_at.format("%m-%d %H:%M"), mood, log.content, tag_str, proj_str);
                }
            }
            Ok(())
        }

        LogCommands::Edit(args) => {
            let update = LogUpdate {
                content: args.content.clone(),
                mood: args.mood.clone(),
                tags: args.tag.as_deref().map(|s| parse_tags(Some(s))),
                project: args.project.clone(),
            };
            db::update_log(&conn, &args.id, &update)?;
            if json {
                let updated = db::get_log(&conn, &args.id)?;
                println!("{}", serde_json::to_string_pretty(&updated)?);
            } else {
                println!("✏️  Log updated: {}", args.id);
            }
            Ok(())
        }

        LogCommands::Show { id } => {
            let log = db::get_log(&conn, id)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&log)?);
            } else {
                println!("📓 Log: {}", log.id);
                println!("   Content: {}", log.content);
                println!("   Mood:    {}", log.mood.as_deref().unwrap_or("-"));
                println!("   Tags:    {}", if log.tags.is_empty() { "-".to_string() } else { log.tags.join(", ") });
                println!("   Project: {}", log.project.as_deref().unwrap_or("-"));
                println!("   Created: {}", log.created_at.format("%Y-%m-%d %H:%M"));
                if let Some(up) = log.updated_at {
                    println!("   Updated: {}", up.format("%Y-%m-%d %H:%M"));
                }
            }
            Ok(())
        }

        LogCommands::Delete { id } => {
            db::delete_log(&conn, id)?;
            println!("🗑️  Log deleted: {}", id);
            Ok(())
        }
    }
}

// ═══════════════════════════════════════════════════════════
// ─── Pipe ───
// ═══════════════════════════════════════════════════════════

fn handle_pipe(args: &PipeArgs, db_path: Option<&str>) -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("stdin read error")?;

    let input = input.trim_end().to_string();
    if input.is_empty() {
        eprintln!("⚠️  No input from pipe.");
        return Ok(());
    }

    let conn = open_db(db_path)?;

    match args.r#type.as_str() {
        "todo" => {
            let parsed = starcatch_core::parser::parse_pipe_todo(&input);
            let todo = Todo {
                id: uuid::Uuid::new_v4().to_string(),
                title: parsed.title,
                description: None,
                priority: parsed.priority,
                status: TodoStatus::Pending,
                due_date: parsed.due_date,
                tags: parsed.tags,
                project: parsed.project,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            db::insert_todo(&conn, &todo)?;
            println!("✅ Todo (from pipe): {} {}", todo.priority.icon(), todo.title);
        }
        "idea" => {
            let parsed = starcatch_core::parser::parse_pipe_idea(&input);
            let idea = Idea {
                id: uuid::Uuid::new_v4().to_string(),
                title: parsed.title,
                content: None,
                source: parsed.source,
                context_window: None,
                tags: parsed.tags,
                project: parsed.project,
                created_at: Utc::now(),
            };
            db::insert_idea(&conn, &idea)?;
            println!("💡 Idea (from pipe): {}", idea.title);
        }
        "log" => {
            let parsed = starcatch_core::parser::parse_pipe_log(&input);
            let log = Log {
                id: uuid::Uuid::new_v4().to_string(),
                content: parsed.content,
                mood: parsed.mood,
                tags: parsed.tags,
                project: parsed.project,
                created_at: Utc::now(),
                updated_at: None,
            };
            db::insert_log(&conn, &log)?;
            println!("📓 Log (from pipe)");
        }
        other => {
            eprintln!("⚠️  Unknown pipe type: {}. Use: todo, idea, log", other);
        }
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════
// ─── Search ───
// ═══════════════════════════════════════════════════════════

fn handle_search(args: &SearchArgs, db_path: Option<&str>, json: bool) -> Result<()> {
    let conn = open_db(db_path)?;
    let results = db::search_all(&conn, &args.query)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else if results.is_empty() {
        println!("🔍 No results for \"{}\".", args.query);
    } else {
        println!("🔍 Search results for \"{}\":", args.query);
        for r in &results {
            let type_icon = match r.entity_type.as_str() {
                "todo" => "📋",
                "idea" => "💡",
                "log" => "📝",
                _ => "•",
            };
            let sub = if r.subtitle.is_empty() {
                "".to_string()
            } else {
                format!(" — {}", safe_truncate_bytes(&r.subtitle, 63))
            };
            println!("  {} [{}] {}{}", type_icon, r.entity_type, r.title, sub);
            println!("      id: {} | {}", r.id, r.created_at);
        }
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// ─── Stats ───
// ═══════════════════════════════════════════════════════════

fn handle_stats(db_path: Option<&str>, json: bool) -> Result<()> {
    let conn = open_db(db_path)?;
    let stats = db::get_stats(&conn)?;

    if json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("📊 Starcatch Stats:");
        println!("   📋 Pending todos:  {}", stats.pending_todos);
        println!("   ✅ Done today:     {}", stats.done_today);
        println!("   📦 Total todos:    {}", stats.total_todos);
        println!("   💡 Ideas (7d):     {}", stats.ideas_7d);
        println!("   📓 Logs (7d):      {}", stats.logs_7d);
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// ─── Export ───
// ═══════════════════════════════════════════════════════════

fn handle_export(args: &ExportArgs, db_path: Option<&str>) -> Result<()> {
    let conn = open_db(db_path)?;

    match args.format {
        ExportFormat::Json => {
            let data = db::export_all(&conn)?;
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        ExportFormat::Csv => {
            let csv = db::export_csv(&conn)?;
            print!("{}", csv);
        }
    }
    Ok(())
}


// ═══════════════════════════════════════════════════════════
// ─── Completions ───
// ═══════════════════════════════════════════════════════════

fn handle_completions(args: &CompletionsArgs) -> Result<()> {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell as ClapShell};

    let mut cmd = <Args as CommandFactory>::command();
    let name = "starcatch";

    let shell = match args.shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::Elvish => ClapShell::Elvish,
        Shell::PowerShell => ClapShell::PowerShell,
    };

    generate(shell, &mut cmd, name, &mut std::io::stdout());
    Ok(())
}

// ═══════════════════════════════════════════════════════════
// ─── Tests ───
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Handler integration tests ───

    /// Create a temp DB, run migrations, return the file path (keeps TempDir alive).
    fn setup_temp_db() -> (tempfile::TempDir, String) {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let path_str = path.to_str().unwrap().to_string();
        let conn = db::open(&path_str).unwrap();
        db::migrate(&conn).unwrap();
        (dir, path_str)
    }

    /// Add a todo and return its ID.
    fn add_todo_via_handler(db_path: &str) -> String {
        let args = TodoAddArgs {
            title: "test todo".to_string(),
            desc: Some("desc".to_string()),
            priority: "P1".to_string(),
            due: None,
            tag: Some("test,dev".to_string()),
            project: Some("proj".to_string()),
        };
        handle_todo_add(&args, &open_db(Some(db_path)).unwrap()).unwrap();
        // Fetch the most recent todo to get its ID
        let conn = open_db(Some(db_path)).unwrap();
        let todos = db::list_todos(&conn, None).unwrap();
        todos.first().unwrap().id.clone()
    }

    #[test]
    fn handler_todo_edit_updates_fields() {
        let (_dir, db_path) = setup_temp_db();
        let id = add_todo_via_handler(&db_path);

        let edit_args = TodoEditArgs {
            id: id.clone(),
            title: Some("edited".to_string()),
            desc: None,
            priority: Some("P0".to_string()),
            due: Some("2027-06-01".to_string()),
            tag: Some("urgent".to_string()),
            project: Some("newproj".to_string()),
        };
        handle_todo_edit(&edit_args, &open_db(Some(&db_path)).unwrap(), false).unwrap();

        let conn = open_db(Some(&db_path)).unwrap();
        let todo = db::get_todo(&conn, &id).unwrap();
        assert_eq!(todo.title, "edited");
        assert_eq!(todo.priority, Priority::P0);
        assert_eq!(todo.due_date.as_deref(), Some("2027-06-01"));
        assert_eq!(todo.tags, vec!["urgent"]);
        assert_eq!(todo.project.as_deref(), Some("newproj"));
    }

    #[test]
    fn handler_todo_edit_partial() {
        let (_dir, db_path) = setup_temp_db();
        let id = add_todo_via_handler(&db_path);

        let edit_args = TodoEditArgs {
            id: id.clone(),
            title: Some("only title".to_string()),
            desc: None,
            priority: None,
            due: None,
            tag: None,
            project: None,
        };
        handle_todo_edit(&edit_args, &open_db(Some(&db_path)).unwrap(), false).unwrap();

        let conn = open_db(Some(&db_path)).unwrap();
        let todo = db::get_todo(&conn, &id).unwrap();
        assert_eq!(todo.title, "only title");
        assert_eq!(todo.priority, Priority::P1); // unchanged
    }

    #[test]
    fn handler_todo_edit_nonexistent_errors() {
        let (_dir, db_path) = setup_temp_db();
        let edit_args = TodoEditArgs {
            id: "nonexistent".to_string(),
            title: Some("nope".to_string()),
            desc: None,
            priority: None,
            due: None,
            tag: None,
            project: None,
        };
        let result = handle_todo_edit(&edit_args, &open_db(Some(&db_path)).unwrap(), false);
        assert!(result.is_err());
    }

    #[test]
    fn handler_todo_show_json() {
        let (_dir, db_path) = setup_temp_db();
        let id = add_todo_via_handler(&db_path);
        let result = handle_todo_show(&id, &open_db(Some(&db_path)).unwrap(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn handler_todo_show_nonexistent_errors() {
        let (_dir, db_path) = setup_temp_db();
        let result = handle_todo_show("nonexistent", &open_db(Some(&db_path)).unwrap(), false);
        assert!(result.is_err());
    }

    #[test]
    fn handler_todo_delete_removes() {
        let (_dir, db_path) = setup_temp_db();
        let id = add_todo_via_handler(&db_path);
        let conn = open_db(Some(&db_path)).unwrap();
        db::delete_todo(&conn, &id).unwrap();
        assert!(db::get_todo(&conn, &id).is_err());
    }

    #[test]
    fn handler_todo_list_overdue() {
        let (_dir, db_path) = setup_temp_db();
        // Add overdue todo
        let conn = open_db(Some(&db_path)).unwrap();
        let overdue = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: "overdue".to_string(),
            description: None,
            priority: Priority::P2,
            status: TodoStatus::Pending,
            due_date: Some("2020-01-01".to_string()),
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db::insert_todo(&conn, &overdue).unwrap();

        let _list_args = TodoListArgs {
            pending: false, done: false, archived: false, all: true,
            tag: None, project: None, overdue: true, today: false,
        };
        // We test via direct DB access since handler prints
        let todos = fetch_todos_by_statuses(&conn, &["pending", "done", "archived"]).unwrap();
        let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let filtered: Vec<_> = todos.iter().filter(|t| {
            t.status != TodoStatus::Done
                && t.due_date.as_ref().map_or(false, |d| d.as_str() < today.as_str())
        }).collect();
        assert!(!filtered.is_empty());
        assert_eq!(filtered[0].title, "overdue");
    }

    #[test]
    fn handler_todo_list_today() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();
        let today_str = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let today_todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: "today task".to_string(),
            description: None,
            priority: Priority::P2,
            status: TodoStatus::Pending,
            due_date: Some(today_str.clone()),
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db::insert_todo(&conn, &today_todo).unwrap();

        let todos = db::list_todos(&conn, Some("pending")).unwrap();
        let filtered: Vec<_> = todos.iter().filter(|t| {
            t.due_date.as_ref().map_or(false, |d| d.as_str() == today_str.as_str())
        }).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "today task");
    }

    #[test]
    fn handler_todo_list_project_filter() {
        let (_dir, db_path) = setup_temp_db();
        add_todo_via_handler(&db_path); // has project="proj"

        let conn = open_db(Some(&db_path)).unwrap();
        let todos = db::list_todos(&conn, None).unwrap();
        let filtered: Vec<_> = todos.iter().filter(|t| t.project.as_deref() == Some("proj")).collect();
        assert_eq!(filtered.len(), 1);

        let none: Vec<_> = todos.iter().filter(|t| t.project.as_deref() == Some("nope")).collect();
        assert!(none.is_empty());
    }

    // ─── Idea handler tests ───

    #[test]
    fn handler_idea_edit_and_delete() {
        let (_dir, db_path) = setup_temp_db();
        // Add idea
        let conn = open_db(Some(&db_path)).unwrap();
        let idea = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: "original".to_string(),
            content: Some("content".to_string()),
            source: Some("src".to_string()),
            context_window: None,
            tags: vec!["old".to_string()],
            project: None,
            created_at: Utc::now(),
        };
        db::insert_idea(&conn, &idea).unwrap();

        // Edit
        let update = IdeaUpdate {
            title: Some("updated".to_string()),
            content: None,
            source: None,
            tags: Some(vec!["new".to_string()]),
            ..Default::default()
        };
        db::update_idea(&conn, &idea.id, &update).unwrap();
        let fetched = db::get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "updated");
        assert_eq!(fetched.tags, vec!["new"]);

        // Delete
        db::delete_idea(&conn, &idea.id).unwrap();
        assert!(db::get_idea(&conn, &idea.id).is_err());
    }

    #[test]
    fn handler_idea_list_tag_filter() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();
        let idea1 = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: "tagged".to_string(),
            content: None, source: None, context_window: None,
            tags: vec!["important".to_string()],
            project: Some("backend".to_string()),
            created_at: Utc::now(),
        };
        let idea2 = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: "other".to_string(),
            content: None, source: None, context_window: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
        };
        db::insert_idea(&conn, &idea1).unwrap();
        db::insert_idea(&conn, &idea2).unwrap();

        let ideas = db::list_ideas(&conn, Some(7)).unwrap();
        let filtered: Vec<_> = ideas.iter().filter(|i| i.tags.iter().any(|t| t == "important")).collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title, "tagged");
    }

    // ─── Log handler tests ───

    #[test]
    fn handler_log_edit_and_delete() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();
        let log = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: "original".to_string(),
            mood: Some("happy".to_string()),
            tags: vec!["old".to_string()],
            project: None,
            created_at: Utc::now(),
            updated_at: None,
        };
        db::insert_log(&conn, &log).unwrap();

        // Edit
        let update = LogUpdate {
            content: Some("updated".to_string()),
            mood: Some("productive".to_string()),
            tags: None,
            ..Default::default()
        };
        db::update_log(&conn, &log.id, &update).unwrap();
        let fetched = db::get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "updated");
        assert_eq!(fetched.mood.as_deref(), Some("productive"));

        // Delete
        db::delete_log(&conn, &log.id).unwrap();
        assert!(db::get_log(&conn, &log.id).is_err());
    }

    #[test]
    fn handler_log_list_filters() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();
        let log1 = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: "log1".to_string(),
            mood: Some("happy".to_string()),
            tags: vec!["work".to_string()],
            project: Some("backend".to_string()),
            created_at: Utc::now(),
            updated_at: None,
        };
        let log2 = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: "log2".to_string(),
            mood: Some("sad".to_string()),
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: None,
        };
        db::insert_log(&conn, &log1).unwrap();
        db::insert_log(&conn, &log2).unwrap();

        let logs = db::list_logs(&conn, Some(7)).unwrap();

        // Mood filter
        let by_mood: Vec<_> = logs.iter().filter(|l| l.mood.as_deref() == Some("happy")).collect();
        assert_eq!(by_mood.len(), 1);

        // Tag filter
        let by_tag: Vec<_> = logs.iter().filter(|l| l.tags.iter().any(|t| t == "work")).collect();
        assert_eq!(by_tag.len(), 1);
    }

    // ─── Search handler tests ───

    #[test]
    fn handler_search_finds_across_types() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();
        // Insert one of each
        let todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: "deploy API".to_string(),
            description: None,
            priority: Priority::P2,
            status: TodoStatus::Pending,
            due_date: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let idea = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: "API v2".to_string(),
            content: Some("deploy strategy".to_string()),
            source: None, context_window: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
        };
        let log = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: "deployed to prod".to_string(),
            mood: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: None,
        };
        db::insert_todo(&conn, &todo).unwrap();
        db::insert_idea(&conn, &idea).unwrap();
        db::insert_log(&conn, &log).unwrap();

        let results = db::search_all(&conn, "deploy").unwrap();
        // Should find all three
        assert!(results.iter().any(|r| r.entity_type == "todo"));
        assert!(results.iter().any(|r| r.entity_type == "idea"));
        assert!(results.iter().any(|r| r.entity_type == "log"));
    }

    #[test]
    fn handler_search_no_match() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();
        let results = db::search_all(&conn, "zzz_nonexistent").unwrap();
        assert!(results.is_empty());
    }

    // ─── Stats handler tests ───

    #[test]
    fn handler_stats_reflects_data() {
        let (_dir, db_path) = setup_temp_db();
        add_todo_via_handler(&db_path);

        let conn = open_db(Some(&db_path)).unwrap();
        let stats = db::get_stats(&conn).unwrap();
        assert!(stats.pending_todos >= 1);
        assert!(stats.total_todos >= 1);
    }

    // ─── Export handler tests ───

    #[test]
    fn handler_export_json_includes_data() {
        let (_dir, db_path) = setup_temp_db();
        add_todo_via_handler(&db_path);

        let conn = open_db(Some(&db_path)).unwrap();
        let data = db::export_all(&conn).unwrap();
        assert!(!data.todos.is_empty());
    }

    // ─── Completions handler test ───

    #[test]
    fn handler_completions_all_shells() {
        // Verify completions generation doesn't panic for any shell
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::Elvish, Shell::PowerShell] {
            let args = CompletionsArgs { shell };
            // Redirect stdout to avoid output clutter
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                use clap::CommandFactory;
                use clap_complete::{generate, Shell as ClapShell};
                let mut cmd = <Args as CommandFactory>::command();
                let s = match args.shell {
                    Shell::Bash => ClapShell::Bash,
                    Shell::Zsh => ClapShell::Zsh,
                    Shell::Fish => ClapShell::Fish,
                    Shell::Elvish => ClapShell::Elvish,
                    Shell::PowerShell => ClapShell::PowerShell,
                };
                generate(s, &mut cmd, "starcatch", &mut std::io::sink());
            }));
            assert!(result.is_ok(), "completions failed for shell");
        }
    }

    // ─── Pipe handler integration tests ───

    /// Simulate `echo "P0 fix it #urgent due:tomorrow project:api" | starcatch pipe todo`
    #[test]
    fn handler_pipe_todo_parses_and_inserts() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_todo("P0 fix it #urgent due:tomorrow project:api");
        let todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: parsed.title,
            description: None,
            priority: parsed.priority,
            status: TodoStatus::Pending,
            due_date: parsed.due_date,
            tags: parsed.tags,
            project: parsed.project,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db::insert_todo(&conn, &todo).unwrap();

        let fetched = db::get_todo(&conn, &todo.id).unwrap();
        assert_eq!(fetched.title, "fix it");
        assert_eq!(fetched.priority, Priority::P0);
        assert_eq!(fetched.tags, vec!["urgent"]);
        assert_eq!(fetched.project.as_deref(), Some("api"));
        assert!(fetched.due_date.is_some());
    }

    /// Simulate `echo "AI idea #tech source:twitter project:myapp" | starcatch pipe idea`
    #[test]
    fn handler_pipe_idea_parses_and_inserts() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_idea("AI idea #tech source:twitter project:myapp");
        let idea = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: parsed.title,
            content: None,
            source: parsed.source,
            context_window: None,
            tags: parsed.tags,
            project: parsed.project,
            created_at: Utc::now(),
        };
        db::insert_idea(&conn, &idea).unwrap();

        let fetched = db::get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "AI idea");
        assert_eq!(fetched.tags, vec!["tech"]);
        assert_eq!(fetched.source.as_deref(), Some("twitter"));
        assert_eq!(fetched.project.as_deref(), Some("myapp"));
    }

    /// Simulate `echo "shipped v2 mood:happy project:backend" | starcatch pipe log`
    #[test]
    fn handler_pipe_log_parses_and_inserts() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_log("shipped v2 mood:happy project:backend");
        let log = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: parsed.content,
            mood: parsed.mood,
            tags: parsed.tags,
            project: parsed.project,
            created_at: Utc::now(),
            updated_at: None,
        };
        db::insert_log(&conn, &log).unwrap();

        let fetched = db::get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "shipped v2");
        assert_eq!(fetched.mood.as_deref(), Some("happy"));
        assert_eq!(fetched.project.as_deref(), Some("backend"));
    }

    /// Pipe todo with plain text only — no keywords
    #[test]
    fn handler_pipe_todo_plain_text() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_todo("just a simple task");
        let todo = Todo {
            id: uuid::Uuid::new_v4().to_string(),
            title: parsed.title,
            description: None,
            priority: parsed.priority,
            status: TodoStatus::Pending,
            due_date: parsed.due_date,
            tags: parsed.tags,
            project: parsed.project,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db::insert_todo(&conn, &todo).unwrap();

        let fetched = db::get_todo(&conn, &todo.id).unwrap();
        assert_eq!(fetched.title, "just a simple task");
        assert_eq!(fetched.priority, Priority::P2); // default
        assert!(fetched.tags.is_empty());
        assert!(fetched.project.is_none());
        assert!(fetched.due_date.is_none());
    }

    /// Pipe idea with only a title — no extra metadata
    #[test]
    fn handler_pipe_idea_plain_title() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_idea("a plain idea");
        let idea = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: parsed.title,
            content: None,
            source: None,
            context_window: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
        };
        db::insert_idea(&conn, &idea).unwrap();

        let fetched = db::get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "a plain idea");
        assert!(fetched.source.is_none());
        assert!(fetched.tags.is_empty());
        assert!(fetched.project.is_none());
    }

    /// Pipe log with only content — no mood or project
    #[test]
    fn handler_pipe_log_plain_content() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_log("just a work log");
        let log = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: parsed.content,
            mood: None,
            tags: vec![],
            project: None,
            created_at: Utc::now(),
            updated_at: None,
        };
        db::insert_log(&conn, &log).unwrap();

        let fetched = db::get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "just a work log");
        assert!(fetched.mood.is_none());
        assert!(fetched.tags.is_empty());
        assert!(fetched.project.is_none());
    }

    /// Pipe idea with project: using fullwidth colon (project：myapp)
    #[test]
    fn handler_pipe_idea_project_fullwidth_colon() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_idea("good idea project：myapp");
        let idea = Idea {
            id: uuid::Uuid::new_v4().to_string(),
            title: parsed.title,
            content: None,
            source: None,
            context_window: None,
            tags: parsed.tags,
            project: parsed.project,
            created_at: Utc::now(),
        };
        db::insert_idea(&conn, &idea).unwrap();

        let fetched = db::get_idea(&conn, &idea.id).unwrap();
        assert_eq!(fetched.title, "good idea");
        assert_eq!(fetched.project.as_deref(), Some("myapp"));
    }

    /// Pipe log with project: separate token
    #[test]
    fn handler_pipe_log_project_separate_token() {
        let (_dir, db_path) = setup_temp_db();
        let conn = open_db(Some(&db_path)).unwrap();

        let parsed = parse_pipe_log("done stuff project: infra");
        let log = Log {
            id: uuid::Uuid::new_v4().to_string(),
            content: parsed.content,
            mood: parsed.mood,
            tags: parsed.tags,
            project: parsed.project,
            created_at: Utc::now(),
            updated_at: None,
        };
        db::insert_log(&conn, &log).unwrap();

        let fetched = db::get_log(&conn, &log.id).unwrap();
        assert_eq!(fetched.content, "done stuff");
        assert_eq!(fetched.project.as_deref(), Some("infra"));
    }
}
