use clap::{Parser, Subcommand, ValueEnum};

/// ⭐ Starcatch (星捕) — Catch your starlight ideas.
#[derive(Parser, Debug)]
#[command(name = "starcatch", version, about = "Catch your starlight ideas ✨")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Database path (default: ~/.local/share/starcatch/starcatch.db)
    #[arg(short = 'D', long, global = true)]
    pub db: Option<String>,

    /// Output as formatted JSON instead of human-readable text
    #[arg(long, global = true)]
    pub json: bool,
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

    /// 🔍 Global search across todos, ideas, and logs
    Search(SearchArgs),

    /// 📊 Show statistics overview
    Stats,

    /// 📤 Export all data
    Export(ExportArgs),

    /// 🐚 Generate shell completions
    Completions(CompletionsArgs),
}

// ─── Todo ───

#[derive(Subcommand, Debug)]
pub enum TodoCommands {
    /// Add a new todo
    Add(TodoAddArgs),
    /// List todos
    List(TodoListArgs),
    /// Edit a todo (title, priority, due, tags, project, description)
    Edit(TodoEditArgs),
    /// Show full details of a todo
    Show { id: String },
    /// Mark todo as done
    Done { id: String },
    /// Mark todo as archived
    Archive { id: String },
    /// Reopen todo (set back to pending)
    Reopen { id: String },
    /// Permanently delete a todo
    Delete { id: String },
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

    /// Due date (YYYY-MM-DD or natural language like "tomorrow", "next Monday", "3天")
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
pub struct TodoEditArgs {
    /// ID of the todo to edit
    pub id: String,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New description
    #[arg(long)]
    pub desc: Option<String>,

    /// New priority: P0 (🔴), P1 (🟡), P2 (🟢), P3 (⚪)
    #[arg(short, long)]
    pub priority: Option<String>,

    /// New due date
    #[arg(long)]
    pub due: Option<String>,

    /// New tags (comma separated, replaces existing)
    #[arg(short, long)]
    pub tag: Option<String>,

    /// New project name
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

    /// Filter by project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,

    /// Show only overdue todos (due_date < today, status != done)
    #[arg(long)]
    pub overdue: bool,

    /// Show only today's todos (due_date == today)
    #[arg(long)]
    pub today: bool,
}

// ─── Idea ───

#[derive(Subcommand, Debug)]
pub enum IdeaCommands {
    /// Add a new idea
    Add(IdeaAddArgs),
    /// List recent ideas
    List(IdeaListArgs),
    /// Edit an idea
    Edit(IdeaEditArgs),
    /// Show full details of an idea
    Show { id: String },
    /// Permanently delete an idea
    Delete { id: String },
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

    /// Project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct IdeaListArgs {
    /// How many days back to show
    #[arg(short, long, default_value = "7")]
    pub days: i64,

    /// Filter by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Filter by project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct IdeaEditArgs {
    /// ID of the idea to edit
    pub id: String,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New content
    #[arg(short, long)]
    pub content: Option<String>,

    /// New source
    #[arg(short, long)]
    pub source: Option<String>,

    /// New tags (comma separated, replaces existing)
    #[arg(short, long)]
    pub tag: Option<String>,

    /// New project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

// ─── Log ───

#[derive(Subcommand, Debug)]
pub enum LogCommands {
    /// Add a new log entry
    Add(LogAddArgs),
    /// List recent logs
    List(LogListArgs),
    /// Edit a log entry
    Edit(LogEditArgs),
    /// Show full details of a log entry
    Show { id: String },
    /// Permanently delete a log entry
    Delete { id: String },
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

    /// Project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct LogListArgs {
    /// How many days back to show
    #[arg(short, long, default_value = "1")]
    pub days: i64,

    /// Filter by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Filter by mood
    #[arg(short, long)]
    pub mood: Option<String>,

    /// Filter by project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

#[derive(clap::Args, Debug)]
pub struct LogEditArgs {
    /// ID of the log to edit
    pub id: String,

    /// New content
    #[arg(short, long)]
    pub content: Option<String>,

    /// New mood
    #[arg(short, long)]
    pub mood: Option<String>,

    /// New tags (comma separated, replaces existing)
    #[arg(short, long)]
    pub tag: Option<String>,

    /// New project name
    #[arg(short = 'P', long)]
    pub project: Option<String>,
}

// ─── Pipe ───

#[derive(clap::Args, Debug)]
pub struct PipeArgs {
    /// Type of capture: todo, idea, log
    pub r#type: String,
}

// ─── Search ───

#[derive(clap::Args, Debug)]
pub struct SearchArgs {
    /// Search query (matched against title and content across todos, ideas, and logs)
    pub query: String,
}

// ─── Export ───

#[derive(clap::Args, Debug)]
pub struct ExportArgs {
    /// Export format
    #[arg(short, long, default_value = "json")]
    pub format: ExportFormat,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportFormat {
    /// JSON format (pretty-printed)
    Json,
    /// CSV format (one file per entity type)
    Csv,
}


// ─── Completions ───

#[derive(clap::Args, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// Elvish shell
    Elvish,
    /// PowerShell,
    PowerShell,
}
