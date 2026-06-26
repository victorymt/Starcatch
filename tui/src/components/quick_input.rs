use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, InputType, char_idx_to_byte};
use crate::styles;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if app.editing {
        draw_editing_bar(frame, area, app);
    } else {
        draw_command_hint(frame, area, app);
    }
}

fn draw_command_hint(frame: &mut Frame, area: Rect, app: &App) {
    let input_type_str = match app.input_type {
        InputType::Todo => "[T] Todo",
        InputType::Idea => "[I] Idea",
        InputType::Log => "[L] Log",
    };

    let block = Block::default()
        .title(format!(" ⚡ Press / to input | {} | Ctrl+T/I/L to switch ", input_type_str))
        .borders(Borders::ALL)
        .border_style(styles::dim_text_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let placeholder = match app.input_type {
        InputType::Todo => "  '/' then type: P1 buy milk #shopping due:tomorrow",
        InputType::Idea => "  '/' then type: new feature idea #innovation from:reading",
        InputType::Log => "  '/' then type: completed something #work mood:happy",
    };

    let hint = Paragraph::new(Line::from(vec![
        Span::styled(placeholder, styles::dim_text_style())
    ]));
    frame.render_widget(hint, inner);
}

fn draw_editing_bar(frame: &mut Frame, area: Rect, app: &App) {
    let input_type_str = match app.input_type {
        InputType::Todo => "[T] Todo",
        InputType::Idea => "[I] Idea",
        InputType::Log => "[L] Log",
    };

    let mode_label = if app.editing_item_id.is_some() { "EDIT" } else { "ADD" };

    let block = Block::default()
        .title(format!(" ⚡ {} {} | Esc:cancel Enter:submit ", mode_label, input_type_str))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(styles::THEME.success));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut spans = Vec::new();

    // Convert char-index cursor to byte offset for safe string slicing
    let byte_cursor = char_idx_to_byte(&app.input_text, app.input_cursor);

    // Text before cursor
    if byte_cursor > 0 {
        spans.push(Span::styled(
            app.input_text[..byte_cursor].to_string(),
            styles::input_style(),
        ));
    }

    // Block cursor — sits ON the character at cursor position
    let remaining = &app.input_text[byte_cursor..];
    if let Some(c) = remaining.chars().next() {
        let char_len = c.len_utf8();
        spans.push(Span::styled(
            c.to_string(),
            Style::default().fg(styles::THEME.input_bg).bg(styles::THEME.primary),
        ));
        // Text after the character under cursor
        if char_len < remaining.len() {
            spans.push(Span::styled(
                remaining[char_len..].to_string(),
                styles::input_style(),
            ));
        }
    } else if !app.input_text.is_empty() {
        // At end of text — show solid block cursor
        spans.push(Span::styled(
            "▉",
            Style::default().fg(styles::THEME.primary).bg(styles::THEME.input_bg),
        ));
    }

    // When empty, show cursor then placeholder
    if app.input_text.is_empty() {
        // Block cursor at position 0
        spans.push(Span::styled(
            "▉",
            Style::default().fg(styles::THEME.primary).bg(styles::THEME.input_bg),
        ));
        let placeholder = match app.input_type {
            InputType::Todo => " Type your todo here... (P1, #tag, due:, project:)",
            InputType::Idea => " Type your idea here... (from:, #tag, project:)",
            InputType::Log => " Type your log here... (mood:, #tag, project:)",
        };
        spans.push(Span::styled(placeholder, styles::dim_text_style()));
    }

    let input_line = Paragraph::new(Line::from(spans));
    frame.render_widget(input_line, inner);
}
