use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Padding, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::styles;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" 💭 Ideas ")
        .borders(Borders::ALL)
        .border_style(styles::title_style())
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.ideas.is_empty() {
        let empty_msg = Paragraph::new(Line::from(vec![Span::styled(
            " No ideas yet. Type below to add one!",
            styles::dim_text_style(),
        )]))
        .block(Block::default().padding(Padding::new(1, 0, 1, 0)));
        frame.render_widget(empty_msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .ideas
        .iter()
        .map(|idea| {
            let mut spans = vec![Span::styled("💡 ", styles::item_style())];

            spans.push(Span::styled(&idea.title, styles::item_style()));

            // Show source if present
            if let Some(ref source) = idea.source {
                spans.push(Span::styled(
                    format!(" from:{}", source),
                    Style::default().fg(styles::THEME.accent),
                ));
            }

            // Add tags
            for tag in &idea.tags {
                spans.push(Span::styled(
                    format!(" #{}", tag),
                    styles::dim_text_style(),
                ));
            }

            // Add project
            if let Some(ref proj) = idea.project {
                spans.push(Span::styled(
                    format!(" [{}]", proj),
                    Style::default().fg(styles::THEME.warning),
                ));
            }

            // Show created time relative
            let duration = chrono::Utc::now() - idea.created_at;
            let time_str = if duration.num_minutes() <= 0 {
                " just now".to_string()
            } else if duration.num_minutes() < 60 {
                format!(" {}m ago", duration.num_minutes())
            } else if duration.num_hours() < 24 {
                format!(" {}h ago", duration.num_hours())
            } else {
                format!(" {}d ago", duration.num_days())
            };
            spans.push(Span::styled(time_str, styles::dim_text_style()));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(styles::selected_item_style())
        .highlight_symbol("▸ ");

    let mut state =
        ratatui::widgets::ListState::default().with_selected(Some(app.selected_index));
    frame.render_stateful_widget(list, inner, &mut state);
}
