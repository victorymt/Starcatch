use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Padding, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::styles;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" 📓 Logs ")
        .borders(Borders::ALL)
        .border_style(styles::title_style())
        .padding(Padding::horizontal(1));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.logs.is_empty() {
        let empty_msg = Paragraph::new(Line::from(vec![Span::styled(
            " No logs yet. Type below to add one!",
            styles::dim_text_style(),
        )]))
        .block(Block::default().padding(Padding::new(1, 0, 1, 0)));
        frame.render_widget(empty_msg, inner);
        return;
    }

    let items: Vec<ListItem> = app
        .logs
        .iter()
        .enumerate()
        .map(|(_i, log)| {
            let content_preview = if log.content.len() > 80 {
                format!("{}...", &log.content[..77])
            } else {
                log.content.clone()
            };

            let mut spans = vec![Span::styled("📝 ", styles::item_style())];

            // Mood emoji
            if let Some(ref mood) = log.mood {
                let mood_icon = match mood.to_lowercase().as_str() {
                    "happy" | "开心" | "good" => "😊 ",
                    "sad" | "难过" | "bad" => "😢 ",
                    "angry" | "生气" => "😠 ",
                    "tired" | "累" => "😴 ",
                    "excited" | "兴奋" => "🎉 ",
                    "calm" | "平静" => "😌 ",
                    _ => "📓 ",
                };
                spans.push(Span::styled(mood_icon, styles::item_style()));
            }

            spans.push(Span::styled(content_preview, styles::item_style()));

            // Tags
            for tag in &log.tags {
                spans.push(Span::styled(
                    format!(" #{}", tag),
                    styles::dim_text_style(),
                ));
            }

            // Project
            if let Some(ref proj) = log.project {
                spans.push(Span::styled(
                    format!(" [{}]", proj),
                    Style::default().fg(styles::THEME.warning),
                ));
            }

            // Time
            let duration = chrono::Utc::now() - log.created_at;
            let time_str = if duration.num_hours() < 24 {
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
