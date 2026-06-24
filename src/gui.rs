// 🖥️ Starcatch GUI — 浮动小面板风格 (gui-panel 分支)
//
// 一个紧凑的浮动窗口，左侧 icon tabs 切换 Todo / Idea / Log
// 右侧内容区展示列表，底部有快速输入框

use eframe::egui;

use crate::db;
use crate::models::*;

#[derive(Default)]
enum PanelTab {
    #[default]
    Todo,
    Idea,
    Log,
}

#[derive(Default)]
pub struct GuiApp {
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

    db_path: String,
    needs_refresh: bool,
}

#[derive(PartialEq)]
enum QuickKind {
    Todo,
    Idea,
    Log,
}

impl Default for QuickKind {
    fn default() -> Self {
        Self::Todo
    }
}

#[derive(Default, PartialEq)]
enum TodoFilter {
    #[default]
    Active,   // pending + done
    Pending,
    All,
}

impl GuiApp {
    pub fn new(db_path: String) -> Self {
        let mut app = Self {
            db_path,
            ..Default::default()
        };
        app.refresh_data();
        app
    }

    fn refresh_data(&mut self) {
        let conn = match db::open(&self.db_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        db::migrate(&conn).ok();

        // Load todos
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
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.needs_refresh {
            self.refresh_data();
        }

        // ── Top bar: icon tabs ──
        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("⭐ Starcatch");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("✕").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

            ui.horizontal(|ui| {
                let tabs = [("📋", "Todo"), ("💭", "Idea"), ("📓", "Log")];
                let tab_values = [PanelTab::Todo, PanelTab::Idea, PanelTab::Log];

                for (i, (icon, name)) in tabs.iter().enumerate() {
                    let is_selected = std::mem::discriminant(&self.tab) == std::mem::discriminant(&tab_values[i]);

                    if ui.selectable_label(is_selected, format!("{} {}", icon, name)).clicked() {
                        // SAFETY: i is always 0,1,2 matching the enum variants
                        self.tab = match i {
                            0 => PanelTab::Todo,
                            1 => PanelTab::Idea,
                            _ => PanelTab::Log,
                        };
                        self.needs_refresh = true;
                    }
                }
            });
        });

        // ── Center: content ──
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.tab {
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
                    QuickKind::Todo => "New todo...",
                    QuickKind::Idea => "New idea...",
                    QuickKind::Log => "New log...",
                };
                let resp = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut self.quick_input)
                        .hint_text(hint)
                        .desired_width(f32::INFINITY),
                );

                let submit = ui.button("+").clicked() || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if submit && !self.quick_input.is_empty() {
                    let conn = db::open(&self.db_path).ok();
                    if let Some(conn) = conn {
                        db::migrate(&conn).ok();
                        match self.quick_kind {
                            QuickKind::Todo => {
                                let todo = Todo {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    title: self.quick_input.clone(),
                                    description: None,
                                    priority: Priority::P2,
                                    status: TodoStatus::Pending,
                                    due_date: None,
                                    tags: vec![],
                                    project: None,
                                    created_at: chrono::Utc::now(),
                                    updated_at: chrono::Utc::now(),
                                };
                                db::insert_todo(&conn, &todo).ok();
                            }
                            QuickKind::Idea => {
                                let idea = Idea::new(self.quick_input.clone());
                                db::insert_idea(&conn, &idea).ok();
                            }
                            QuickKind::Log => {
                                let log = Log::new(self.quick_input.clone());
                                db::insert_log(&conn, &log).ok();
                            }
                        }
                    }
                    self.quick_input.clear();
                    self.needs_refresh = true;
                }

                // Focus the input when switching tabs
                if resp.gained_focus() {
                    // Keep focus
                }
            });
        });
    }
}

