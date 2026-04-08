use crate::models::PortEntry;
use crate::tui::app::{App, ViewMode};
use crate::tui::theme::Theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
};

/// Render the application header bar.
pub fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: title and version
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" ◆ ", theme.accent()),
        Span::styled("PortForge", theme.title()),
        Span::styled(" · local port intelligence", theme.muted()),
        Span::styled(format!(" v{}", env!("CARGO_PKG_VERSION")), theme.muted()),
        Span::styled(format!(" [{}]", theme.name().as_str()), theme.muted()),
        if app.loading {
            Span::styled(" ⟳", theme.accent())
        } else {
            Span::raw("")
        },
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border()),
    );
    f.render_widget(title, chunks[0]);

    // Right: stats
    let total = app.entries.len();
    let healthy = app
        .entries
        .iter()
        .filter(|e| e.status == crate::models::Status::Healthy)
        .count();
    let docker_count = app.entries.iter().filter(|e| e.docker.is_some()).count();

    let stats = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {}", total), theme.accent()),
        Span::styled(" ports", theme.muted()),
        Span::styled("  •  ", theme.title()),
        Span::styled(format!("{}", healthy), theme.healthy()),
        Span::styled(" healthy", theme.muted()),
        Span::styled("  •  ", theme.title()),
        Span::styled(format!("{}", docker_count), theme.docker()),
        Span::styled(" docker", theme.muted()),
        Span::styled("  •  ", theme.title()),
        if app.show_all {
            Span::styled("[ALL]", theme.warning())
        } else {
            Span::styled("[DEV]", theme.accent())
        },
    ]))
    .alignment(Alignment::Right)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border()),
    );
    f.render_widget(stats, chunks[1]);
}

