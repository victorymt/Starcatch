use crate::app::{ActiveView, App, InputType};
use crate::components;
use crate::styles;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: status bar (top 1), content (middle), input bar (bottom 3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // status bar
            Constraint::Min(1),     // content
            Constraint::Length(3),  // input bar
        ])
        .split(area);

    draw_status_bar(frame, chunks[0], app);
    draw_content(frame, chunks[1], app);
    draw_input_bar(frame, chunks[2], app);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    let view_name = match app.active_view {
        ActiveView::Todo => "Todo",
        ActiveView::Idea => "Idea",
        ActiveView::Log => "Log",
    };

    let input_mode = match app.input_type {
        InputType::Todo => "[T]",
        InputType::Idea => "[I]",
        InputType::Log => "[L]",
    };

    let mode_indicator = if app.editing { " EDIT " } else { " CMD " };
    let mode_style = if app.editing {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        styles::status_bar_style()
    };

    let status_text = match &app.status_message {
        Some(msg) => format!(" {}", msg),
        None => format!(" ⭐ Starcatch | {} {} | {}", view_name, input_mode, mode_indicator.trim()),
    };

    let line = Line::from(vec![
        Span::styled(mode_indicator, mode_style),
        Span::styled(status_text, styles::status_bar_style()),
        Span::styled(
            format!("  {}", app.current_list_len()),
            styles::dim_text_style(),
        ),
        Span::raw(" "),
    ]);

    let paragraph = Paragraph::new(Text::from(line));
    frame.render_widget(paragraph, area);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    // Split content into sidebar (left) and main area (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(1)])
        .split(area);

    components::sidebar::draw(frame, chunks[0], app);
    draw_main_area(frame, chunks[1], app);
}

fn draw_main_area(frame: &mut Frame, area: Rect, app: &App) {
    match app.active_view {
        ActiveView::Todo => components::todo_list::draw(frame, area, app),
        ActiveView::Idea => components::idea_list::draw(frame, area, app),
        ActiveView::Log => components::log_list::draw(frame, area, app),
    }
}

fn draw_input_bar(frame: &mut Frame, area: Rect, app: &App) {
    components::quick_input::draw(frame, area, app);
}
