use clap::{Parser, Subcommand};

/// ⭐ Starcatch (星捕) — Catch your starlight ideas.
#[derive(Parser, Debug)]
#[command(name = "starcatch", version, about = "Catch your starlight ideas ✨")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Database path (default: ~/.local/share/starcatch/starcatch.db)
    #[arg(short = 'D', long, global = true)]
    pub db: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 📋 Todo management
    #[command(subcommand)]
    Todo(TodoCommands),

    /// 💭 Idea management
    #[command(subcommand)]
    Idea(IdeaCommands),

    /// 📓 Daily log management
    #[command(subcommand)]
    Log(LogCommands),

    /// 🚰 Pipe mode: read from stdin and capture
    Pipe(PipeArgs),
}

// ─── Todo ───

#[derive(Subcommand, Debug)]
pub enum TodoCommands {
    /// Add a new todo
    Add(TodoAddArgs),
    /// List todos
    List(TodoListArgs),
    /// Mark todo as done
    Done { id: String },
    /// Mark todo as archived
    Archive { id: String },
}

#[derive(clap::Args, Debug)]
pub struct TodoAddArgs {
    /// Todo title
    pub title: String,

    /// Description
    #[arg(short, long)]
    pub desc: Option<String>,

    /// Priority: P0 (🔴 urgent), P1 (🟡 important), P2 (🟢 normal), P3 (⚪ low)
    #[arg(short, long, default_value = "P2")]
    pub priority: String,

    /// Due date (YYYY-MM-DD)
    #[arg(long)]
    pub due: Option<String>,

    /// Tags (comma separated)
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct TodoListArgs {
    /// Show only pending todos
    #[arg(long)]
    pub pending: bool,

    /// Show only done todos
    #[arg(long)]
    pub done: bool,

    /// Show only archived todos
    #[arg(long)]
    pub archived: bool,

    /// Show all todos (including archived)
    #[arg(long)]
    pub all: bool,

    /// Filter by tag
    #[arg(short, long)]
    pub tag: Option<String>,
}

// ─── Idea ───

#[derive(Subcommand, Debug)]
pub enum IdeaCommands {
    /// Add a new idea
    Add(IdeaAddArgs),
    /// List recent ideas
    List(IdeaListArgs),
}

#[derive(clap::Args, Debug)]
pub struct IdeaAddArgs {
    /// Idea title (required)
    pub title: String,

    /// Extended content (optional)
    #[arg(short, long)]
    pub content: Option<String>,

    /// Source of inspiration
    #[arg(short, long)]
    pub source: Option<String>,

    /// Tags (comma separated)
    #[arg(short, long)]
    pub tag: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct IdeaListArgs {
    /// How many days back to show
    #[arg(short, long, default_value = "7")]
    pub days: i64,
}

// ─── Log ───

#[derive(Subcommand, Debug)]
pub enum LogCommands {
    /// Add a new log entry
    Add(LogAddArgs),
    /// List recent logs
    List(LogListArgs),
}

#[derive(clap::Args, Debug)]
pub struct LogAddArgs {
    /// Log content
    pub content: String,

    /// Mood (happy, sad, excited, ...)
    #[arg(short, long)]
    pub mood: Option<String>,

    /// Tags (comma separated)
    #[arg(short, long)]
    pub tag: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct LogListArgs {
    /// How many days back to show
    #[arg(short, long, default_value = "1")]
    pub days: i64,
}

// ─── Pipe ───

#[derive(clap::Args, Debug)]
pub struct PipeArgs {
    /// Type of capture: todo, idea, log
    pub r#type: String,
}
