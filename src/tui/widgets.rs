use crate::models::PortEntry;
use crate::tui::app::{App, ViewMode};
use crate::tui::theme::Theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Render the application header bar.
pub fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: title and version
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" ⚡ ", Theme::title()),
        Span::styled("PortForge", Theme::title()),
        Span::styled(format!(" v{}", env!("CARGO_PKG_VERSION")), Theme::muted()),
        if app.loading {
            Span::styled(" ⟳", Theme::info())
        } else {
            Span::raw("")
        },
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border()),
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
        Span::styled(format!(" {} ", total), Theme::info()),
        Span::styled("ports", Theme::muted()),
        Span::raw("  "),
        Span::styled(format!("{}", healthy), Theme::healthy()),
        Span::styled(" healthy", Theme::muted()),
        Span::raw("  "),
        Span::styled(format!("{}", docker_count), Theme::docker()),
        Span::styled(" docker", Theme::muted()),
        Span::raw("  "),
        if app.show_all {
            Span::styled("[ALL]", Theme::warning())
        } else {
            Span::styled("[DEV]", Theme::info())
        },
    ]))
    .alignment(Alignment::Right)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border()),
    );
    f.render_widget(stats, chunks[1]);
}

/// Render the bottom status bar.
pub fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let content = if let Some((ref msg, _)) = app.status_message {
        Line::from(vec![
            Span::styled(" ", Theme::muted()),
            Span::styled(msg, Theme::info()),
        ])
    } else {
        match app.view_mode {
            ViewMode::Table => Line::from(vec![
                Span::styled(" j/k", Theme::key_hint()),
                Span::styled(" navigate  ", Theme::muted()),
                Span::styled("Enter", Theme::key_hint()),
                Span::styled(" detail  ", Theme::muted()),
                Span::styled("K", Theme::key_hint()),
                Span::styled(" kill  ", Theme::muted()),
                Span::styled("/", Theme::key_hint()),
                Span::styled(" search  ", Theme::muted()),
                Span::styled("t", Theme::key_hint()),
                Span::styled(" tree  ", Theme::muted()),
                Span::styled("T", Theme::key_hint()),
                Span::styled(" all  ", Theme::muted()),
                Span::styled("?", Theme::key_hint()),
                Span::styled(" help  ", Theme::muted()),
                Span::styled("q", Theme::key_hint()),
                Span::styled(" quit", Theme::muted()),
            ]),
            ViewMode::Search => Line::from(vec![
                Span::styled(" Type to search, ", Theme::muted()),
                Span::styled("Enter", Theme::key_hint()),
                Span::styled(" to confirm, ", Theme::muted()),
                Span::styled("Esc", Theme::key_hint()),
                Span::styled(" to cancel", Theme::muted()),
            ]),
            ViewMode::Detail => Line::from(vec![
                Span::styled(" Esc", Theme::key_hint()),
                Span::styled(" back  ", Theme::muted()),
                Span::styled("K", Theme::key_hint()),
                Span::styled(" kill  ", Theme::muted()),
                Span::styled("t", Theme::key_hint()),
                Span::styled(" tree", Theme::muted()),
            ]),
            _ => Line::from(vec![
                Span::styled(" Esc", Theme::key_hint()),
                Span::styled(" back", Theme::muted()),
            ]),
        }
    };

    let bar = Paragraph::new(content).style(Theme::status_bar());
    f.render_widget(bar, area);
}

/// Render the help overlay modal.
pub fn render_help_overlay(f: &mut Frame, area: Rect) {
    let modal_area = centered_rect(60, 70, area);
    f.render_widget(Clear, modal_area);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("  Navigation", Theme::title())),
        Line::from(vec![
            Span::styled("    j / ↓      ", Theme::key_hint()),
            Span::styled("Move down", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    k / ↑      ", Theme::key_hint()),
            Span::styled("Move up", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    g          ", Theme::key_hint()),
            Span::styled("Go to top", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    G          ", Theme::key_hint()),
            Span::styled("Go to bottom", Theme::muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Actions", Theme::title())),
        Line::from(vec![
            Span::styled("    Enter / d  ", Theme::key_hint()),
            Span::styled("View port details", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    K          ", Theme::key_hint()),
            Span::styled("Kill process (confirm)", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    t          ", Theme::key_hint()),
            Span::styled("Process tree", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    /          ", Theme::key_hint()),
            Span::styled("Search / filter", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    T          ", Theme::key_hint()),
            Span::styled("Toggle all / dev ports", Theme::muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Sorting", Theme::title())),
        Line::from(vec![
            Span::styled("    1-8        ", Theme::key_hint()),
            Span::styled("Sort by column (toggle direction)", Theme::muted()),
        ]),
        Line::from(""),
        Line::from(Span::styled("  General", Theme::title())),
        Line::from(vec![
            Span::styled("    q / Esc    ", Theme::key_hint()),
            Span::styled("Quit / Go back", Theme::muted()),
        ]),
        Line::from(vec![
            Span::styled("    Ctrl+C     ", Theme::key_hint()),
            Span::styled("Force quit", Theme::muted()),
        ]),
        Line::from(""),
    ];

    let help = Paragraph::new(help_text)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Theme::border_focus())
                .title(Span::styled(" ⌨ Keyboard Shortcuts ", Theme::title()))
                .title_bottom(Line::from(Span::styled(
                    " Press ? or Esc to close ",
                    Theme::muted(),
                ))),
        )
        .style(ratatui::style::Style::default().bg(Theme::BG_OVERLAY));
    f.render_widget(help, modal_area);
}

/// Render the kill confirmation dialog.
pub fn render_kill_confirm(f: &mut Frame, area: Rect, entry: &PortEntry) {
    let modal_area = centered_rect(50, 25, area);
    f.render_widget(Clear, modal_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Are you sure you want to kill this process?",
            Theme::warning(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Port:    ", Theme::muted()),
            Span::styled(format!("{}", entry.port), Theme::port_number()),
        ]),
        Line::from(vec![
            Span::styled("    PID:     ", Theme::muted()),
            Span::styled(format!("{}", entry.pid), Theme::info()),
        ]),
        Line::from(vec![
            Span::styled("    Process: ", Theme::muted()),
            Span::styled(&entry.process_name, Theme::process_name()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y", Theme::key_hint()),
            Span::styled(" = graceful  ", Theme::muted()),
            Span::styled("f", Theme::key_hint()),
            Span::styled(" = force  ", Theme::muted()),
            Span::styled("any other", Theme::key_hint()),
            Span::styled(" = cancel", Theme::muted()),
        ]),
        Line::from(""),
    ];

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Theme::error())
                .title(Span::styled(" ⚠ Kill Confirmation ", Theme::error())),
        )
        .style(ratatui::style::Style::default().bg(Theme::BG_OVERLAY));
    f.render_widget(dialog, modal_area);
}

/// Render the search bar overlay at the bottom.
pub fn render_search_bar(f: &mut Frame, area: Rect, query: &str) {
    let search_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(4),
        width: area.width,
        height: 3,
    };
    f.render_widget(Clear, search_area);

    let search = Paragraph::new(Line::from(vec![
        Span::styled(" / ", Theme::key_hint()),
        Span::raw(query),
        Span::styled("█", Theme::info()),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border_focus())
            .title(Span::styled(" Search ", Theme::title())),
    )
    .style(ratatui::style::Style::default().bg(Theme::BG_OVERLAY));
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
