// 🖥️ Starcatch GUI — 浮动小面板风格
//
// 功能打磨 v2:
//   ✅ 焦点保持 — 提交后自动聚焦输入框
//   ✅ filter 切换即时刷新
//   ✅ Escape 关闭窗口
//   ✅ 删除功能 (Todo / Idea / Log)
//   ✅ Toast 反馈 — 操作成功有提示

use eframe::egui;

use crate::db;
use crate::models::*;

// ──────────────────────────────────────────────
//  Toast / 临时消息
// ──────────────────────────────────────────────

struct Toast {
    text: String,
    icon: &'static str,
    expires_at: f64, // seconds since start
}

// ──────────────────────────────────────────────
//  Tab & Filter
// ──────────────────────────────────────────────

#[derive(Default, Clone, Copy, PartialEq)]
enum PanelTab {
    #[default]
    Todo,
    Idea,
    Log,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum TodoFilter {
    #[default]
    Active,
    Pending,
    All,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum QuickKind {
    #[default]
    Todo,
    Idea,
    Log,
}

// ──────────────────────────────────────────────
//  App
// ──────────────────────────────────────────────

pub struct GuiApp {
    // Data
    db_path: String,
    needs_refresh: bool,
    start_time: f64,

    // Navigation
    tab: PanelTab,

    // Todo
    todos: Vec<Todo>,
    todo_filter: TodoFilter,

    // Idea
    ideas: Vec<Idea>,
    idea_days: i64,

    // Log
    logs: Vec<Log>,
    log_days: i64,

    // Quick input
    quick_input: String,
    quick_kind: QuickKind,
    focus_input: bool,

    // Feedback
    toast: Option<Toast>,
}

impl GuiApp {
    pub fn new(db_path: String) -> Self {
        let mut app = Self {
            db_path,
            needs_refresh: true,
            start_time: 0.0,
            tab: PanelTab::Todo,
            todos: vec![],
            todo_filter: TodoFilter::Active,
            ideas: vec![],
            idea_days: 7,
            logs: vec![],
            log_days: 1,
            quick_input: String::new(),
            quick_kind: QuickKind::Todo,
            focus_input: true,
            toast: None,
        };
        app.refresh_data();
        app
    }

    // ── Data ──

    fn refresh_data(&mut self) {
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        db::migrate(&conn).ok();

        self.todos = Vec::new();
        match self.todo_filter {
            TodoFilter::Active => {
                if let Ok(mut t) = db::list_todos(&conn, Some("pending")) {
                    self.todos.append(&mut t);
                }
                if let Ok(mut t) = db::list_todos(&conn, Some("done")) {
                    self.todos.append(&mut t);
                }
            }
            TodoFilter::Pending => {
                if let Ok(mut t) = db::list_todos(&conn, Some("pending")) {
                    self.todos.append(&mut t);
                }
            }
            TodoFilter::All => {
                for status in &["pending", "done", "archived"] {
                    if let Ok(mut t) = db::list_todos(&conn, Some(status)) {
                        self.todos.append(&mut t);
                    }
                }
            }
        }

        self.ideas = db::list_ideas(&conn, Some(self.idea_days)).unwrap_or_default();
        self.logs = db::list_logs(&conn, Some(self.log_days)).unwrap_or_default();
        self.needs_refresh = false;
    }

    /// Open a connection for a quick write operation
    fn with_db<F>(&self, f: F)
    where
        F: FnOnce(&rusqlite::Connection),
    {
        if let Ok(conn) = db::open(&self.db_path) {
            db::migrate(&conn).ok();
            f(&conn);
        }
    }

    // ── Toast ──

    fn show_toast(&mut self, icon: &'static str, text: String) {
        self.toast = Some(Toast {
            icon,
            text,
            expires_at: self.start_time + 2.5,
        });
    }

    // ── Actions ──

    fn quick_capture(&mut self) {
        let text = self.quick_input.trim().to_string();
        if text.is_empty() {
            return;
        }

        self.with_db(|conn| match self.quick_kind {
            QuickKind::Todo => {
                let (title, priority, due_date, tags) = parse_todo_input(&text);
                let todo = Todo {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    description: None,
                    priority,
                    status: TodoStatus::Pending,
                    due_date,
                    tags,
                    project: None,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                };
                db::insert_todo(conn, &todo).ok();
            }
            QuickKind::Idea => {
                let idea = Idea::new(text.clone());
                db::insert_idea(conn, &idea).ok();
            }
            QuickKind::Log => {
                let log = Log::new(text.clone());
                db::insert_log(conn, &log).ok();
            }
        });

        self.show_toast("✅", text);
        self.quick_input.clear();
        self.focus_input = true;
        self.needs_refresh = true;
    }

    fn delete_todo(&mut self, id: &str) {
        self.with_db(|conn| {
            conn.execute("DELETE FROM todos WHERE id = ?1", rusqlite::params![id])
                .ok();
        });
        self.needs_refresh = true;
    }

    fn delete_idea(&mut self, id: &str) {
        self.with_db(|conn| {
            conn.execute("DELETE FROM ideas WHERE id = ?1", rusqlite::params![id])
                .ok();
        });
        self.needs_refresh = true;
    }

    fn delete_log(&mut self, id: &str) {
        self.with_db(|conn| {
            conn.execute("DELETE FROM logs WHERE id = ?1", rusqlite::params![id])
                .ok();
        });
        self.needs_refresh = true;
    }

    fn toggle_todo(&mut self, id: &str, is_done: bool) {
        let new_status = if is_done {
            TodoStatus::Done
        } else {
            TodoStatus::Pending
        };
        self.with_db(|conn| {
            db::update_todo_status(conn, id, &new_status).ok();
        });
        self.needs_refresh = true;
    }

    fn archive_todo(&mut self, id: &str) {
        self.with_db(|conn| {
            db::update_todo_status(conn, id, &TodoStatus::Archived).ok();
        });
        self.needs_refresh = true;
    }
}

// ──────────────────────────────────────────────
//  Quick Input Parser
// ──────────────────────────────────────────────

/// Parse a quick-input string into (title, priority, due_date, tags).
///
/// Recognised tokens:
///   `P0` | `P1` | `P2` | `P3`  → priority override
///   `due:YYYY-MM-DD`            → due date
///   `#tag`                      → tag (strips trailing punctuation)
///
/// Everything else is joined back into the title.
fn parse_todo_input(raw: &str) -> (String, Priority, Option<String>, Vec<String>) {
    let mut priority = Priority::P2;
    let mut due_date: Option<String> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut title_parts: Vec<String> = Vec::new();

    let tokens: Vec<&str> = raw.split_whitespace().collect();
    let mut i = 0;

    while i < tokens.len() {
        let token = tokens[i];

        // Priority keyword
        if token == "P0" {
            priority = Priority::P0;
        } else if token == "P1" {
            priority = Priority::P1;
        } else if token == "P3" {
            priority = Priority::P3;
        } else if token == "P2" {
            priority = Priority::P2;
        }
        // due: prefix — value may be in the same token or the next
        else if let Some(due_val) = token.strip_prefix("due:") {
            let val = due_val.trim();
            if !val.is_empty() {
                due_date = Some(val.to_string());
            } else if i + 1 < tokens.len() {
                i += 1;
                due_date = Some(tokens[i].to_string());
            }
        }
        // #tag
        else if let Some(tag) = token.strip_prefix('#') {
            let cleaned: String = tag
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            if !cleaned.is_empty() {
                tags.push(cleaned);
            }
        }
        // plain title word
        else {
            title_parts.push(token.to_string());
        }

        i += 1;
    }

    let title = if title_parts.is_empty() {
        raw.to_string()
    } else {
        title_parts.join(" ")
    };

    (title, priority, due_date, tags)
}

// ──────────────────────────────────────────────
//  eframe::App
// ──────────────────────────────────────────────

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Track time for toast expiry
        self.start_time = ctx.input(|i| i.time);

        // Refresh if dirty
        if self.needs_refresh {
            self.refresh_data();
        }

        // ── Handle global keys ──
        // Enter to submit quick input
        let submit_quick = !self.quick_input.is_empty()
            && ctx.input(|i| i.key_pressed(egui::Key::Enter));

        if submit_quick {
            self.quick_capture();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // ── Top: tabs ──
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("⭐ Starcatch");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            // Tab bar
            ui.horizontal(|ui| {
                let changed = ui.selectable_value(&mut self.tab, PanelTab::Todo, "📋 Todo").changed()
                    || ui.selectable_value(&mut self.tab, PanelTab::Idea, "💭 Idea").changed()
                    || ui.selectable_value(&mut self.tab, PanelTab::Log, "📓 Log").changed();
                if changed {
                    self.needs_refresh = true;
                }
            });
        });

