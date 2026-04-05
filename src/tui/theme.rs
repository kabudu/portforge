use ratatui::style::{Color, Modifier, Style};

/// PortForge dark theme color palette.
/// Inspired by modern developer tools (GitHub Dark, VS Code Dark+).
pub struct Theme;

impl Theme {
    // ─── Background Colors ───
    pub const BG_PRIMARY: Color = Color::Rgb(13, 17, 23); // #0d1117 deep navy
    pub const BG_SURFACE: Color = Color::Rgb(22, 27, 34); // #161b22 elevated
    pub const BG_HIGHLIGHT: Color = Color::Rgb(33, 38, 45); // #21262d selection
    pub const BG_OVERLAY: Color = Color::Rgb(22, 27, 34); // #161b22 modals

    // ─── Border Colors ───
    pub const BORDER: Color = Color::Rgb(48, 54, 61); // #30363d
    pub const BORDER_FOCUS: Color = Color::Rgb(88, 166, 255); // #58a6ff

    // ─── Text Colors ───
    pub const TEXT_PRIMARY: Color = Color::Rgb(230, 237, 243); // #e6edf3
    pub const TEXT_SECONDARY: Color = Color::Rgb(139, 148, 158); // #8b949e
    pub const TEXT_MUTED: Color = Color::Rgb(110, 118, 129); // #6e7681
    pub const TEXT_INVERSE: Color = Color::Rgb(13, 17, 23); // #0d1117

    // ─── Status Colors ───
    pub const HEALTHY: Color = Color::Rgb(63, 185, 80); // #3fb950 green
    pub const WARNING: Color = Color::Rgb(210, 153, 34); // #d29922 amber
    pub const ERROR: Color = Color::Rgb(248, 81, 73); // #f85149 red
    pub const INFO: Color = Color::Rgb(88, 166, 255); // #58a6ff blue

    // ─── Accent Colors ───
    pub const ACCENT_BLUE: Color = Color::Rgb(88, 166, 255); // #58a6ff
    pub const ACCENT_PURPLE: Color = Color::Rgb(188, 140, 255); // #bc8cff
    pub const ACCENT_CYAN: Color = Color::Rgb(63, 214, 207); // #3fd6cf
    pub const ACCENT_ORANGE: Color = Color::Rgb(219, 171, 9); // #dbab09

    // ─── Semantic Styles ───

    pub fn title() -> Style {
        Style::default()
            .fg(Self::ACCENT_BLUE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header() -> Style {
        Style::default()
            .fg(Self::TEXT_PRIMARY)
            .bg(Self::BG_SURFACE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn row_normal() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }

    pub fn row_selected() -> Style {
        Style::default()
            .fg(Self::TEXT_PRIMARY)
            .bg(Self::BG_HIGHLIGHT)
            .add_modifier(Modifier::BOLD)
    }

    pub fn row_alt() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY).bg(Self::BG_SURFACE)
    }

    pub fn healthy() -> Style {
        Style::default()
            .fg(Self::HEALTHY)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning() -> Style {
        Style::default()
            .fg(Self::WARNING)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error() -> Style {
        Style::default()
            .fg(Self::ERROR)
            .add_modifier(Modifier::BOLD)
    }

    pub fn info() -> Style {
        Style::default().fg(Self::INFO)
    }

    pub fn muted() -> Style {
        Style::default().fg(Self::TEXT_MUTED)
    }

    pub fn border() -> Style {
        Style::default().fg(Self::BORDER)
    }

    pub fn border_focus() -> Style {
        Style::default().fg(Self::BORDER_FOCUS)
    }

    pub fn status_bar() -> Style {
        Style::default()
            .fg(Self::TEXT_SECONDARY)
            .bg(Self::BG_SURFACE)
    }

    pub fn key_hint() -> Style {
        Style::default()
            .fg(Self::ACCENT_CYAN)
            .add_modifier(Modifier::BOLD)
    }

    pub fn search_highlight() -> Style {
        Style::default()
            .fg(Self::TEXT_INVERSE)
            .bg(Self::ACCENT_ORANGE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn docker() -> Style {
        Style::default().fg(Self::ACCENT_CYAN)
    }

    pub fn git_clean() -> Style {
        Style::default().fg(Self::HEALTHY)
    }

    pub fn git_dirty() -> Style {
        Style::default().fg(Self::WARNING)
    }

    pub fn port_number() -> Style {
        Style::default()
            .fg(Self::ACCENT_PURPLE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn process_name() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }

    /// Get the style for a status value.
    pub fn status_style(status: &crate::models::Status) -> Style {
        match status {
            crate::models::Status::Healthy => Self::healthy(),
            crate::models::Status::Warning(_) => Self::warning(),
            crate::models::Status::Zombie => Self::error(),
            crate::models::Status::Orphaned => Self::warning(),
            crate::models::Status::Unknown => Self::muted(),
        }
    }
}
