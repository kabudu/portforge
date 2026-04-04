use crate::models::Status;
use crate::tui::app::{App, ViewMode};
use crate::tui::theme::Theme;
use crate::tui::widgets;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

/// Main render function — dispatches to the current view.
pub fn render(f: &mut Frame, app: &App) {
    // Main background
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(Theme::BG_PRIMARY)),
        area,
    );

    // Main layout: header + content + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Header
    widgets::render_header(f, chunks[0], app);

    // Content area
    match app.view_mode {
        ViewMode::Table | ViewMode::Search => render_table(f, chunks[1], app),
        ViewMode::Detail => render_detail(f, chunks[1], app),
        ViewMode::ProcessTree => render_process_tree(f, chunks[1], app),
        _ => render_table(f, chunks[1], app),
    }

    // Status bar
    widgets::render_status_bar(f, chunks[2], app);

    // Overlays (modals)
    match app.view_mode {
        ViewMode::Help => widgets::render_help_overlay(f, area),
        ViewMode::KillConfirm => {
            if let Some(entry) = app.selected_entry() {
                widgets::render_kill_confirm(f, area, entry);
            }
        }
        ViewMode::Search => widgets::render_search_bar(f, area, &app.search_query),
        _ => {}
    }
}

/// Render the main port table.
fn render_table(f: &mut Frame, area: Rect, app: &App) {
    if app.entries.is_empty() && !app.loading {
        let msg = if app.show_all {
            "No listening ports found."
        } else {
            "No dev project ports found. Press 'a' to show all ports."
        };
        let paragraph = Paragraph::new(msg)
            .style(Theme::muted())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Theme::border())
                    .title(Span::styled(" Ports ", Theme::title())),
            );
        f.render_widget(paragraph, area);
        return;
    }

    if app.loading && app.entries.is_empty() {
        let paragraph = Paragraph::new("⏳ Scanning ports...")
            .style(Theme::info())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Theme::border())
                    .title(Span::styled(" Ports ", Theme::title())),
            );
        f.render_widget(paragraph, area);
        return;
    }

    // Column headers with sort indicators
    let header_cells = [
        ("Port", SortCol::Port),
        ("PID", SortCol::Pid),
        ("Process", SortCol::Process),
        ("Project", SortCol::Project),
        ("Git", SortCol::Git),
        ("Docker", SortCol::Docker),
        ("Uptime", SortCol::Uptime),
        ("Mem", SortCol::Mem),
        ("CPU", SortCol::Cpu),
        ("Status", SortCol::Status),
    ];

    let header = Row::new(header_cells.iter().map(|(name, col)| {
        let sort_indicator = if col.matches_field(app.sort_field) {
            format!(" {}", app.sort_direction.indicator())
        } else {
            String::new()
        };
        Cell::from(format!("{}{}", name, sort_indicator))
    }))
    .style(Theme::header())
    .height(1);

    // Data rows
    let rows: Vec<Row> = app
        .filtered_entries
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let entry = &app.entries[idx];
            let is_selected = i == app.selected;

            let status_style = Theme::status_style(&entry.status);

            let cells = vec![
                Cell::from(format!("{}", entry.port)).style(Theme::port_number()),
                Cell::from(format!("{}", entry.pid)).style(Theme::muted()),
                Cell::from(entry.process_name.clone()).style(Theme::process_name()),
                Cell::from(entry.project_display()).style(Theme::info()),
                Cell::from(entry.git_display()).style(if entry.git.as_ref().is_some_and(|g| g.dirty) {
                    Theme::git_dirty()
                } else {
                    Theme::git_clean()
                }),
                Cell::from(entry.docker_display()).style(Theme::docker()),
                Cell::from(entry.uptime_display()).style(Theme::muted()),
                Cell::from(format!("{:.0}MB", entry.memory_mb)).style(Theme::muted()),
                Cell::from(format!("{:.1}%", entry.cpu_percent)).style(
                    if entry.cpu_percent > 50.0 {
                        Theme::warning()
                    } else {
                        Theme::muted()
                    },
                ),
                Cell::from(entry.status.to_string()).style(status_style),
            ];

            let style = if is_selected {
                Theme::row_selected()
            } else if i % 2 == 0 {
                Theme::row_normal()
            } else {
                Theme::row_alt()
            };

            Row::new(cells).style(style).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),   // Port
            Constraint::Length(8),   // PID
            Constraint::Length(15),  // Process
            Constraint::Min(18),    // Project
            Constraint::Length(14),  // Git
            Constraint::Length(14),  // Docker
            Constraint::Length(9),   // Uptime
            Constraint::Length(8),   // Mem
            Constraint::Length(7),   // CPU
            Constraint::Length(12),  // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if app.view_mode == ViewMode::Search {
                Theme::border_focus()
            } else {
                Theme::border()
            })
            .title(Span::styled(
                format!(
                    " ⚡ Ports ({}{}) ",
                    app.filtered_entries.len(),
                    if !app.search_query.is_empty() {
                        format!(" / {} total", app.entries.len())
                    } else {
                        String::new()
                    }
                ),
                Theme::title(),
            ))
            .title_bottom(Line::from(vec![
                Span::styled(" Sort: ", Theme::muted()),
                Span::styled(
                    format!("{} {}", app.sort_field.label(), app.sort_direction.indicator()),
                    Theme::info(),
                ),
                Span::raw(" "),
            ])),
    )
    .row_highlight_style(Theme::row_selected());

    let mut state = TableState::default();
    state.select(Some(app.selected));
    f.render_stateful_widget(table, area, &mut state);
}