        // ── Center ──
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                PanelTab::Todo => self.show_todo_panel(ui),
                PanelTab::Idea => self.show_idea_panel(ui),
                PanelTab::Log => self.show_log_panel(ui),
            }
        });

        // ── Bottom: quick input ──
        egui::TopBottomPanel::bottom("quick_input").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("⚡");
                ui.selectable_value(&mut self.quick_kind, QuickKind::Todo, "📋");
                ui.selectable_value(&mut self.quick_kind, QuickKind::Idea, "💭");
                ui.selectable_value(&mut self.quick_kind, QuickKind::Log, "📓");

                let hint = match self.quick_kind {
                    QuickKind::Todo => "添加 Todo... (支持 P0-P3, due:)",
                    QuickKind::Idea => "记录 Idea...",
                    QuickKind::Log => "写 Log...",
                };

                let resp = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut self.quick_input)
                        .hint_text(hint)
                        .desired_width(f32::INFINITY),
                );

                // Auto-focus on first render
                if self.focus_input {
                    resp.request_focus();
                    self.focus_input = false;
                }

                if ui.button("➕").clicked() && !self.quick_input.is_empty() {
                    self.quick_capture();
                }
            });

            // ── Toast (shown above the quick input bar) ──
            if let Some(toast) = &self.toast {
                if self.start_time < toast.expires_at {
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(format!("{} {}", toast.icon, toast.text))
                            .color(egui::Color32::LIGHT_GREEN)
                            .size(13.0));
                    });
                } else {
                    self.toast = None;
                }
            }
        });
    }
}

