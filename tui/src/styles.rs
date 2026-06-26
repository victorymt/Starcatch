use ratatui::style::{Color, Modifier, Style};

/// Theme colors for the TUI
pub struct Theme {
    pub primary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub text: Color,
    pub text_dim: Color,
    pub selected_bg: Color,
    pub input_bg: Color,
}

pub const THEME: Theme = Theme {
    primary: Color::Cyan,
    accent: Color::Magenta,
    success: Color::Green,
    warning: Color::Yellow,
    text: Color::White,
    text_dim: Color::DarkGray,
    selected_bg: Color::DarkGray,
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
        .fg(Color::Black)
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
