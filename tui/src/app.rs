use starcatch_core::db::{self};
use starcatch_core::models::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveView {
    Todo,
    Idea,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Todo,
    Idea,
    Log,
}

pub struct App {
    // Database
    pub db_path: String,

    // Current view
    pub active_view: ActiveView,

    // Data
    pub todos: Vec<Todo>,
    pub ideas: Vec<Idea>,
    pub logs: Vec<Log>,

    // UI state
    pub selected_index: usize,
    pub scroll_offset: usize,

    // Input
    pub editing: bool,        // true = input mode (/), false = command mode
    pub input_text: String,
    pub input_type: InputType,
    pub input_cursor: usize,

    // Status
    pub status_message: Option<String>,
    pub needs_refresh: bool,

    // Toast
    pub toast: Option<(String, Instant)>,
}

use std::time::Instant;

impl App {
    pub fn new(db_path: &str) -> Result<Self, String> {
        // Open and migrate database
        let conn = db::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;
        db::migrate(&conn).map_err(|e| format!("Failed to migrate database: {}", e))?;
        drop(conn); // We'll open connections as needed

        let mut app = Self {
            db_path: db_path.to_string(),
            active_view: ActiveView::Todo,
            todos: vec![],
            ideas: vec![],
            logs: vec![],
            selected_index: 0,
            scroll_offset: 0,
            editing: false,
            input_text: String::new(),
            input_type: InputType::Todo,
            input_cursor: 0,
            status_message: None,
            needs_refresh: true,
            toast: None,
        };
        app.refresh_data();
        Ok(app)
    }

    pub fn refresh_data(&mut self) {
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        self.todos = db::list_todos(&conn, None).unwrap_or_default();
        self.ideas = db::list_ideas(&conn, None).unwrap_or_default();
        self.logs = db::list_logs(&conn, None).unwrap_or_default();
    }

    pub fn refresh_current_list(&mut self) {
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        match self.active_view {
            ActiveView::Todo => {
                self.todos = db::list_todos(&conn, None).unwrap_or_default();
            }
            ActiveView::Idea => {
                self.ideas = db::list_ideas(&conn, None).unwrap_or_default();
            }
            ActiveView::Log => {
                self.logs = db::list_logs(&conn, None).unwrap_or_default();
            }
        }
    }

    pub fn current_list_len(&self) -> usize {
        match self.active_view {
            ActiveView::Todo => self.todos.len(),
            ActiveView::Idea => self.ideas.len(),
            ActiveView::Log => self.logs.len(),
        }
    }

    pub fn submit_input(&mut self) {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return;
        }

        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => {
                self.set_status("Failed to open database");
                return;
            }
        };

        match self.input_type {
            InputType::Todo => {
                let parsed = starcatch_core::parser::parse_pipe_todo(&text);
                let todo = Todo {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: parsed.title,
                    description: None,
                    priority: parsed.priority,
                    status: TodoStatus::Pending,
                    due_date: parsed.due_date,
                    tags: parsed.tags,
                    project: parsed.project,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                if let Err(e) = db::insert_todo(&conn, &todo) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                self.set_status("Todo added");
            }
            InputType::Idea => {
                let parsed = starcatch_core::parser::parse_pipe_idea(&text);
                let idea = Idea {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: parsed.title,
                    content: None,
                    source: parsed.source,
                    context_window: None,
                    tags: parsed.tags,
                    project: parsed.project,
                    created_at: chrono::Utc::now(),
                };
                if let Err(e) = db::insert_idea(&conn, &idea) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                self.set_status("Idea added");
            }
            InputType::Log => {
                let parsed = starcatch_core::parser::parse_pipe_log(&text);
                let log = Log {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: parsed.content,
                    mood: parsed.mood,
                    tags: parsed.tags,
                    project: parsed.project,
                    created_at: chrono::Utc::now(),
                    updated_at: None,
                };
                if let Err(e) = db::insert_log(&conn, &log) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                self.set_status("Log added");
            }
        }

        self.input_text.clear();
        self.input_cursor = 0;
        self.needs_refresh = true;
    }

    pub fn toggle_selected_todo(&mut self) {
        if self.active_view != ActiveView::Todo {
            return;
        }
        if self.selected_index >= self.todos.len() {
            return;
        }
        let todo = &self.todos[self.selected_index];
        let new_status = match todo.status {
            TodoStatus::Pending => TodoStatus::Done,
            TodoStatus::Done => TodoStatus::Pending,
            TodoStatus::Archived => TodoStatus::Archived,
        };
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Err(e) = db::update_todo_status(&conn, &todo.id, &new_status) {
            self.set_status(&format!("Error: {}", e));
            return;
        }
        self.set_status(&format!(
            "Todo marked as {}",
            if new_status == TodoStatus::Done {
                "done"
            } else {
                "pending"
            }
        ));
        self.needs_refresh = true;
    }

    pub fn archive_selected_todo(&mut self) {
        if self.active_view != ActiveView::Todo {
            return;
        }
        if self.selected_index >= self.todos.len() {
            return;
        }
        let id = self.todos[self.selected_index].id.clone();
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        if let Err(e) = db::update_todo_status(&conn, &id, &TodoStatus::Archived) {
            self.set_status(&format!("Error: {}", e));
            return;
        }
        self.set_status("Todo archived");
        self.needs_refresh = true;
        if self.selected_index >= self.todos.len().saturating_sub(1) {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    pub fn delete_selected(&mut self) {
        if self.current_list_len() == 0 {
            return;
        }
        if self.selected_index >= self.current_list_len() {
            return;
        }
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let type_name = match self.active_view {
            ActiveView::Todo => {
                let id = self.todos[self.selected_index].id.clone();
                if let Err(e) = db::delete_todo(&conn, &id) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                "Todo"
            }
            ActiveView::Idea => {
                let id = self.ideas[self.selected_index].id.clone();
                if let Err(e) = db::delete_idea(&conn, &id) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                "Idea"
            }
            ActiveView::Log => {
                let id = self.logs[self.selected_index].id.clone();
                if let Err(e) = db::delete_log(&conn, &id) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                "Log"
            }
        };

        self.set_status(&format!("{} deleted", type_name));
        self.needs_refresh = true;
        if self.selected_index >= self.current_list_len().saturating_sub(1) {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }
}
