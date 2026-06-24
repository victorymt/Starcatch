use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Priority,
    pub status: TodoStatus,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub project: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    P0, // 🔴 紧急
    P1, // 🟡 重要
    P2, // 🟢 一般
    P3, // ⚪ 低优
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
            Priority::P3 => write!(f, "P3"),
        }
    }
}

impl Priority {
    pub fn icon(&self) -> &'static str {
        match self {
            Priority::P0 => "🔴",
            Priority::P1 => "🟡",
            Priority::P2 => "🟢",
            Priority::P3 => "⚪",
        }
    }

    /// Sort order: P0 (most urgent) = 0, P3 (lowest) = 3
    pub fn order(&self) -> i32 {
        match self {
            Priority::P0 => 0,
            Priority::P1 => 1,
            Priority::P2 => 2,
            Priority::P3 => 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TodoStatus {
    Pending,
    Done,
    Archived,
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoStatus::Pending => write!(f, "pending"),
            TodoStatus::Done => write!(f, "done"),
            TodoStatus::Archived => write!(f, "archived"),
        }
    }
}
