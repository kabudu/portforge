use crate::models::Status;
use crate::resource_history::sparkline_text;
use crate::tui::app::{App, Tab, ViewMode};
use crate::tui::widgets;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
};

/// Main render function — dispatches to the current view.
pub fn render(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    
    // Main background
    let area = f.area();
    f.render_widget(
        Block::default().style(Style::default().bg(theme.bg_primary())),
        area,
    );

    // Main layout: header + tabs + content + status bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(1), // Tab bar
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Header
    widgets::render_header(f, chunks[0], app);

    // Tab bar
    render_tab_bar(f, chunks[1], app);

    // Content area based on active tab
    match app.active_tab {
        Tab::Ports => {
            match app.view_mode {
                ViewMode::Table | ViewMode::Search => render_table(f, chunks[2], app),
                ViewMode::Detail => render_detail(f, chunks[2], app),
                ViewMode::ProcessTree => render_process_tree(f, chunks[2], app),
                _ => render_table(f, chunks[2], app),
            }
        }
        Tab::Processes => render_processes_tab(f, chunks[2], app),
        Tab::Docker => render_docker_tab(f, chunks[2], app),
        Tab::Logs => render_logs_tab(f, chunks[2], app),
    }

    // Status bar
    widgets::render_status_bar(f, chunks[3], app);

    // Overlays (modals)
    match app.view_mode {
        ViewMode::Help => widgets::render_help_overlay(f, area, theme),
        ViewMode::KillConfirm => {
            if let Some(entry) = app.selected_entry() {
                widgets::render_kill_confirm(f, area, entry, theme);
            }
        }
        ViewMode::Search => widgets::render_search_bar(f, area, &app.search_query, theme),
        _ => {}
    }
}

/// Render the tab bar.
fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let tabs = [Tab::Ports, Tab::Processes, Tab::Docker, Tab::Logs];
    
    let spans: Vec<Span> = tabs.iter().map(|tab| {
        if *tab == app.active_tab {
            Span::styled(
                format!(" {} ", tab.label()),
                theme.tab_active(),
            )
        } else {
            Span::styled(
                format!(" {} ", tab.label()),
                theme.tab_inactive(),
            )
        }
    }).collect();

    let mut line_spans = vec![Span::raw("  ")];
    for (i, span) in spans.into_iter().enumerate() {
        line_spans.push(span);
        if i < tabs.len() - 1 {
            line_spans.push(Span::styled(" │ ", theme.muted()));
        }
    }

    let bar = Paragraph::new(Line::from(line_spans))
        .style(Style::default().bg(theme.bg_surface()));
    f.render_widget(bar, area);
}

