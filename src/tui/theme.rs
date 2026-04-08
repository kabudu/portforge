use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Available theme names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeName {
    Dark,
    Light,
    Solarized,
    Nord,
    Dracula,
}

impl ThemeName {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeName::Dark => "dark",
            ThemeName::Light => "light",
            ThemeName::Solarized => "solarized",
            ThemeName::Nord => "nord",
            ThemeName::Dracula => "dracula",
        }
    }

    pub fn all() -> Vec<ThemeName> {
        vec![
            ThemeName::Dark,
            ThemeName::Light,
            ThemeName::Solarized,
            ThemeName::Nord,
            ThemeName::Dracula,
        ]
    }
}

impl FromStr for ThemeName {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dark" => Ok(ThemeName::Dark),
            "light" => Ok(ThemeName::Light),
            "solarized" => Ok(ThemeName::Solarized),
            "nord" => Ok(ThemeName::Nord),
            "dracula" => Ok(ThemeName::Dracula),
            _ => Err(()),
        }
    }
}

/// Color palette for a theme.
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg_primary: Color,
    pub bg_surface: Color,
    pub bg_highlight: Color,
    pub bg_overlay: Color,
    pub border: Color,
    pub border_focus: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_inverse: Color,
    pub healthy: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub accent_blue: Color,
    pub accent_orange: Color,
    pub sparkline: Color,
    pub sparkline_peak: Color,
}

impl ThemePalette {
    pub fn for_theme(name: ThemeName) -> Self {
        match name {
            ThemeName::Dark => Self::dark(),
            ThemeName::Light => Self::light(),
            ThemeName::Solarized => Self::solarized(),
            ThemeName::Nord => Self::nord(),
            ThemeName::Dracula => Self::dracula(),
        }
    }

    fn dark() -> Self {
        Self {
            bg_primary: Color::Rgb(9, 16, 28),
            bg_surface: Color::Rgb(15, 24, 42),
            bg_highlight: Color::Rgb(20, 31, 53),
            bg_overlay: Color::Rgb(10, 19, 35),
            border: Color::Rgb(38, 53, 80),
            border_focus: Color::Rgb(85, 198, 255),
            text_primary: Color::Rgb(243, 246, 251),
            text_secondary: Color::Rgb(188, 199, 218),
            text_muted: Color::Rgb(145, 160, 184),
            text_inverse: Color::Rgb(8, 17, 29),
            healthy: Color::Rgb(96, 211, 148),
            warning: Color::Rgb(255, 177, 90),
            error: Color::Rgb(255, 107, 87),
            info: Color::Rgb(85, 198, 255),
            accent_blue: Color::Rgb(85, 198, 255),
            accent_orange: Color::Rgb(255, 122, 26),
            sparkline: Color::Rgb(85, 198, 255),
            sparkline_peak: Color::Rgb(255, 107, 87),
        }
    }

    fn light() -> Self {
        Self {
            bg_primary: Color::Rgb(250, 250, 250),
            bg_surface: Color::Rgb(240, 240, 240),
            bg_highlight: Color::Rgb(224, 224, 224),
            bg_overlay: Color::Rgb(245, 245, 245),
            border: Color::Rgb(200, 200, 200),
            border_focus: Color::Rgb(66, 133, 244),
            text_primary: Color::Rgb(33, 33, 33),
            text_secondary: Color::Rgb(66, 66, 66),
            text_muted: Color::Rgb(117, 117, 117),
            text_inverse: Color::Rgb(255, 255, 255),
            healthy: Color::Rgb(46, 125, 50),
            warning: Color::Rgb(245, 124, 0),
            error: Color::Rgb(211, 47, 47),
            info: Color::Rgb(66, 133, 244),
            accent_blue: Color::Rgb(66, 133, 244),
            accent_orange: Color::Rgb(230, 81, 0),
            sparkline: Color::Rgb(66, 133, 244),
            sparkline_peak: Color::Rgb(211, 47, 47),
        }
    }

