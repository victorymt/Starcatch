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
    // Note: Ctrl+I = Tab (ASCII 0x09) in most terminals, so Tab is also mapped here.
    let ctrl_i_pressed = matches!((key.code, key.modifiers), (KeyCode::Tab, _) | (KeyCode::Char('i'), KeyModifiers::CONTROL));
    if key.modifiers == KeyModifiers::CONTROL || ctrl_i_pressed {
        match key.code {
            KeyCode::Char('t') => app.input_type = InputType::Todo,
            _ if ctrl_i_pressed => app.input_type = InputType::Idea,
            KeyCode::Char('i') => app.input_type = InputType::Idea,
            KeyCode::Char('l') => app.input_type = InputType::Log,
            _ => {}
        }
        // Don't return for Tab in command mode (Tab also cycles views)
        if !ctrl_i_pressed {
            return false;
        }
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
            app.toggle_selected_todo();
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
