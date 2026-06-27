use starcatch_core::db::{self};
use starcatch_core::models::*;

/// Convert a char-index (cursor position in characters) to a byte offset
/// for safe string slicing. Returns `s.len()` if `char_idx` is out of range.
pub fn char_idx_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

/// Safely truncate a string to at most `max_bytes` bytes, on a character
/// boundary. If truncation occurs, appends "..." (included in max_bytes).
/// Always yields valid UTF-8 — walks backward to find a char boundary,
/// avoiding the byte-slicing panics that raw `&s[..N]` suffers from with
/// multi-byte UTF-8 (CJK, emoji, etc.).
pub fn safe_truncate_bytes(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        s.to_string()
    } else {
        let cutoff = max_bytes.saturating_sub(3); // room for "..."
        let mut boundary = cutoff;
        while boundary > 0 && !s.is_char_boundary(boundary) {
            boundary -= 1;
        }
        format!("{}...", &s[..boundary])
    }
}

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
    // Database connection (persistent — opened once in new())
    pub conn: rusqlite::Connection,

    // Current view
    pub active_view: ActiveView,

    // Data
    pub todos: Vec<Todo>,
    pub ideas: Vec<Idea>,
    pub logs: Vec<Log>,

    // UI state
    pub selected_index: usize,

    // Input
    pub editing: bool,        // true = input mode (/), false = command mode
    pub editing_item_id: Option<String>,  // set when editing an existing item
    pub input_text: String,
    pub input_type: InputType,
    pub input_cursor: usize,

    // Confirmations
    pub confirm_delete: bool,  // wait for second 'd' to confirm

    // Status
    pub status_message: Option<String>,
    pub needs_refresh: bool,
    pub status_auto_clear: bool,  // clear status on next Tick

}

impl App {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = db::open(db_path).map_err(|e| format!("Failed to open database: {}", e))?;
        db::migrate(&conn).map_err(|e| format!("Failed to migrate database: {}", e))?;