/// Render the main port table.
fn render_table(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
    if app.entries.is_empty() && !app.loading {
        let msg = if app.show_all {
            "No listening ports found."
        } else {
            "No dev project ports found. Press 'a' to show all ports."
        };
        let paragraph = Paragraph::new(msg).style(theme.muted()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border())
                .title(Span::styled(" Ports ", theme.title())),
        );
        f.render_widget(paragraph, area);
        return;
    }

    if app.loading && app.entries.is_empty() {
        let paragraph = Paragraph::new("⏳ Scanning ports...")
            .style(theme.info())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border())
                    .title(Span::styled(" Ports ", theme.title())),
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
        ("Tunnel", SortCol::Tunnel),
        ("Docker", SortCol::Docker),
        ("Uptime", SortCol::Uptime),
        ("CPU %", SortCol::Cpu),
        ("Mem MB", SortCol::Mem),
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
    .style(theme.header())
    .height(1);

    let visible_rows = table_visible_rows(area);
    let viewport_start = app
        .table_scroll_offset
        .min(app.filtered_entries.len().saturating_sub(visible_rows));
    let viewport_end = (viewport_start + visible_rows).min(app.filtered_entries.len());

    // Data rows
    let rows: Vec<Row> = app
        .filtered_entries
        .iter()
        .skip(viewport_start)
        .take(viewport_end.saturating_sub(viewport_start))
        .enumerate()
        .map(|(i, &idx)| {
            let entry = &app.entries[idx];
            let absolute_index = viewport_start + i;
            let is_selected = absolute_index == app.selected;

            let status_style = theme.status_style(&entry.status);

            // Get sparkline for CPU if we have history
            let cpu_display = if let Some(history) = app.resource_tracker.get(entry.pid) {
                if history.samples.len() > 1 {
                    let spark = sparkline_text(&history.cpu_values(), 8);
                    format!("{:.1}% {}", entry.cpu_percent, spark)
                } else {
                    format!("{:.1}%", entry.cpu_percent)
                }
            } else {
                format!("{:.1}%", entry.cpu_percent)
            };

            let cells = vec![
                Cell::from(format!("{}", entry.port)).style(theme.port_number()),
                Cell::from(format!("{}", entry.pid)).style(theme.muted()),
                Cell::from(entry.process_name.clone()).style(theme.process_name()),
                Cell::from(entry.project_display()).style(theme.info()),
                Cell::from(entry.git_display()).style(
                    if entry.git.as_ref().is_some_and(|g| g.dirty) {
                        theme.git_dirty()
                    } else {
                        theme.git_clean()
                    },
                ),
                Cell::from(entry.tunnel_display()).style(theme.tunnel()),
                Cell::from(entry.docker_display()).style(theme.docker()),
                Cell::from(entry.uptime_display()).style(theme.muted()),
                Cell::from(cpu_display).style(
                    if entry.cpu_percent > 50.0 {
                        theme.warning()
                    } else {
                        theme.muted()
                    },
                ),
                Cell::from(format!("{:.0}MB", entry.memory_mb.max(0.0))).style(theme.muted()),
                Cell::from(entry.status.to_string()).style(status_style),
            ];

            let style = if is_selected {
                theme.row_selected()
            } else if i % 2 == 0 {
                theme.row_normal()
            } else {
                theme.row_alt()
            };

            Row::new(cells).style(style).height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),  // Port
            Constraint::Length(8),  // PID
            Constraint::Length(15), // Process
            Constraint::Min(16),    // Project
            Constraint::Length(12), // Git
            Constraint::Length(22), // Tunnel
            Constraint::Length(14), // Docker
            Constraint::Length(9),  // Uptime
            Constraint::Length(16), // CPU (with sparkline)
            Constraint::Length(8),  // Mem
            Constraint::Length(12), // Status
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if app.view_mode == ViewMode::Search {
                theme.border_focus()
            } else {
                theme.border()
            })
            .title(Span::styled(
                format!(
                    " ◆ Active Ports ({}{}) ",
                    app.filtered_entries.len(),
                    if !app.search_query.is_empty() {
                        format!(" / {} total", app.entries.len())
                    } else {
                        String::new()
                    }
                ),
                theme.title(),
            ))
            .title_bottom(Line::from(vec![
                Span::styled(" Sort: ", theme.muted()),
                Span::styled(
                    format!(
                        "{} {}",
                        app.sort_field.label(),
                        app.sort_direction.indicator()
                    ),
                    theme.accent(),
                ),
                Span::raw(" "),
            ])),
    )
    .row_highlight_style(theme.row_selected());

    let mut state = TableState::default();
    if viewport_start <= app.selected && app.selected < viewport_end {
        state.select(Some(app.selected - viewport_start));
    }
    f.render_stateful_widget(table, area, &mut state);
}

fn table_visible_rows(area: Rect) -> usize {
    area.height.saturating_sub(4).max(1) as usize
}

