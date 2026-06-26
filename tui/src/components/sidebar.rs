use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph};
use ratatui::Frame;

use crate::app::{ActiveView, App, InputType};
use crate::styles;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" Views ")
        .borders(Borders::ALL)
        .border_style(styles::dim_text_style())
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);

    // Render the block background
    frame.render_widget(block, area);

    let items = vec![
        view_line(app, ActiveView::Todo, " 📋 Todo", app.todos.len()),
        view_line(app, ActiveView::Idea, " 💭 Idea", app.ideas.len()),
        view_line(app, ActiveView::Log, " 📓 Log", app.logs.len()),
    ];

    let list_items: Vec<ListItem> = items
        .into_iter()
        .map(|line| ListItem::new(line))
        .collect();

    let list = List::new(list_items)
        .highlight_style(styles::selected_item_style())
        .highlight_symbol("▸ ");

    // Determine which index is selected
    let list_index = match app.active_view {
        ActiveView::Todo => Some(0),
        ActiveView::Idea => Some(1),
        ActiveView::Log => Some(2),
    };

    let mut state = ListState::default().with_selected(list_index);
    frame.render_stateful_widget(list, inner, &mut state);

    // Draw input type indicator below the view list
    let input_label = match app.input_type {
        InputType::Todo => Span::styled(" [T] Todo ", styles::title_style()),
        InputType::Idea => Span::styled(" [I] Idea ", styles::title_style()),
        InputType::Log => Span::styled(" [L] Log ", styles::title_style()),
    };

    // Draw input type label below the list border
    let help_y = area.y + area.height.saturating_sub(1);
    if help_y > inner.y + 4 {
        let type_area = Rect {
            x: area.x + 1,
            y: inner.y + 4,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let type_widget = Paragraph::new(Line::from(vec![
            Span::styled("── ", styles::dim_text_style()),
            Span::styled("Input Type", styles::dim_text_style()),
            Span::styled(" ──", styles::dim_text_style()),
        ]));
        frame.render_widget(type_widget, type_area);

        let type_val_area = Rect {
            x: area.x + 1,
            y: inner.y + 5,
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let type_val_widget = Paragraph::new(input_label);
        frame.render_widget(type_val_widget, type_val_area);

        let hint_area = Rect {
            x: area.x + 1,
            y: (inner.y + 6).min(area.y + area.height.saturating_sub(3)),
            width: area.width.saturating_sub(2),
            height: 1,
        };
        let hint_widget = Paragraph::new(Line::from(vec![Span::styled(
            "Ctrl+T/I/L",
            styles::dim_text_style(),
        )]));
        frame.render_widget(hint_widget, hint_area);
    }
}

fn view_line(app: &App, view: ActiveView, label: &str, count: usize) -> Line<'static> {
    let is_active = app.active_view == view;
    let icon = if is_active { "●" } else { "○" };

    let icon_style = if is_active {
        styles::title_style()
    } else {
        styles::dim_text_style()
    };

    let count_style = if count > 0 {
        styles::item_style()
    } else {
        styles::dim_text_style()
    };

    Line::from(vec![
        Span::styled(format!("{} {}", icon, label), icon_style),
        Span::raw(" "),
        Span::styled(format!("({})", count), count_style),
    ])
}