    fn solarized() -> Self {
        Self {
            bg_primary: Color::Rgb(0, 43, 54),
            bg_surface: Color::Rgb(7, 54, 66),
            bg_highlight: Color::Rgb(88, 110, 117),
            bg_overlay: Color::Rgb(0, 43, 54),
            border: Color::Rgb(131, 148, 150),
            border_focus: Color::Rgb(38, 139, 210),
            text_primary: Color::Rgb(238, 232, 213),
            text_secondary: Color::Rgb(147, 161, 161),
            text_muted: Color::Rgb(131, 148, 150),
            text_inverse: Color::Rgb(0, 43, 54),
            healthy: Color::Rgb(133, 153, 0),
            warning: Color::Rgb(181, 137, 0),
            error: Color::Rgb(220, 50, 47),
            info: Color::Rgb(38, 139, 210),
            accent_blue: Color::Rgb(38, 139, 210),
            accent_orange: Color::Rgb(203, 75, 22),
            sparkline: Color::Rgb(38, 139, 210),
            sparkline_peak: Color::Rgb(220, 50, 47),
        }
    }

    fn nord() -> Self {
        Self {
            bg_primary: Color::Rgb(46, 52, 64),
            bg_surface: Color::Rgb(59, 66, 82),
            bg_highlight: Color::Rgb(67, 76, 94),
            bg_overlay: Color::Rgb(46, 52, 64),
            border: Color::Rgb(76, 86, 106),
            border_focus: Color::Rgb(136, 192, 208),
            text_primary: Color::Rgb(216, 222, 233),
            text_secondary: Color::Rgb(180, 192, 204),
            text_muted: Color::Rgb(143, 156, 172),
            text_inverse: Color::Rgb(46, 52, 64),
            healthy: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            info: Color::Rgb(136, 192, 208),
            accent_blue: Color::Rgb(136, 192, 208),
            accent_orange: Color::Rgb(208, 135, 112),
            sparkline: Color::Rgb(136, 192, 208),
            sparkline_peak: Color::Rgb(191, 97, 106),
        }
    }

    fn dracula() -> Self {
        Self {
            bg_primary: Color::Rgb(40, 42, 54),
            bg_surface: Color::Rgb(68, 71, 90),
            bg_highlight: Color::Rgb(98, 114, 164),
            bg_overlay: Color::Rgb(40, 42, 54),
            border: Color::Rgb(68, 71, 90),
            border_focus: Color::Rgb(139, 233, 253),
            text_primary: Color::Rgb(248, 248, 242),
            text_secondary: Color::Rgb(189, 193, 203),
            text_muted: Color::Rgb(139, 147, 163),
            text_inverse: Color::Rgb(40, 42, 54),
            healthy: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(241, 250, 140),
            error: Color::Rgb(255, 85, 85),
            info: Color::Rgb(139, 233, 253),
            accent_blue: Color::Rgb(139, 233, 253),
            accent_orange: Color::Rgb(255, 184, 108),
            sparkline: Color::Rgb(139, 233, 253),
            sparkline_peak: Color::Rgb(255, 85, 85),
        }
    }
}

/// Active theme — uses the palette for the current theme.
pub struct Theme {
    palette: ThemePalette,
    name: ThemeName,
}

impl Theme {
    pub fn new(name: ThemeName) -> Self {
        Self {
            palette: ThemePalette::for_theme(name),
            name,
        }
    }

    pub fn name(&self) -> ThemeName {
        self.name
    }

    pub fn palette(&self) -> &ThemePalette {
        &self.palette
    }

    // ─── Color accessors ───

    pub const fn bg_primary(&self) -> Color {
        self.palette.bg_primary
    }
    pub const fn bg_surface(&self) -> Color {
        self.palette.bg_surface
    }
    pub const fn bg_highlight(&self) -> Color {
        self.palette.bg_highlight
    }
    pub const fn bg_overlay(&self) -> Color {
        self.palette.bg_overlay
    }
    pub const fn border_color(&self) -> Color {
        self.palette.border
    }
    pub const fn border_focus_color(&self) -> Color {
        self.palette.border_focus
    }