/// Render detailed port inspection with sparklines.
fn render_detail(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let entry = match app.selected_entry() {
        Some(e) => e,
        None => return,
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Basic info
            Constraint::Length(5),  // Sparklines
            Constraint::Min(4),     // Extra sections
        ])
        .split(area);

    // Basic info
    let basic_lines = vec![
        Line::from(vec![
            Span::styled("  Port:      ", theme.muted()),
            Span::styled(
                format!("{}/{}", entry.port, entry.protocol),
                theme.port_number(),
            ),
        ]),
        Line::from(vec![
            Span::styled("  PID:       ", theme.muted()),
            Span::styled(format!("{}", entry.pid), theme.info()),
        ]),
        Line::from(vec![
            Span::styled("  Process:   ", theme.muted()),
            Span::styled(&entry.process_name, theme.process_name()),
        ]),
        Line::from(vec![
            Span::styled("  Command:   ", theme.muted()),
            Span::raw(&entry.command),
        ]),
        Line::from(vec![
            Span::styled("  Memory:    ", theme.muted()),
            Span::raw(format!("{:.1} MB", entry.memory_mb.max(0.0))),
        ]),
        Line::from(vec![
            Span::styled("  Status:    ", theme.muted()),
            Span::styled(entry.status.to_string(), theme.status_style(&entry.status)),
        ]),
    ];

    let basic = Paragraph::new(basic_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border_focus())
            .title(Span::styled(
                format!(" ◆ Port {} Detail ", entry.port),
                theme.title(),
            ))
            .title_bottom(Line::from(Span::styled(
                " ESC to go back │ K to kill │ t for tree ",
                theme.muted(),
            ))),
    );
    f.render_widget(basic, chunks[0]);

    // Sparklines section
    let mut sparklines_lines = Vec::new();

    if let Some(history) = app.resource_tracker.get(entry.pid) {
        let cpu_spark_str = sparkline_text(&history.cpu_values(), 40);
        let mem_spark_str = sparkline_text(&history.memory_values(), 40);

        sparklines_lines.push(Line::from(vec![
            Span::styled("  📈 CPU    ", theme.muted()),
            Span::styled(cpu_spark_str, theme.sparkline()),
            Span::styled(
                format!(" avg:{:.1}% peak:{:.1}%", history.avg_cpu(), history.peak_cpu()),
                theme.muted(),
            ),
        ]));
        sparklines_lines.push(Line::from(vec![
            Span::styled("  📈 Memory ", theme.muted()),
            Span::styled(mem_spark_str, theme.sparkline()),
            Span::styled(
                format!(" avg:{:.0}MB peak:{:.0}MB", history.avg_memory(), history.peak_memory()),
                theme.muted(),
            ),
        ]));
    } else {
        sparklines_lines.push(Line::from(Span::styled(
            "  📈 No resource history yet (collecting...)",
            theme.muted(),
        )));
    }

    let sparklines = Paragraph::new(sparklines_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border())
            .title(Span::styled(" Resource History ", theme.title())),
    );
    f.render_widget(sparklines, chunks[1]);

    // Extra sections (project, git, docker, health)
    let mut extra_lines = Vec::new();

    if let Some(ref project) = entry.project {
        extra_lines.push(Line::from(Span::styled("  📦 Project", theme.title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Kind:      ", theme.muted()),
            Span::raw(&project.kind),
        ]));
        if !project.framework.is_empty() {
            extra_lines.push(Line::from(vec![
                Span::styled("    Framework: ", theme.muted()),
                Span::styled(&project.framework, theme.info()),
            ]));
        }
        extra_lines.push(Line::from(""));
    }

    if let Some(ref git) = entry.git {
        extra_lines.push(Line::from(Span::styled("  🔀 Git", theme.title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Branch:    ", theme.muted()),
            Span::styled(
                &git.branch,
                if git.dirty {
                    theme.git_dirty()
                } else {
                    theme.git_clean()
                },
            ),
            if git.dirty {
                Span::styled(" (modified)", theme.warning())
            } else {
                Span::styled(" (clean)", theme.healthy())
            },
        ]));
        extra_lines.push(Line::from(""));
    }

    if let Some(ref docker) = entry.docker {
        extra_lines.push(Line::from(Span::styled("  🐳 Docker", theme.title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Container: ", theme.muted()),
            Span::styled(&docker.container_name, theme.docker()),
        ]));
        extra_lines.push(Line::from(vec![
            Span::styled("    Image:     ", theme.muted()),
            Span::raw(&docker.image),
        ]));
        extra_lines.push(Line::from(""));
    }

    if let Some(ref health) = entry.health_check {
        extra_lines.push(Line::from(Span::styled(
            "  🏥 Health Check",
            theme.title(),
        )));
        extra_lines.push(Line::from(vec![
            Span::styled("    Status:    ", theme.muted()),
            Span::styled(
                health.status.to_string(),
                theme.status_style(match health.status {
                    crate::models::HealthStatus::Healthy => &Status::Healthy,
                    crate::models::HealthStatus::Unhealthy => &Status::Zombie,
                    crate::models::HealthStatus::Unknown => &Status::Unknown,
                }),
            ),
        ]));
        extra_lines.push(Line::from(vec![
            Span::styled("    Latency:   ", theme.muted()),
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
                .border_style(theme.border()),
        );
    f.render_widget(extra, chunks[2]);
}

/// Render process tree view.
fn render_process_tree(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lines: Vec<Line> = app
        .process_tree
        .iter()
        .map(|entry| {
            let indent = if entry.depth == 0 {
                String::new()
            } else {
                format!("{}├─ ", "│  ".repeat(entry.depth - 1))
            };

            // Get sparkline if we have history
            let cpu_display = if let Some(history) = app.resource_tracker.get(entry.pid) {
                if history.samples.len() > 1 {
                    let spark = sparkline_text(&history.cpu_values(), 6);
                    format!("CPU: {:.1}% {}", entry.cpu_percent, spark)
                } else {
                    format!("CPU: {:.1}%", entry.cpu_percent)
                }
            } else {
                format!("CPU: {:.1}%", entry.cpu_percent)
            };

            Line::from(vec![
                Span::styled(indent, theme.border()),
                Span::styled(entry.name.clone(), theme.process_name()),
                Span::styled(format!(" (PID: {})", entry.pid), theme.muted()),
                Span::raw("  "),
                Span::styled(
                    cpu_display,
                    if entry.cpu_percent > 50.0 {
                        theme.warning()
                    } else {
                        theme.muted()
                    },
                ),
                Span::raw("  "),
                Span::styled(
                    format!("Mem: {:.1}MB", entry.memory_mb.max(0.0)),
                    theme.muted(),
                ),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border_focus())
            .title(Span::styled(" 🌲 Process Tree ", theme.title()))
            .title_bottom(Line::from(Span::styled(" ESC to go back ", theme.muted()))),
    );
    f.render_widget(paragraph, area);
}

/// Render Processes tab — sorted by CPU usage.
fn render_processes_tab(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
    // Sort entries by CPU descending for the processes view
    let mut sorted: Vec<_> = app.entries.iter().collect();
    sorted.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));

    let header = Row::new(["PID", "Process", "CPU %", "Memory MB", "Port", "Uptime"])
        .style(theme.header())
        .height(1);

    let rows: Vec<Row> = sorted.iter().enumerate().map(|(i, entry)| {
        let cpu_spark = if let Some(history) = app.resource_tracker.get(entry.pid) {
            if history.samples.len() > 1 {
                format!(" {:.1}% {}", entry.cpu_percent, sparkline_text(&history.cpu_values(), 8))
            } else {
                format!(" {:.1}%", entry.cpu_percent)
            }
        } else {
            format!(" {:.1}%", entry.cpu_percent)
        };

        let cells = vec![
            Cell::from(format!("{}", entry.pid)).style(theme.muted()),
            Cell::from(entry.process_name.clone()).style(theme.process_name()),
            Cell::from(cpu_spark).style(if entry.cpu_percent > 50.0 { theme.warning() } else { theme.muted() }),
            Cell::from(format!("{:.0}", entry.memory_mb.max(0.0))).style(theme.muted()),
            Cell::from(format!("{}", entry.port)).style(theme.port_number()),
            Cell::from(entry.uptime_display()).style(theme.muted()),
        ];

        let style = if i % 2 == 0 { theme.row_normal() } else { theme.row_alt() };
        Row::new(cells).style(style).height(1)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Length(8),
        Constraint::Min(16),
        Constraint::Length(18),
        Constraint::Length(10),
        Constraint::Length(7),
        Constraint::Length(9),
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border())
            .title(Span::styled(
                format!(" ◆ Processes ({}) sorted by CPU ", sorted.len()),
                theme.title(),
            )),
    );

    f.render_widget(table, area);
}

/// Render Docker tab — shows only Docker containers.
fn render_docker_tab(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    
    let docker_entries: Vec<_> = app.entries.iter().filter(|e| e.docker.is_some()).collect();

    if docker_entries.is_empty() {
        let paragraph = Paragraph::new("No Docker containers found.")
            .style(theme.muted())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border())
                    .title(Span::styled(" 🐳 Docker ", theme.title())),
            );
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(["Container", "Image", "Port", "Status", "Compose"])
        .style(theme.header())
        .height(1);

    let rows: Vec<Row> = docker_entries.iter().enumerate().map(|(i, entry)| {
        let docker = entry.docker.as_ref().unwrap();
        let cells = vec![
            Cell::from(docker.container_name.clone()).style(theme.docker()),
            Cell::from(docker.image.clone()).style(theme.info()),
            Cell::from(format!("{}", entry.port)).style(theme.port_number()),
            Cell::from(entry.status.to_string()).style(theme.status_style(&entry.status)),
            Cell::from(docker.compose_project.as_deref().unwrap_or("—")).style(theme.muted()),
        ];
        let style = if i % 2 == 0 { theme.row_normal() } else { theme.row_alt() };
        Row::new(cells).style(style).height(1)
    }).collect();

    let table = Table::new(rows, [
        Constraint::Min(16),
        Constraint::Min(16),
        Constraint::Length(7),
        Constraint::Length(12),
        Constraint::Min(10),
    ])
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border())
            .title(Span::styled(
                format!(" 🐳 Docker Containers ({}) ", docker_entries.len()),
                theme.title(),
            )),
    );

    f.render_widget(table, area);
}

fn render_logs_tab(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;

    let lines: Vec<Line> = if app.activity_log.is_empty() {
        vec![Line::from(Span::styled(
            "  No activity yet. Refresh, switch tabs, or run an action to populate the log.",
            theme.muted(),
        ))]
    } else {
        app.activity_log
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
            .map(|entry| Line::from(Span::styled(format!("  {}", entry), theme.info())))
            .collect()
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.border())
            .title(Span::styled(" 📋 Activity Log ", theme.title())),
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
    Tunnel,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_visible_rows_has_minimum_one() {
        assert_eq!(table_visible_rows(Rect::new(0, 0, 80, 0)), 1);
        assert_eq!(table_visible_rows(Rect::new(0, 0, 80, 4)), 1);
        assert_eq!(table_visible_rows(Rect::new(0, 0, 80, 12)), 8);
    }
}