/// Render the bottom status bar.
pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let content = if let Some((ref msg, _)) = app.status_message {
        Line::from(vec![
            Span::styled(" ● ", theme.title()),
            Span::styled(msg, theme.accent()),
        ])
    } else {
        match app.view_mode {
            ViewMode::Table => Line::from(vec![
                Span::styled(" j/k", theme.key_hint()),
                Span::styled(" nav  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("Enter", theme.key_hint()),
                Span::styled(" detail  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("K", theme.key_hint()),
                Span::styled(" kill  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("/", theme.key_hint()),
                Span::styled(" search  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("a", theme.key_hint()),
                Span::styled(" all  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("Tab", theme.key_hint()),
                Span::styled(" tabs  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("?", theme.key_hint()),
                Span::styled(" help  ", theme.muted()),
                Span::styled("• ", theme.title()),
                Span::styled("q", theme.key_hint()),
                Span::styled(" quit", theme.muted()),
            ]),
            ViewMode::Search => Line::from(vec![
                Span::styled(" Type to search, ", theme.muted()),
                Span::styled("j/k", theme.key_hint()),
                Span::styled(" to browse, ", theme.muted()),
                Span::styled("Enter", theme.key_hint()),
                Span::styled(" to keep filter, ", theme.muted()),
                Span::styled("Esc", theme.key_hint()),
                Span::styled(" to clear", theme.muted()),
            ]),
            ViewMode::Detail => Line::from(vec![
                Span::styled(" Esc", theme.key_hint()),
                Span::styled(" back  ", theme.muted()),
                Span::styled("j/k", theme.key_hint()),
                Span::styled(" prev/next  ", theme.muted()),
                Span::styled("K", theme.key_hint()),
                Span::styled(" kill  ", theme.muted()),
                Span::styled("t", theme.key_hint()),
                Span::styled(" tree", theme.muted()),
            ]),
            _ => Line::from(vec![
                Span::styled(" Esc", theme.key_hint()),
                Span::styled(" back", theme.muted()),
            ]),
        }
    };

    let bar = Paragraph::new(content).style(theme.status_bar());
    f.render_widget(bar, area);
}

/// Render the help overlay modal.
pub fn render_help_overlay(f: &mut Frame, area: Rect, theme: &Theme) {
    let modal_area = centered_rect(60, 75, area);
    f.render_widget(Clear, modal_area);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", theme.title())),
        Line::from(vec![
            Span::styled("    j / ↓      ", theme.key_hint()),
            Span::styled("Move down", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    k / ↑      ", theme.key_hint()),
            Span::styled("Move up", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    g          ", theme.key_hint()),
            Span::styled("Go to top", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    G          ", theme.key_hint()),
            Span::styled("Go to bottom", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Mouse      ", theme.key_hint()),
            Span::styled("Click to select, scroll to navigate", theme.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Actions", theme.title())),
        Line::from(vec![
            Span::styled("    Enter / d  ", theme.key_hint()),
            Span::styled("View port details", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    K          ", theme.key_hint()),
            Span::styled("Kill process (confirm)", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    t          ", theme.key_hint()),
            Span::styled("Process tree", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    /          ", theme.key_hint()),
            Span::styled("Search / filter (j/k browse results)", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    a          ", theme.key_hint()),
            Span::styled("Kill process (confirm, Esc/n cancels)", theme.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Tabs & Views", theme.title())),
        Line::from(vec![
            Span::styled("    Tab        ", theme.key_hint()),
            Span::styled("Next tab", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Shift+Tab  ", theme.key_hint()),
            Span::styled("Previous tab", theme.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Appearance", theme.title())),
        Line::from(vec![
            Span::styled("    T          ", theme.key_hint()),
            Span::styled("Cycle theme (dark/light/solarized/nord/dracula)", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    m          ", theme.key_hint()),
            Span::styled("Toggle mouse support", theme.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Sorting", theme.title())),
        Line::from(vec![
            Span::styled("    1-8        ", theme.key_hint()),
            Span::styled("Sort by column (toggle direction)", theme.muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  General", theme.title())),
        Line::from(vec![
            Span::styled("    q / Esc    ", theme.key_hint()),
            Span::styled("Quit / Go back", theme.muted()),
        ]),
        Line::from(vec![
            Span::styled("    Ctrl+C     ", theme.key_hint()),
            Span::styled("Force quit", theme.muted()),
        ]),
        Line::from(""),
    ];

    let help = Paragraph::new(help_text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(theme.border_focus())
                .title(Span::styled(" ◆ Keyboard Shortcuts ", theme.title()))
                .title_bottom(Line::from(Span::styled(
                    " Press ? or Esc to close ",
                    theme.muted(),
                ))),
        )
        .style(ratatui::style::Style::default().bg(theme.bg_overlay()));
    f.render_widget(help, modal_area);
}

/// Render the kill confirmation dialog.
pub fn render_kill_confirm(f: &mut Frame, area: Rect, entry: &PortEntry, theme: &Theme) {
    let modal_area = centered_rect(50, 25, area);
    f.render_widget(Clear, modal_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Are you sure you want to kill this process?",
            theme.warning(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Port:    ", theme.muted()),
            Span::styled(format!("{}", entry.port), theme.port_number()),
        ]),
        Line::from(vec![
            Span::styled("    PID:     ", theme.muted()),
            Span::styled(format!("{}", entry.pid), theme.info()),
        ]),
        Line::from(vec![
            Span::styled("    Process: ", theme.muted()),
            Span::styled(&entry.process_name, theme.process_name()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y", theme.key_hint()),
            Span::styled(" / Enter = graceful  ", theme.muted()),
            Span::styled("f", theme.key_hint()),
            Span::styled(" = force  ", theme.muted()),
            Span::styled("Esc / n", theme.key_hint()),
            Span::styled(" = cancel", theme.muted()),
        ]),
        Line::from(""),
    ];

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(theme.error())
                .title(Span::styled(" ◆ Kill Confirmation ", theme.error())),
        )
        .style(ratatui::style::Style::default().bg(theme.bg_overlay()));
    f.render_widget(dialog, modal_area);
}

/// Render the search bar overlay at the bottom.
pub fn render_search_bar(f: &mut Frame, area: Rect, query: &str, theme: &Theme) {
    let search_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(4),
        width: area.width,
        height: 3,
    };
    f.render_widget(Clear, search_area);

    let search = Paragraph::new(Line::from(vec![
        Span::styled(" / ", theme.key_hint()),
        Span::raw(query),
        Span::styled("█", theme.info()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border_focus())
            .title(Span::styled(" ◆ Search ", theme.title())),
    )
    .style(ratatui::style::Style::default().bg(theme.bg_overlay()));
    f.render_widget(search, search_area);
}

/// Create a centered rectangle within the given area.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
