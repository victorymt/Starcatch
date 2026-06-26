use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActiveView, App, InputType};
use crate::event::Event;

/// Handle an event, return true if the app should quit.
pub fn handle(app: &mut App, event: Event) -> std::io::Result<bool> {
    match event {
        Event::Tick => {}
        Event::Resize(w, h) => {
            let _ = (w, h);
        }
        Event::Key(key) => {
            if handle_key(app, key) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // ── Quit: always active ─────────────────────────────────────
    if matches!(key.code, KeyCode::Char('q'))
        && key.modifiers == KeyModifiers::NONE
    {
        return true;
    }
    if matches!((key.code, key.modifiers), (KeyCode::Char('c'), KeyModifiers::CONTROL)) {
        return true;
    }

    // ── Auto-clear status message & confirm on next interaction ─
    if app.status_message.is_some() {
        app.clear_status();
    }
    // Reset delete-confirm if pressing any key other than 'd'
    if app.confirm_delete && !matches!((key.code, key.modifiers), (KeyCode::Char('d'), KeyModifiers::NONE)) {
        app.confirm_delete = false;
    }

    // ── Ctrl+T/I/L: switch input type (only in command mode) ─────
    if !app.editing && key.modifiers == KeyModifiers::CONTROL {
        match key.code {
            KeyCode::Char('t') => app.input_type = InputType::Todo,
            KeyCode::Char('i') => app.input_type = InputType::Idea,
            KeyCode::Char('l') => app.input_type = InputType::Log,
            _ => {}
        }
        return false;
    }

    // ── Editing mode: / entered ────────────────────────────────
    if app.editing {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, _) => {
                app.editing = false;
                app.editing_item_id = None;
                app.input_text.clear();
                app.input_cursor = 0;
            }
            (KeyCode::Enter, _) => {
                app.submit_input();
                app.editing = false;
            }
            (KeyCode::Backspace, _) => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                    let byte_pos = crate::app::char_idx_to_byte(&app.input_text, app.input_cursor);
                    let end = byte_pos + app.input_text[byte_pos..].chars().next().map_or(0, |c| c.len_utf8());
                    app.input_text.drain(byte_pos..end);
                }
            }

            // ── Emacs / Ctrl keybindings ──
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                app.input_cursor = 0;
            }
            (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                app.input_cursor = app.input_text.chars().count();
            }
            (KeyCode::Char('k'), KeyModifiers::CONTROL) => {
                let byte_pos = crate::app::char_idx_to_byte(&app.input_text, app.input_cursor);
                app.input_text.truncate(byte_pos);
            }

            // ── Arrow key navigation ──
            (KeyCode::Left, _) => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                }
            }
            (KeyCode::Right, _) => {
                let max = app.input_text.chars().count();
                if app.input_cursor < max {
                    app.input_cursor += 1;
                }
            }

            // ── Regular character input ──
            (KeyCode::Char(ch), KeyModifiers::NONE) | (KeyCode::Char(ch), KeyModifiers::SHIFT) => {
                let byte_pos = crate::app::char_idx_to_byte(&app.input_text, app.input_cursor);
                app.input_text.insert(byte_pos, ch);
                app.input_cursor += 1;
            }
            _ => {}
        }
        return false;
    }

    // ── Command mode ───────────────────────────────────────────
    match key.code {
        // Enter editing mode
        KeyCode::Char('/') => {
            app.editing = true;
        }

        // Navigation
        KeyCode::Tab | KeyCode::Right => {
            cycle_view(app);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let max = app.current_list_len().saturating_sub(1);
            if app.selected_index < max {
                app.selected_index += 1;
            }
        }

        // Actions
        KeyCode::Enter => {
            match app.active_view {
                ActiveView::Todo => app.toggle_selected_todo(),
                ActiveView::Idea => {
                    if let Some(idea) = app.ideas.get(app.selected_index) {
                        let tags = if idea.tags.is_empty() { "".to_string() } else { format!(" #{}", idea.tags.join(" #")) };
                        app.set_status(&format!("💡 {} from:{} project:{}{}", idea.title, idea.source.as_deref().unwrap_or("-"), idea.project.as_deref().unwrap_or("-"), tags));
                    }
                }
                ActiveView::Log => {
                    if let Some(log) = app.logs.get(app.selected_index) {
                        let mood = log.mood.as_deref().unwrap_or("-");
                        let preview = if log.content.len() > 60 { format!("{}...", &log.content[..57]) } else { log.content.clone() };
                        app.set_status(&format!("📝 mood:{} {}", mood, preview));
                    }
                }
            }
        }
        KeyCode::Char('e') => {
            app.start_edit();
        }
        KeyCode::Char('d') => {
            app.delete_selected();
        }
        KeyCode::Char('a') => {
            app.archive_selected_todo();
        }

        // Quick view switching
        KeyCode::Char('1') => {
            app.active_view = ActiveView::Todo;
            app.selected_index = 0;
            app.refresh_current_list();
        }
        KeyCode::Char('2') => {
            app.active_view = ActiveView::Idea;
            app.selected_index = 0;
            app.refresh_current_list();
        }
        KeyCode::Char('3') => {
            app.active_view = ActiveView::Log;
            app.selected_index = 0;
            app.refresh_current_list();
        }

        // Help
        KeyCode::Char('?') => {
            app.set_status(
                "/:Input  Tab:Switch  ↑↓/jk:Nav  Enter:Toggle  e:Edit  d:Del  a:Archive  1-3:View  q:Quit",
            );
        }

        _ => {}
    }

    false
}

fn cycle_view(app: &mut App) {
    app.active_view = match app.active_view {
        ActiveView::Todo => ActiveView::Idea,
        ActiveView::Idea => ActiveView::Log,
        ActiveView::Log => ActiveView::Todo,
    };
    app.selected_index = 0;
    app.refresh_current_list();
}
