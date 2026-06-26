use ratatui::style::{Color, Modifier, Style};

/// Theme colors for the TUI
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text: Color,
    pub text_dim: Color,
    pub background: Color,
    pub selected_bg: Color,
    pub border: Color,
    pub input_bg: Color,
}

pub const THEME: Theme = Theme {
    primary: Color::Cyan,
    secondary: Color::Blue,
    accent: Color::Magenta,
    success: Color::Green,
    warning: Color::Yellow,
    error: Color::Red,
    text: Color::White,
    text_dim: Color::DarkGray,
    background: Color::Reset,
    selected_bg: Color::DarkGray,
    border: Color::Gray,
    input_bg: Color::Black,
};

pub fn item_style() -> Style {
    Style::default().fg(THEME.text)
}

pub fn selected_item_style() -> Style {
    Style::default()
        .fg(THEME.primary)
        .bg(THEME.selected_bg)
        .add_modifier(Modifier::BOLD)
}

pub fn dim_text_style() -> Style {
    Style::default().fg(THEME.text_dim)
}

pub fn title_style() -> Style {
    Style::default()
        .fg(THEME.primary)
        .add_modifier(Modifier::BOLD)
}

pub fn status_bar_style() -> Style {
    Style::default()
        .fg(THEME.background)
        .bg(THEME.primary)
        .add_modifier(Modifier::BOLD)
}

pub fn input_style() -> Style {
    Style::default().fg(THEME.text).bg(THEME.input_bg)
}

pub fn priority_color(priority: &starcatch_core::models::Priority) -> Color {
    match priority {
        starcatch_core::models::Priority::P0 => Color::Red,
        starcatch_core::models::Priority::P1 => Color::Yellow,
        starcatch_core::models::Priority::P2 => Color::Green,
        starcatch_core::models::Priority::P3 => Color::DarkGray,
    }
}

pub fn status_color(status: &starcatch_core::models::TodoStatus) -> Color {
    match status {
        starcatch_core::models::TodoStatus::Pending => Color::Yellow,
        starcatch_core::models::TodoStatus::Done => Color::Green,
        starcatch_core::models::TodoStatus::Archived => Color::DarkGray,
    }
}
