use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActiveView, App, InputType};
use crate::event::Event;

/// Handle an event, return true if the app should quit.
pub fn handle(app: &mut App, event: Event) -> std::io::Result<bool> {
    match event {
        Event::Tick => {
            // Periodic tasks can go here
        }
        Event::Resize(w, h) => {
            // Terminal resize - ratatui handles this automatically
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
    // Global shortcuts (take priority)
    match key.code {
        KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
            return true; // quit
        }
        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
            return true; // quit
        }
        KeyCode::Char('?') => {
            // Help - could show help overlay later
            app.set_status(
                "Tab:Switch  ↑↓:Nav  Enter:Toggle  d:Del  a:Archive  q:Quit",
            );
            return false;
        }
        _ => {}
    }

    // If user is typing in the input bar, handle input keys
    // We consider input active when there is text or when certain keys are pressed
    let is_input_key = matches!(
        key.code,
        KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Enter | KeyCode::Tab
    );

    if is_input_key {
        handle_input_key(app, key);
        return false;
    }

    // Navigation and action keys (only when not in input mode)
    match key.code {
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
        KeyCode::Enter => {
            app.toggle_selected_todo();
        }
        KeyCode::Char('d') => {
            app.delete_selected();
        }
        KeyCode::Char('a') => {
            app.archive_selected_todo();
        }
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
        _ => {}
    }

    false
}

fn handle_input_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(ch) => {
            app.input_text.insert(app.input_cursor, ch);
            app.input_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.input_cursor > 0 {
                app.input_cursor -= 1;
                app.input_text.remove(app.input_cursor);
            }
        }
        KeyCode::Enter => {
            app.submit_input();
        }
        KeyCode::Tab => {
            cycle_view(app);
        }
        _ => {}
    }

    // Handle Ctrl+T, Ctrl+I, Ctrl+L for input type switching
    if key.modifiers == KeyModifiers::CONTROL {
        match key.code {
            KeyCode::Char('t') => app.input_type = InputType::Todo,
            KeyCode::Char('i') => app.input_type = InputType::Idea,
            KeyCode::Char('l') => app.input_type = InputType::Log,
            _ => {}
        }
    }
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