impl GuiApp {
    fn show_todo_panel(&mut self, ui: &mut egui::Ui) {
        // Filter buttons
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.todo_filter, TodoFilter::Active, "📋 待办+完成");
            ui.selectable_value(&mut self.todo_filter, TodoFilter::Pending, "⬜ 仅待办");
            ui.selectable_value(&mut self.todo_filter, TodoFilter::All, "📦 全部");
        });
        ui.separator();

        if self.todos.is_empty() {
            ui.label("✨ 还没有 todo，在底部输入框添加吧〜");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            let conn = db::open(&self.db_path).ok();
            let mut to_update: Option<(String, TodoStatus)> = None;

            for todo in &self.todos {
                ui.horizontal(|ui| {
                    // Priority color dot
                    let color = match todo.priority {
                        Priority::P0 => egui::Color32::RED,
                        Priority::P1 => egui::Color32::YELLOW,
                        Priority::P2 => egui::Color32::GREEN,
                        Priority::P3 => egui::Color32::GRAY,
                    };
                    ui.colored_label(color, &todo.priority.to_string());

                    // Checkbox for done toggle
                    let is_done = todo.status == TodoStatus::Done;
                    let mut checked = is_done;
                    if ui.checkbox(&mut checked, "").clicked() {
                        let new_status = if checked { TodoStatus::Done } else { TodoStatus::Pending };
                        to_update = Some((todo.id.clone(), new_status));
                    }

                    // Title
                    if is_done {
                        ui.label(egui::RichText::new(&todo.title).strikethrough().color(egui::Color32::GRAY));
                    } else {
                        ui.label(&todo.title);
                    }

                    // Due date
                    if let Some(due) = &todo.due_date {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(egui::Color32::LIGHT_BLUE, due);
                        });
                    }

                    // Tags
                    if !todo.tags.is_empty() {
                        for tag in &todo.tags {
                            ui.label(
                                egui::RichText::new(format!("#{}", tag))
                                    .color(egui::Color32::LIGHT_BLUE)
                                    .size(11.0),
                            );
                        }
                    }
                });
            }

            // Apply updates
            if let Some((id, status)) = to_update {
                conn.as_ref().and_then(|c| db::update_todo_status(c, &id, &status).ok());
                self.needs_refresh = true;
            }
        });
    }

    fn show_idea_panel(&mut self, ui: &mut egui::Ui) {
        // Days selector
        ui.horizontal(|ui| {
            ui.label("显示最近:");
            ui.add(egui::Slider::new(&mut self.idea_days, 1..=365).text("天"));
        });
        if ui.button("刷新").clicked() {
            self.needs_refresh = true;
        }
        ui.separator();

        if self.ideas.is_empty() {
            ui.label("💭 还没有 idea 呢〜");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for idea in &self.ideas {
                ui.horizontal(|ui| {
                    ui.label("💡");
                    ui.label(idea.created_at.format("%m-%d %H:%M").to_string())
                        .on_hover_text(idea.created_at.to_rfc3339());
                    ui.label(&idea.title);
                    if let Some(source) = &idea.source {
                        ui.label(
                            egui::RichText::new(format!("({})", source))
                                .color(egui::Color32::LIGHT_GRAY)
                                .italics(),
                        );
                    }
                    for tag in &idea.tags {
                        ui.label(
                            egui::RichText::new(format!("#{}", tag))
                                .color(egui::Color32::LIGHT_BLUE)
                                .size(11.0),
                        );
                    }
                });
            }
        });
    }

    fn show_log_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("显示最近:");
            ui.add(egui::Slider::new(&mut self.log_days, 1..=365).text("天"));
        });
        if ui.button("刷新").clicked() {
            self.needs_refresh = true;
        }
        ui.separator();

        if self.logs.is_empty() {
            ui.label("📓 还没有日志〜");
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for log in &self.logs {
                ui.horizontal(|ui| {
                    ui.label("📝");
                    ui.label(log.created_at.format("%m-%d %H:%M").to_string());
                    if let Some(mood) = &log.mood {
                        ui.label(
                            egui::RichText::new(format!("[{}]", mood))
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    }
                    ui.label(&log.content);
                });
            }
        });
    }
}
