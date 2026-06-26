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

    // ── Ctrl+T/I/L: switch input type (always active) ────────────
    if key.modifiers == KeyModifiers::CONTROL {
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
        match key.code {
            KeyCode::Esc => {
                app.editing = false;
                app.input_text.clear();
                app.input_cursor = 0;
            }
            KeyCode::Enter => {
                app.submit_input();
                app.editing = false;
            }
            KeyCode::Backspace => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                    app.input_text.remove(app.input_cursor);
                }
            }
            KeyCode::Char(ch) => {
                app.input_text.insert(app.input_cursor, ch);
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
            app.toggle_selected_todo();
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
                "/:Input  Tab:Switch  ↑↓:Nav  Enter:Toggle  d:Del  a:Archive  q:Quit",
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
