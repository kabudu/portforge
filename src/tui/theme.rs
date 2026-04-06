use ratatui::style::{Color, Modifier, Style};

/// PortForge dark theme color palette.
/// Matches the GitHub Pages site and dashboard for a consistent product identity.
pub struct Theme;

impl Theme {
    // ─── Background Colors ───
    pub const BG_PRIMARY: Color = Color::Rgb(9, 16, 28); // #09101c
    pub const BG_SURFACE: Color = Color::Rgb(15, 24, 42); // #0f182a
    pub const BG_HIGHLIGHT: Color = Color::Rgb(20, 31, 53); // #141f35
    pub const BG_OVERLAY: Color = Color::Rgb(10, 19, 35); // #0a1323

    // ─── Border Colors ───
    pub const BORDER: Color = Color::Rgb(38, 53, 80); // #263550
    pub const BORDER_FOCUS: Color = Color::Rgb(85, 198, 255); // #55c6ff

    // ─── Text Colors ───
    pub const TEXT_PRIMARY: Color = Color::Rgb(243, 246, 251); // #f3f6fb
    pub const TEXT_SECONDARY: Color = Color::Rgb(188, 199, 218); // #bcc7da
    pub const TEXT_MUTED: Color = Color::Rgb(145, 160, 184); // #91a0b8
    pub const TEXT_INVERSE: Color = Color::Rgb(8, 17, 29); // #08111d

    // ─── Status Colors ───
    pub const HEALTHY: Color = Color::Rgb(96, 211, 148); // #60d394
    pub const WARNING: Color = Color::Rgb(255, 177, 90); // #ffb15a
    pub const ERROR: Color = Color::Rgb(255, 107, 87); // #ff6b57
    pub const INFO: Color = Color::Rgb(85, 198, 255); // #55c6ff

    // ─── Accent Colors ───
    pub const ACCENT_BLUE: Color = Color::Rgb(85, 198, 255); // #55c6ff
    pub const ACCENT_PURPLE: Color = Color::Rgb(255, 177, 90); // warm accent reused for numeric emphasis
    pub const ACCENT_CYAN: Color = Color::Rgb(85, 198, 255); // align docker and info surfaces
    pub const ACCENT_ORANGE: Color = Color::Rgb(255, 122, 26); // #ff7a1a

    // ─── Semantic Styles ───

    pub fn title() -> Style {
        Style::default()
            .fg(Self::ACCENT_ORANGE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header() -> Style {
        Style::default()
            .fg(Self::TEXT_PRIMARY)
            .bg(Self::BG_HIGHLIGHT)
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
            .fg(Self::TEXT_PRIMARY)
            .bg(Self::BG_HIGHLIGHT)
    }

    pub fn accent() -> Style {
        Style::default()
            .fg(Self::ACCENT_BLUE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn key_hint() -> Style {
        Style::default()
            .fg(Self::ACCENT_BLUE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn search_highlight() -> Style {
        Style::default()
            .fg(Self::TEXT_INVERSE)
            .bg(Self::ACCENT_BLUE)
            .add_modifier(Modifier::BOLD)
    }

    pub fn docker() -> Style {
        Style::default().fg(Self::ACCENT_BLUE)
    }

    pub fn tunnel() -> Style {
        Style::default().fg(Self::ACCENT_ORANGE)
    }

    pub fn git_clean() -> Style {
        Style::default().fg(Self::HEALTHY)
    }

    pub fn git_dirty() -> Style {
        Style::default().fg(Self::WARNING)
    }

    pub fn port_number() -> Style {
        Style::default()
            .fg(Self::ACCENT_ORANGE)
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