// ──────────────────────────────────────────────
//  Todo Panel
// ──────────────────────────────────────────────

impl GuiApp {
    fn show_todo_panel(&mut self, ui: &mut egui::Ui) {
        // ── Filter chips ──
        ui.horizontal(|ui| {
            let changed = ui.selectable_value(&mut self.todo_filter, TodoFilter::Active, "📋 待办+完成").changed()
                || ui.selectable_value(&mut self.todo_filter, TodoFilter::Pending, "⬜ 仅待办").changed()
                || ui.selectable_value(&mut self.todo_filter, TodoFilter::All, "📦 全部").changed();
            if changed {
                self.needs_refresh = true;
            }
        });
        ui.separator();

        // ── Empty state ──
        if self.todos.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label("✨ 还没有 todo");
                ui.label("在底部的输入框添加吧〜");
            });
            return;
        }

        // ── Todo list ──
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut to_delete: Option<String> = None;
                let mut to_toggle: Option<(String, bool)> = None;
                let mut to_archive: Option<String> = None;
                for todo in &self.todos {
                    let id = todo.id.clone();
                    let is_done = todo.status == TodoStatus::Done;
                    let is_archived = todo.status == TodoStatus::Archived;

                    // Background tint for done/archived
                    let bg = if is_archived {
                        egui::Color32::from_rgba_premultiplied(40, 40, 40, 20)
                    } else if is_done {
                        egui::Color32::from_rgba_premultiplied(30, 60, 30, 20)
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    egui::Frame::NONE
                        .fill(bg)
                        .inner_margin(egui::Margin::symmetric(4, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Priority badge
                                let (p_color, p_label) = match todo.priority {
                                    Priority::P0 => (egui::Color32::RED, "P0"),
                                    Priority::P1 => (egui::Color32::YELLOW, "P1"),
                                    Priority::P2 => (egui::Color32::GREEN, "P2"),
                                    Priority::P3 => (egui::Color32::GRAY,  "P3"),
                                };
                                ui.colored_label(p_color, p_label);

                                // Checkbox
                                let mut checked = is_done;
                                if ui.checkbox(&mut checked, "").clicked() {
                                    to_toggle = Some((id.clone(), checked));
                                }

                                // Title
                                let title = if is_done {
                                    egui::RichText::new(&todo.title).strikethrough().color(egui::Color32::GRAY)
                                } else if is_archived {
                                    egui::RichText::new(&todo.title).color(egui::Color32::DARK_GRAY)
                                } else {
                                    egui::RichText::new(&todo.title)
                                };
                                ui.label(title);

                                // Spacer to push right items to the right
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    // Delete button
                                    if ui.button("🗑").clicked() {
                                        to_delete = Some(id.clone());
                                    }

                                    // Archive button (only for non-archived)
                                    if !is_archived && ui.button("📦").clicked() {
                                        to_archive = Some(id.clone());
                                    }

                                    // Due date
                                    if let Some(due) = &todo.due_date {
                                        ui.colored_label(egui::Color32::LIGHT_BLUE, due);
                                    }

                                    // Tags
                                    for tag in &todo.tags {
                                        ui.label(
                                            egui::RichText::new(format!("#{}", tag))
                                                .color(egui::Color32::LIGHT_BLUE)
                                                .size(11.0),
                                        );
                                    }
                                });
                            });
                        });
                }

                // Apply queued actions
                if let Some(id) = to_delete {
                    self.delete_todo(&id);
                }
                if let Some((id, done)) = to_toggle {
                    self.toggle_todo(&id, done);
                }
                if let Some(id) = to_archive {
                    self.archive_todo(&id);
                }
            });
    }
}

