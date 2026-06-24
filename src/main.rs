mod cli;
mod db;
mod models;

#[cfg(feature = "gui")]
mod gui;

use std::io::Read;

use chrono::Utc;
use clap::Parser;

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

    let result = match &args.command {
        Some(Commands::Todo(cmd)) => handle_todo(cmd, args.db.as_deref()),
        Some(Commands::Idea(cmd)) => handle_idea(cmd, args.db.as_deref()),
        Some(Commands::Log(cmd)) => handle_log(cmd, args.db.as_deref()),
        Some(Commands::Pipe(cmd)) => handle_pipe(cmd, args.db.as_deref()),
        None => {
            // No subcommand → launch GUI
            #[cfg(feature = "gui")]
            {
                let db_path = args.db.unwrap_or_else(default_db_path);
                launch_gui(db_path)
            }
            #[cfg(not(feature = "gui"))]
            {
                eprintln!("🌙 Starcatch 星捕 — No command given.");
                eprintln!("   Run `starcatch --help` to see available commands.");
                eprintln!("   Or compile with --features gui for the GUI mode.");
                Ok(())
            }
        }
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

// ─── Todo ───

fn handle_todo(cmd: &TodoCommands, db_path: Option<&str>) -> rusqlite::Result<()> {
    let conn = open_db(db_path)?;

    match cmd {
        TodoCommands::Add(args) => {
            let priority = match args.priority.to_uppercase().as_str() {
                "P0" => Priority::P0,
                "P1" => Priority::P1,
                "P3" => Priority::P3,
                _ => Priority::P2,
            };
            let icon = priority.icon();

            let todo = Todo {
                id: uuid::Uuid::new_v4().to_string(),
                title: args.title.clone(),
                description: args.desc.clone(),
                priority,
                status: TodoStatus::Pending,
                due_date: args.due.clone(),
                tags: parse_tags(args.tag.as_deref()),
                project: args.project.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            db::insert_todo(&conn, &todo)?;
            println!("✅ Todo added: {} {}", icon, todo.title);
        }

        TodoCommands::List(args) => {
            // Determine which statuses to show
            let show_statuses: Vec<&str> = if args.all {
                vec!["pending", "done", "archived"]
            } else if args.archived {
                vec!["archived"]
            } else if args.done {
                vec!["done"]
            } else if args.pending {
                vec!["pending"]
            } else {
                // default: pending + done (hide archived)
                vec!["pending", "done"]
            };

            let mut todos = Vec::new();
            for s in &show_statuses {
                if let Ok(mut t) = db::list_todos(&conn, Some(s)) {
                    todos.append(&mut t);
                }
            }

            // Show P0→P3 order (db already sorts by priority within each status batch)
            // Re-sort globally by priority
            let priority_order = |p: &Priority| -> i32 {
                match p {
                    Priority::P0 => 0,
                    Priority::P1 => 1,
                    Priority::P2 => 2,
                    Priority::P3 => 3,
                }
            };
            todos.sort_by_key(|t| (priority_order(&t.priority), std::cmp::Reverse(t.created_at)));

            let filtered: Vec<&Todo> = if let Some(tag) = &args.tag {
                todos.iter().filter(|t| t.tags.iter().any(|t2| t2 == tag)).collect()
            } else {
                todos.iter().collect()
            };

            if filtered.is_empty() {
                println!("📋 No todos found.");
            } else {
                // Show section headers
                println!("📋 Todos:");

                let mut current_section = "";
                for todo in &filtered {
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

                    // Only show status on archived (for clarity)
                }
            }
        }

        TodoCommands::Done { id } => {
            db::update_todo_status(&conn, id, &TodoStatus::Done)?;
            println!("✅ Todo marked as done: {}", id);
        }

        TodoCommands::Archive { id } => {
            db::update_todo_status(&conn, id, &TodoStatus::Archived)?;
            println!("📦 Todo archived: {}", id);
        }
    }

    Ok(())
}

// ─── Idea ───

fn handle_idea(cmd: &IdeaCommands, db_path: Option<&str>) -> rusqlite::Result<()> {
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
            println!("💡 Idea captured: {}", idea.title);
        }

        IdeaCommands::List(args) => {
            let ideas = db::list_ideas(&conn, Some(args.days))?;
            if ideas.is_empty() {
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

fn handle_log(cmd: &LogCommands, db_path: Option<&str>) -> rusqlite::Result<()> {
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
            let mood_icon = log.mood.as_deref().unwrap_or("");
            println!("📓 Log saved {}{}", mood_icon, if !mood_icon.is_empty() { " " } else { "" });
        }

        LogCommands::List(args) => {
            let logs = db::list_logs(&conn, Some(args.days))?;
            if logs.is_empty() {
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
            let todo = Todo {
                id: uuid::Uuid::new_v4().to_string(),
                title: input,
                description: None,
                priority: Priority::P2,
                status: TodoStatus::Pending,
                due_date: None,
                tags: vec![],
                project: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            db::insert_todo(&conn, &todo)?;
            println!("✅ Todo (from pipe): {}", todo.title);
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

#[cfg(feature = "gui")]
fn launch_gui(db_path: String) -> rusqlite::Result<()> {
    use eframe::egui::ViewportBuilder;
    let native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([420.0, 520.0])
            .with_title("⭐ Starcatch 星捕")
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Starcatch 星捕",
        native_options,
        Box::new(|_cc| Ok(Box::new(gui::GuiApp::new(db_path)))),
    )
    .map_err(|e| rusqlite::Error::InvalidParameterName(format!("GUI error: {}", e)))?;

    Ok(())
}