    // ─── Semantic Styles ───

    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_orange)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header(&self) -> Style {
        Style::default()
            .fg(self.palette.text_primary)
            .bg(self.palette.bg_highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn row_normal(&self) -> Style {
        Style::default().fg(self.palette.text_primary)
    }

    pub fn row_selected(&self) -> Style {
        Style::default()
            .fg(self.palette.text_primary)
            .bg(self.palette.bg_highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn row_alt(&self) -> Style {
        Style::default()
            .fg(self.palette.text_primary)
            .bg(self.palette.bg_surface)
    }

    pub fn healthy(&self) -> Style {
        Style::default()
            .fg(self.palette.healthy)
            .add_modifier(Modifier::BOLD)
    }

    pub fn warning(&self) -> Style {
        Style::default()
            .fg(self.palette.warning)
            .add_modifier(Modifier::BOLD)
    }

    pub fn error(&self) -> Style {
        Style::default()
            .fg(self.palette.error)
            .add_modifier(Modifier::BOLD)
    }

    pub fn info(&self) -> Style {
        Style::default().fg(self.palette.info)
    }

    pub fn muted(&self) -> Style {
        Style::default().fg(self.palette.text_muted)
    }

    pub fn border(&self) -> Style {
        Style::default().fg(self.palette.border)
    }

    pub fn border_focus(&self) -> Style {
        Style::default().fg(self.palette.border_focus)
    }

    pub fn status_bar(&self) -> Style {
        Style::default()
            .fg(self.palette.text_primary)
            .bg(self.palette.bg_highlight)
    }

    pub fn accent(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_blue)
            .add_modifier(Modifier::BOLD)
    }

    pub fn key_hint(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_blue)
            .add_modifier(Modifier::BOLD)
    }

    pub fn search_highlight(&self) -> Style {
        Style::default()
            .fg(self.palette.text_inverse)
            .bg(self.palette.accent_blue)
            .add_modifier(Modifier::BOLD)
    }

    pub fn docker(&self) -> Style {
        Style::default().fg(self.palette.accent_blue)
    }

    pub fn tunnel(&self) -> Style {
        Style::default().fg(self.palette.accent_orange)
    }

    pub fn git_clean(&self) -> Style {
        Style::default().fg(self.palette.healthy)
    }

    pub fn git_dirty(&self) -> Style {
        Style::default().fg(self.palette.warning)
    }

    pub fn port_number(&self) -> Style {
        Style::default()
            .fg(self.palette.accent_orange)
            .add_modifier(Modifier::BOLD)
    }

    pub fn process_name(&self) -> Style {
        Style::default().fg(self.palette.text_primary)
    }

    pub fn sparkline(&self) -> Style {
        Style::default().fg(self.palette.sparkline)
    }

    pub fn sparkline_peak(&self) -> Style {
        Style::default().fg(self.palette.sparkline_peak)
    }

    pub fn tab_active(&self) -> Style {
        Style::default()
            .fg(self.palette.text_primary)
            .bg(self.palette.bg_highlight)
            .add_modifier(Modifier::BOLD)
    }

    pub fn tab_inactive(&self) -> Style {
        Style::default().fg(self.palette.text_muted)
    }

    /// Get the style for a status value.
    pub fn status_style(&self, status: &crate::models::Status) -> Style {
        match status {
            crate::models::Status::Healthy => self.healthy(),
            crate::models::Status::Warning(_) => self.warning(),
            crate::models::Status::Zombie => self.error(),
            crate::models::Status::Orphaned => self.warning(),
            crate::models::Status::Unknown => self.muted(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_name_from_str() {
        assert_eq!("dark".parse::<ThemeName>(), Ok(ThemeName::Dark));
        assert_eq!("light".parse::<ThemeName>(), Ok(ThemeName::Light));
        assert_eq!("solarized".parse::<ThemeName>(), Ok(ThemeName::Solarized));
        assert_eq!("nord".parse::<ThemeName>(), Ok(ThemeName::Nord));
        assert_eq!("dracula".parse::<ThemeName>(), Ok(ThemeName::Dracula));
        assert_eq!("unknown".parse::<ThemeName>(), Err(()));
        assert_eq!("DARK".parse::<ThemeName>(), Ok(ThemeName::Dark));
    }

    #[test]
    fn test_theme_name_as_str() {
        assert_eq!(ThemeName::Dark.as_str(), "dark");
        assert_eq!(ThemeName::Light.as_str(), "light");
        assert_eq!(ThemeName::Solarized.as_str(), "solarized");
        assert_eq!(ThemeName::Nord.as_str(), "nord");
        assert_eq!(ThemeName::Dracula.as_str(), "dracula");
    }

    #[test]
    fn test_theme_name_all() {
        let all = ThemeName::all();
        assert_eq!(all.len(), 5);
        assert!(all.contains(&ThemeName::Dark));
        assert!(all.contains(&ThemeName::Light));
        assert!(all.contains(&ThemeName::Solarized));
        assert!(all.contains(&ThemeName::Nord));
        assert!(all.contains(&ThemeName::Dracula));
    }

    #[test]
    fn test_theme_creation() {
        for name in ThemeName::all() {
            let theme = Theme::new(name);
            assert_eq!(theme.name(), name);
        }
    }

    #[test]
    fn test_theme_palette_not_black() {
        // Ensure themes have non-zero colors
        for name in ThemeName::all() {
            let palette = ThemePalette::for_theme(name);
            // bg_primary should not be black for any theme
            assert_ne!(palette.bg_primary, Color::Rgb(0, 0, 0));
            assert_ne!(palette.text_primary, Color::Rgb(0, 0, 0));
        }
    }

    #[test]
    fn test_theme_styles_not_default() {
        let theme = Theme::new(ThemeName::Dark);
        // Verify styles are not the default Style
        assert_ne!(theme.title(), Style::default());
        assert_ne!(theme.header(), Style::default());
        assert_ne!(theme.healthy(), Style::default());
        assert_ne!(theme.warning(), Style::default());
        assert_ne!(theme.error(), Style::default());
        assert_ne!(theme.muted(), Style::default());
        assert_ne!(theme.accent(), Style::default());
    }

    #[test]
    fn test_theme_status_style() {
        let theme = Theme::new(ThemeName::Dark);
        let healthy_style = theme.status_style(&crate::models::Status::Healthy);
        let warning_style = theme.status_style(&crate::models::Status::Warning("test".into()));
        let zombie_style = theme.status_style(&crate::models::Status::Zombie);
        let unknown_style = theme.status_style(&crate::models::Status::Unknown);

        // Each status should have a distinct style
        assert_ne!(healthy_style, warning_style);
        assert_ne!(healthy_style, zombie_style);
        assert_ne!(warning_style, zombie_style);
        assert_ne!(unknown_style, healthy_style);
    }

    #[test]
    fn test_all_themes_have_distinct_palettes() {
        let dark = ThemePalette::for_theme(ThemeName::Dark);
        let light = ThemePalette::for_theme(ThemeName::Light);
        let solarized = ThemePalette::for_theme(ThemeName::Solarized);
        let nord = ThemePalette::for_theme(ThemeName::Nord);
        let dracula = ThemePalette::for_theme(ThemeName::Dracula);

        // Dark and light should have very different backgrounds
        assert_ne!(dark.bg_primary, light.bg_primary);

        // All themes should have different primary backgrounds
        let bgs = [
            dark.bg_primary,
            light.bg_primary,
            solarized.bg_primary,
            nord.bg_primary,
            dracula.bg_primary,
        ];
        let unique_bgs: std::collections::HashSet<_> = bgs.iter().collect();
        assert_eq!(
            unique_bgs.len(),
            5,
            "All themes should have distinct background colors"
        );
    }
}