// ──────────────────────────────────────────────
//  Idea Panel
// ──────────────────────────────────────────────

impl GuiApp {
    fn show_idea_panel(&mut self, ui: &mut egui::Ui) {
        // ── Days selector ──
        ui.horizontal(|ui| {
            ui.label("最近");
            let changed = ui
                .add(egui::Slider::new(&mut self.idea_days, 1..=365).text("天"))
                .changed();
            if changed {
                self.needs_refresh = true;
            }
        });
        ui.separator();

        // ── Empty state ──
        if self.ideas.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label("💭 还没有 idea");
                ui.label("切到 Idea 模式，在底部记录吧〜");
            });
            return;
        }

        // ── Idea list ──
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut to_delete: Option<String> = None;

                for idea in &self.ideas {
                    let id = idea.id.clone();

                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(4, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("💡");
                                ui.colored_label(
                                    egui::Color32::LIGHT_GRAY,
                                    idea.created_at.format("%m-%d %H:%M").to_string(),
                                )
                                .on_hover_text(idea.created_at.to_rfc3339());
                                ui.label(&idea.title);

                                if let Some(source) = &idea.source {
                                    ui.label(
                                        egui::RichText::new(format!("({})", source))
                                            .color(egui::Color32::LIGHT_GRAY)
                                            .italics(),
                                    );
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑").clicked() {
                                        to_delete = Some(id.clone());
                                    }
                                    for tag in &idea.tags {
                                        ui.label(
                                            egui::RichText::new(format!("#{}", tag))
                                                .color(egui::Color32::LIGHT_BLUE)
                                                .size(11.0),
                                        );
                                    }
                                });
                            });
                        });
                }

                if let Some(id) = to_delete {
                    self.delete_idea(&id);
                }
            });
    }
}

// ──────────────────────────────────────────────
//  Log Panel
// ──────────────────────────────────────────────

impl GuiApp {
    fn show_log_panel(&mut self, ui: &mut egui::Ui) {
        // ── Days selector ──
        ui.horizontal(|ui| {
            ui.label("最近");
            let changed = ui
                .add(egui::Slider::new(&mut self.log_days, 1..=365).text("天"))
                .changed();
            if changed {
                self.needs_refresh = true;
            }
        });
        ui.separator();

        // ── Empty state ──
        if self.logs.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label("📓 还没有日志");
                ui.label("切到 Log 模式，记录今天的事吧〜");
            });
            return;
        }

        // ── Log list ──
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut to_delete: Option<String> = None;

                for log in &self.logs {
                    let id = log.id.clone();

                    egui::Frame::NONE
                        .inner_margin(egui::Margin::symmetric(4, 2))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("📝");
                                ui.colored_label(
                                    egui::Color32::LIGHT_GRAY,
                                    log.created_at.format("%m-%d %H:%M").to_string(),
                                );

                                if let Some(mood) = &log.mood {
                                    let mood_icon = match mood.as_str() {
                                        "happy" => "😊",
                                        "sad" => "😢",
                                        "excited" => "🤩",
                                        "angry" => "😤",
                                        "calm" => "😌",
                                        "tired" => "😴",
                                        m => m,
                                    };
                                    ui.label(mood_icon);
                                }

                                ui.label(&log.content);

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("🗑").clicked() {
                                        to_delete = Some(id.clone());
                                    }
                                });
                            });
                        });
                }

                if let Some(id) = to_delete {
                    self.delete_log(&id);
                }
            });
    }
}