        let mut app = Self {
            conn,
            active_view: ActiveView::Todo,
            todos: vec![],
            ideas: vec![],
            logs: vec![],
            selected_index: 0,
            editing: false,
            editing_item_id: None,
            input_text: String::new(),
            input_type: InputType::Todo,
            input_cursor: 0,
            confirm_delete: false,
            status_message: None,
            needs_refresh: true,
            status_auto_clear: false,
        };
        app.refresh_data();
        Ok(app)
    }

    pub fn refresh_data(&mut self) {
        self.todos = db::list_todos(&self.conn, None).unwrap_or_default();
        self.ideas = db::list_ideas(&self.conn, None).unwrap_or_default();
        self.logs = db::list_logs(&self.conn, None).unwrap_or_default();
    }

    pub fn refresh_current_list(&mut self) {
        match self.active_view {
            ActiveView::Todo => {
                self.todos = db::list_todos(&self.conn, None).unwrap_or_default();
            }
            ActiveView::Idea => {
                self.ideas = db::list_ideas(&self.conn, None).unwrap_or_default();
            }
            ActiveView::Log => {
                self.logs = db::list_logs(&self.conn, None).unwrap_or_default();
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

    /// Submit the current input. Returns true on success, false if nothing
    /// was submitted (empty input or database error).
    pub fn submit_input(&mut self) -> bool {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return false;
        }

        // ── If editing an existing item, update it ──
        if let Some(ref item_id) = self.editing_item_id.clone() {
            let ok = match self.input_type {
                InputType::Todo => {
                    let parsed = starcatch_core::parser::parse_pipe_todo(&text);
                    let update = starcatch_core::db::TodoUpdate {
                        title: Some(parsed.title),
                        description: None,
                        priority: Some(parsed.priority),
                        due_date: parsed.due_date,
                        tags: if parsed.tags.is_empty() { None } else { Some(parsed.tags) },
                        project: parsed.project,
                    };
                    db::update_todo(&self.conn, &item_id, &update).is_ok()
                }
                InputType::Idea => {
                    let parsed = starcatch_core::parser::parse_pipe_idea(&text);
                    let update = starcatch_core::db::IdeaUpdate {
                        title: Some(parsed.title),
                        content: None, // preserve existing content
                        source: parsed.source,
                        tags: if parsed.tags.is_empty() { None } else { Some(parsed.tags) },
                        project: parsed.project,
                    };
                    db::update_idea(&self.conn, &item_id, &update).is_ok()
                }
                InputType::Log => {
                    let parsed = starcatch_core::parser::parse_pipe_log(&text);
                    let update = starcatch_core::db::LogUpdate {
                        content: Some(parsed.content),
                        mood: parsed.mood,
                        tags: if parsed.tags.is_empty() { None } else { Some(parsed.tags) },
                        project: parsed.project,
                    };
                    db::update_log(&self.conn, &item_id, &update).is_ok()
                }
            };
            if ok {
                self.set_status("Item updated");
            } else {
                self.set_status("Failed to update item");
            }
            self.editing_item_id = None;
            self.input_text.clear();
            self.input_cursor = 0;
            self.needs_refresh = true;
            return true;
        }

        // ── Otherwise insert new item ──
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
                if let Err(e) = db::insert_todo(&self.conn, &todo) {
                    self.set_status(&format!("Error: {}", e));
                    return false;
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
                if let Err(e) = db::insert_idea(&self.conn, &idea) {
                    self.set_status(&format!("Error: {}", e));
                    return false;
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
                if let Err(e) = db::insert_log(&self.conn, &log) {
                    self.set_status(&format!("Error: {}", e));
                    return false;
                }
                self.set_status("Log added");
            }
        }

        self.input_text.clear();
        self.input_cursor = 0;
        self.needs_refresh = true;
        true
    }

    pub fn toggle_selected_todo(&mut self) {
        if self.active_view != ActiveView::Todo {
            return;
        }
        if self.selected_index >= self.todos.len() {
            return;
        }
        let todo = &self.todos[self.selected_index];
        // Archived items are not toggleable — use archive action instead
        if todo.status == TodoStatus::Archived {
            return;
        }
        let new_status = match todo.status {
            TodoStatus::Pending => TodoStatus::Done,
            TodoStatus::Done => TodoStatus::Pending,
            TodoStatus::Archived => unreachable!(),
        };
        if let Err(e) = db::update_todo_status(&self.conn, &todo.id, &new_status) {
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
        if let Err(e) = db::update_todo_status(&self.conn, &id, &TodoStatus::Archived) {
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

        // First 'd': ask for confirmation
        if !self.confirm_delete {
            self.confirm_delete = true;
            let type_name = match self.active_view {
                ActiveView::Todo => "Todo",
                ActiveView::Idea => "Idea",
                ActiveView::Log => "Log",
            };
            self.set_status(&format!("Press d again to confirm deleting this {}", type_name));
            return;
        }

        // Second 'd': actually delete
        self.confirm_delete = false;

        let type_name = match self.active_view {
            ActiveView::Todo => {
                let id = self.todos[self.selected_index].id.clone();
                if let Err(e) = db::delete_todo(&self.conn, &id) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                "Todo"
            }
            ActiveView::Idea => {
                let id = self.ideas[self.selected_index].id.clone();
                if let Err(e) = db::delete_idea(&self.conn, &id) {
                    self.set_status(&format!("Error: {}", e));
                    return;
                }
                "Idea"
            }
            ActiveView::Log => {
                let id = self.logs[self.selected_index].id.clone();
                if let Err(e) = db::delete_log(&self.conn, &id) {
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

    /// Start editing the currently selected item.
    /// Pre-fills the input bar and enters editing mode.
    /// Uses " | " to separate raw title/content (left) from metadata (right)
    /// so that re-parsing never consumes parts of the title as keywords.
    pub fn start_edit(&mut self) {
        if self.current_list_len() == 0 || self.selected_index >= self.current_list_len() {
            return;
        }

        let (id, text) = match self.active_view {
            ActiveView::Todo => {
                let t = &self.todos[self.selected_index];
                let mut buf = t.title.clone();
                buf.push_str(" | ");
                let mut meta_parts: Vec<String> = Vec::new();
                if t.priority != Priority::P2 {
                    meta_parts.push(t.priority.to_string());
                }
                for tag in &t.tags {
                    meta_parts.push(format!("#{}", tag));
                }
                if let Some(ref due) = t.due_date {
                    meta_parts.push(format!("due:{}", due));
                }
                if let Some(ref proj) = t.project {
                    meta_parts.push(format!("project:{}", proj));
                }
                if !meta_parts.is_empty() {
                    buf.push_str(&meta_parts.join(" "));
                }
                (t.id.clone(), buf)
            }
            ActiveView::Idea => {
                let t = &self.ideas[self.selected_index];
                let mut buf = t.title.clone();
                buf.push_str(" | ");
                let mut meta_parts: Vec<String> = Vec::new();
                for tag in &t.tags {
                    meta_parts.push(format!("#{}", tag));
                }
                if let Some(ref src) = t.source {
                    meta_parts.push(format!("source:{}", src));
                }
                if let Some(ref proj) = t.project {
                    meta_parts.push(format!("project:{}", proj));
                }
                if !meta_parts.is_empty() {
                    buf.push_str(&meta_parts.join(" "));
                }
                (t.id.clone(), buf)
            }
            ActiveView::Log => {
                let t = &self.logs[self.selected_index];
                let mut buf = t.content.clone();
                buf.push_str(" | ");
                let mut meta_parts: Vec<String> = Vec::new();
                if let Some(ref mood) = t.mood {
                    meta_parts.push(format!("mood:{}", mood));
                }
                for tag in &t.tags {
                    meta_parts.push(format!("#{}", tag));
                }
                if let Some(ref proj) = t.project {
                    meta_parts.push(format!("project:{}", proj));
                }
                if !meta_parts.is_empty() {
                    buf.push_str(&meta_parts.join(" "));
                }
                (t.id.clone(), buf)
            }
        };

        self.input_type = match self.active_view {
            ActiveView::Todo => InputType::Todo,
            ActiveView::Idea => InputType::Idea,
            ActiveView::Log => InputType::Log,
        };

        self.editing = true;
        self.editing_item_id = Some(id);
        self.input_text = text;
        self.input_cursor = self.input_text.chars().count();
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
        self.status_auto_clear = true;
    }

    /// Called on Tick events. Clears the status after one tick delay,
    /// giving the user ~250ms to read it.
    pub fn tick_clear_status(&mut self) {
        if self.status_auto_clear {
            self.status_auto_clear = false;
            self.status_message = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_idx_to_byte_ascii() {
        assert_eq!(char_idx_to_byte("hello", 0), 0);
        assert_eq!(char_idx_to_byte("hello", 2), 2);
        assert_eq!(char_idx_to_byte("hello", 4), 4);
        // out of range returns len
        assert_eq!(char_idx_to_byte("hello", 10), 5);
    }

    #[test]
    fn test_char_idx_to_byte_unicode() {
        // "中" is 3 bytes
        let s = "a中b";
        assert_eq!(char_idx_to_byte(s, 0), 0); // 'a' at byte 0
        assert_eq!(char_idx_to_byte(s, 1), 1); // '中' at byte 1
        assert_eq!(char_idx_to_byte(s, 2), 4); // 'b' at byte 4
        assert_eq!(char_idx_to_byte(s, 3), 5); // end
    }

    #[test]
    fn test_char_idx_to_byte_emoji() {
        // "😀" is 4 bytes
        let s = "😀hi";
        assert_eq!(char_idx_to_byte(s, 0), 0);
        assert_eq!(char_idx_to_byte(s, 1), 4); // 'h' at byte 4
        assert_eq!(char_idx_to_byte(s, 2), 5); // 'i' at byte 5
    }

    #[test]
    fn test_safe_truncate_bytes_short() {
        assert_eq!(safe_truncate_bytes("hello", 10), "hello");
        assert_eq!(safe_truncate_bytes("", 5), "");
    }

    #[test]
    fn test_safe_truncate_bytes_long_ascii() {
        assert_eq!(safe_truncate_bytes("hello world!", 8), "hello...");
        // Exactly at boundary
        assert_eq!(safe_truncate_bytes("abc", 3), "abc");
        assert_eq!(safe_truncate_bytes("abcd", 3), "..."); // "..." fills all 3 bytes
    }

    #[test]
    fn test_safe_truncate_bytes_unicode_no_panic() {
        // 20 Chinese characters = 60 bytes; truncate at 30 bytes
        let s = "中".repeat(20);
        let result = safe_truncate_bytes(&s, 30);
        assert!(result.ends_with("..."));
        // 27 bytes of CJK + 3 dots = 30 bytes max
        assert!(result.len() <= 30);
    }

    #[test]
    fn test_safe_truncate_bytes_mixed() {
        // 'h'(1) 'e'(1) 'l'(1) 'l'(1) 'o'(1) '世'(3) '界'(3) '!'(1) = 12 bytes
        let s = "hello世界!";
        let result = safe_truncate_bytes(s, 8);
        assert_eq!(result, "hello...");
    }

    #[test]
    fn test_safe_truncate_bytes_emoji() {
        let s = "😀😀😀😀😀"; // 5 emoji × 4 bytes = 20 bytes
        let result = safe_truncate_bytes(s, 15); // room for 3 emoji (12 bytes) + "..."
        assert!(result.ends_with("..."));
        assert!(result.len() <= 15);
    }

    #[test]
    fn test_safe_truncate_bytes_char_boundary() {
        // "a" (1B) + "中" (3B) + "b" (1B) = 5 bytes
        let s = "a中b";
        // Budget 4: 5 > 4, cutoff=1, boundary=1 (after 'a'), result="a..."
        assert_eq!(safe_truncate_bytes(s, 4), "a...");
        // Budget 3: 5 > 3, cutoff=0, boundary=0, result="..."
        assert_eq!(safe_truncate_bytes(s, 3), "...");
        // Budget 5: exactly fits, no truncation
        assert_eq!(safe_truncate_bytes(s, 5), "a中b");
        // Budget 8: more than enough
        assert_eq!(safe_truncate_bytes(s, 8), "a中b");
    }
}