/// Render detailed port inspection.
fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let entry = match app.selected_entry() {
        Some(e) => e,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Basic info
            Constraint::Min(4),    // Extra sections
        ])
        .split(area);

    // Basic info
    let basic_lines = vec![
        Line::from(vec![
            Span::styled("  Port:      ", Theme::muted()),
            Span::styled(format!("{}/{}", entry.port, entry.protocol), Theme::port_number()),
        ]),
        Line::from(vec![
            Span::styled("  PID:       ", Theme::muted()),
            Span::styled(format!("{}", entry.pid), Theme::info()),
        ]),
        Line::from(vec![
            Span::styled("  Process:   ", Theme::muted()),
            Span::styled(&entry.process_name, Theme::process_name()),
        ]),
        Line::from(vec![
            Span::styled("  Command:   ", Theme::muted()),
            Span::raw(&entry.command),
        ]),
        Line::from(vec![
            Span::styled("  Memory:    ", Theme::muted()),
            Span::raw(format!("{:.1} MB", entry.memory_mb)),
        ]),
        Line::from(vec![
            Span::styled("  Status:    ", Theme::muted()),
            Span::styled(entry.status.to_string(), Theme::status_style(&entry.status)),
        ]),
    ];

    let basic = Paragraph::new(basic_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Theme::border_focus())
            .title(Span::styled(
                format!(" 🔍 Port {} Detail ", entry.port),
                Theme::title(),
            ))
            .title_bottom(Line::from(Span::styled(
                " ESC to go back │ K to kill │ t for tree ",
                Theme::muted(),
            ))),
    );
    f.render_widget(basic, chunks[0]);

    // Extra sections (project, git, docker, health)
    let mut extra_lines = Vec::new();

    if let Some(ref project) = entry.project {
        extra_lines.push(Line::from(Span::styled("  📦 Project", Theme::title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Kind:      ", Theme::muted()),
            Span::raw(&project.kind),
        ]));
        if !project.framework.is_empty() {
            extra_lines.push(Line::from(vec![
                Span::styled("    Framework: ", Theme::muted()),
                Span::styled(&project.framework, Theme::info()),
            ]));
        }
        extra_lines.push(Line::from(""));
    }

    if let Some(ref git) = entry.git {
        extra_lines.push(Line::from(Span::styled("  🔀 Git", Theme::title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Branch:    ", Theme::muted()),
            Span::styled(
                &git.branch,
                if git.dirty { Theme::git_dirty() } else { Theme::git_clean() },
            ),
            if git.dirty {
                Span::styled(" (modified)", Theme::warning())
            } else {
                Span::styled(" (clean)", Theme::healthy())
            },
        ]));
        extra_lines.push(Line::from(""));
    }

    if let Some(ref docker) = entry.docker {
        extra_lines.push(Line::from(Span::styled("  🐳 Docker", Theme::title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Container: ", Theme::muted()),
            Span::styled(&docker.container_name, Theme::docker()),
        ]));
        extra_lines.push(Line::from(vec![
            Span::styled("    Image:     ", Theme::muted()),
            Span::raw(&docker.image),
        ]));
        extra_lines.push(Line::from(""));
    }

    if let Some(ref health) = entry.health_check {
        extra_lines.push(Line::from(Span::styled("  🏥 Health Check", Theme::title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Status:    ", Theme::muted()),
            Span::styled(
                health.status.to_string(),
                Theme::status_style(match health.status {
                    crate::models::HealthStatus::Healthy => &Status::Healthy,
                    crate::models::HealthStatus::Unhealthy => &Status::Zombie,
                    crate::models::HealthStatus::Unknown => &Status::Unknown,
                }),
            ),
        ]));
        extra_lines.push(Line::from(vec![
            Span::styled("    Latency:   ", Theme::muted()),
            Span::raw(format!("{}ms", health.latency_ms)),
        ]));
        extra_lines.push(Line::from(""));
    }

    let extra = Paragraph::new(extra_lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::border()),
        );
    f.render_widget(extra, chunks[1]);
}

/// Render process tree view.
fn render_process_tree(f: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = app
        .process_tree
        .iter()
        .map(|entry| {
            let indent = if entry.depth == 0 {
                String::new()
            } else {
                format!("{}├─ ", "│  ".repeat(entry.depth - 1))
            };

            Line::from(vec![
                Span::styled(indent, Theme::border()),
                Span::styled(entry.name.clone(), Theme::process_name()),
                Span::styled(format!(" (PID: {})", entry.pid), Theme::muted()),
                Span::raw("  "),
                Span::styled(
                    format!("CPU: {:.1}%", entry.cpu_percent),
                    if entry.cpu_percent > 50.0 {
                        Theme::warning()
                    } else {
                        Theme::muted()
                    },
                ),
                Span::raw("  "),
                Span::styled(format!("Mem: {:.1}MB", entry.memory_mb), Theme::muted()),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::border_focus())
                .title(Span::styled(" 🌲 Process Tree ", Theme::title()))
                .title_bottom(Line::from(Span::styled(
                    " ESC to go back ",
                    Theme::muted(),
                ))),
        );
    f.render_widget(paragraph, area);
}

/// Helper enum for matching sort columns.
enum SortCol {
    Port,
    Pid,
    Process,
    Project,
    Git,
    Docker,
    Uptime,
    Mem,
    Cpu,
    Status,
}

impl SortCol {
    fn matches_field(&self, field: crate::models::SortField) -> bool {
        use crate::models::SortField;
        matches!(
            (self, field),
            (SortCol::Port, SortField::Port)
                | (SortCol::Pid, SortField::Pid)
                | (SortCol::Process, SortField::Process)
                | (SortCol::Project, SortField::Project)
                | (SortCol::Mem, SortField::Memory)
                | (SortCol::Cpu, SortField::Cpu)
                | (SortCol::Uptime, SortField::Uptime)
                | (SortCol::Status, SortField::Status)
        )
    }
}
