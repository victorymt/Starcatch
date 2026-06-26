use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::styles;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let total_todos = app.todos.len();
    let total_ideas = app.ideas.len();
    let total_logs = app.logs.len();

    let status = Line::from(vec![
        Span::styled(
            format!(" 📊 {} todos | {} ideas | {} logs ", total_todos, total_ideas, total_logs),
            styles::dim_text_style(),
        ),
    ]);

    frame.render_widget(Paragraph::new(status), area);
}
