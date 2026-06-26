use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Padding, Paragraph};
use ratatui::Frame;
use ratatui::prelude::Stylize;

use crate::app::App;
use crate::styles;
use starcatch_core::models::TodoStatus;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" 📋 Todo ")
        .borders(Borders::ALL)
        .border_style(styles::title_style())
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.todos.is_empty() {
        let empty_msg = Paragraph::new(Line::from(vec![Span::styled(
            " No todos yet. Type below to add one!",
            styles::dim_text_style(),
        )]))
        .block(Block::default().padding(Padding::new(1, 0, 1, 0)));
        frame.render_widget(empty_msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .todos
        .iter()
        .enumerate()
        .map(|(_i, todo)| {
            let is_done = todo.status == TodoStatus::Done;
            let is_archived = todo.status == TodoStatus::Archived;

            let checkbox = match todo.status {
                TodoStatus::Pending => "⬜ ",
                TodoStatus::Done => "✅ ",
                TodoStatus::Archived => "📦 ",
            };

            let priority_str = todo.priority.icon();
            let priority_color = styles::priority_color(&todo.priority);

            let title_style = if is_done {
                Style::default()
                    .fg(styles::THEME.success)
                    .crossed_out()
            } else if is_archived {
                styles::dim_text_style()
            } else {
                styles::item_style()
            };

            let mut spans = vec![
                Span::styled(checkbox, title_style),
                Span::styled(priority_str, Style::default().fg(priority_color)),
                Span::raw(" "),
                Span::styled(&todo.title, title_style),
            ];

            // Add tags
            for tag in &todo.tags {
                spans.push(Span::styled(
                    format!(" #{}", tag),
                    styles::dim_text_style(),
                ));
            }

            // Add due date
            if let Some(ref due) = todo.due_date {
                spans.push(Span::styled(
                    format!(" due:{}", due),
                    styles::dim_text_style(),
                ));
            }

            // Add project
            if let Some(ref proj) = todo.project {
                spans.push(Span::styled(
                    format!(" [{}]", proj),
                    Style::default().fg(styles::THEME.warning),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(styles::selected_item_style())
        .highlight_symbol("▸ ");

    let mut state = ratatui::widgets::ListState::default()
        .with_selected(Some(app.selected_index));
    frame.render_stateful_widget(list, inner, &mut state);

    // Show key hints at bottom
    let hint = Line::from(vec![Span::styled(
        " Enter:toggle  e:edit  d:del  a:archive  1-3:view",
        styles::dim_text_style(),
    )]);
    if area.height > 3 {
        let hint_area = Rect {
            x: area.x,
            y: area.y + area.height - 1,
            width: area.width,
            height: 1,
        };
        frame.render_widget(Paragraph::new(hint), hint_area);
    }
}
