use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::{App, InputType};
use crate::styles;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let input_type_str = match app.input_type {
        InputType::Todo => "[T] Todo",
        InputType::Idea => "[I] Idea",
        InputType::Log => "[L] Log",
    };

    let block = Block::default()
        .title(format!(" ⚡ {} | Ctrl+T/I/L to switch type ", input_type_str))
        .borders(Borders::ALL)
        .border_style(styles::title_style());

    // Show the input text with cursor
    let mut spans = Vec::new();

    // Show cursor at the right position
    let text_before_cursor = &app.input_text[..app.input_cursor];
    let text_after_cursor = &app.input_text[app.input_cursor..];

    // Build the input line
    if !app.input_text.is_empty() {
        spans.push(Span::styled(
            text_before_cursor.to_string(),
            styles::input_style(),
        ));
    }

    // Cursor character
    spans.push(Span::styled(
        "█",
        Style::default().fg(styles::THEME.primary),
    ));

    if !text_after_cursor.is_empty() {
        spans.push(Span::styled(
            text_after_cursor.to_string(),
            styles::input_style(),
        ));
    } else {
        // Placeholder text when empty
        let placeholder = match app.input_type {
            InputType::Todo => " Enter a todo... (P1 buy milk #shopping due:tomorrow)",
            InputType::Idea => " Enter an idea... (new feature idea #innovation from:reading)",
            InputType::Log => " Enter a log entry... (completed something #work mood:happy)",
        };
        if app.input_text.is_empty() {
            spans.push(Span::styled(placeholder, styles::dim_text_style()));
        }
    }

    let inner = block.inner(area);
    frame.render_widget(block, area);
    let input_line = Paragraph::new(Line::from(spans));
    frame.render_widget(input_line, inner);
}
