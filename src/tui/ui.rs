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

    // Main layout mirrors the marketing composition: a compact product masthead,
    // a command/tab rail, the live workspace, and a persistent shortcut rail.
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
        Tab::Ports => match app.view_mode {
            ViewMode::Table | ViewMode::Search => render_ports_workspace(f, chunks[2], app),
            ViewMode::Detail => render_detail(f, chunks[2], app),
            ViewMode::ProcessTree => render_process_tree(f, chunks[2], app),
            _ => render_table(f, chunks[2], app),
        },
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

/// Marketing-style ports workspace: table above a live event log and inspector.
/// On small terminals it collapses to the table so the TUI remains usable.
fn render_ports_workspace(f: &mut Frame, area: Rect, app: &App) {
    if area.width < 88 || area.height < 16 {
        render_table(f, area, app);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(64), Constraint::Percentage(36)])
        .split(area);
    render_table(f, rows[0], app);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(64), Constraint::Percentage(36)])
        .split(rows[1]);
    render_event_log(f, columns[0], app);
    render_port_summary(f, columns[1], app);
}

fn render_event_log(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let lines: Vec<Line> = if app.activity_log.is_empty() {
        vec![Line::from(Span::styled(
            "  Waiting for activity…",
            theme.muted(),
        ))]
    } else {
        app.activity_log
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
            .map(|entry| {
                Line::from(vec![
                    Span::styled("  ", theme.muted()),
                    Span::styled(entry, theme.info()),
                ])
            })
            .collect()
    };

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border())
                .title(Span::styled(" EVENTS LOG ", theme.title())),
        ),
        area,
    );
}

fn render_port_summary(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let Some(entry) = app.selected_entry() else {
        f.render_widget(
            Paragraph::new("  Select a port to inspect it.")
                .style(theme.muted())
                .block(marketing_block(" PORT DETAILS ", theme)),
            area,
        );
        return;
    };

    let network = entry
        .kubernetes
        .as_ref()
        .and_then(|k| k.bind_address.as_deref())
        .map(|address| format!("{}:{}", address, entry.port))
        .unwrap_or_else(|| format!("{} :{}", entry.protocol, entry.port));
    let details = [
        ("PROCESS", entry.display_name().to_string()),
        ("CMD", entry.command.clone()),
        ("PROJECT", entry.project_display()),
        ("CPU", format!("{:.1}%", entry.cpu_percent)),
        ("MEM", format!("{:.1} MB", entry.memory_mb.max(0.0))),
        ("NETWORK", network),
    ];
    let lines = details.into_iter().map(|(label, value)| {
        Line::from(vec![
            Span::styled(format!("  [{label:<7}] "), theme.muted()),
            Span::styled(value, theme.process_name()),
        ])
    });

    f.render_widget(
        Paragraph::new(lines.collect::<Vec<_>>())
            .wrap(Wrap { trim: true })
            .block(marketing_block(
                &format!(" PORT DETAILS [PID {}] ", entry.pid),
                theme,
            )),
        area,
    );
}

fn marketing_block<'a>(title: &'a str, theme: &crate::tui::theme::Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border())
        .title(Span::styled(title, theme.title()))
}

/// Render the tab bar.
fn render_tab_bar(f: &mut Frame, area: Rect, app: &App) {
    let theme = &app.theme;
    let tabs = [Tab::Ports, Tab::Processes, Tab::Docker, Tab::Logs];

    let spans: Vec<Span> = tabs
        .iter()
        .map(|tab| {
            if *tab == app.active_tab {
                Span::styled(format!(" {} ", tab.label()), theme.tab_active())
            } else {
                Span::styled(format!(" {} ", tab.label()), theme.tab_inactive())
            }
        })
        .collect();

    let mut line_spans = vec![Span::raw("  ")];
    for (i, span) in spans.into_iter().enumerate() {
        line_spans.push(span);
        if i < tabs.len() - 1 {
            line_spans.push(Span::styled(" │ ", theme.muted()));
        }
    }

    let bar = Paragraph::new(Line::from(line_spans)).style(Style::default().bg(theme.bg_surface()));
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
    let marketing_layout = area.width >= 118;
    let header_cells = if marketing_layout {
        vec![
            ("ID", SortCol::Index),
            ("PORT", SortCol::Port),
            ("PID", SortCol::Pid),
            ("PROCESS", SortCol::Process),
            ("CMD", SortCol::Command),
            ("HEALTH", SortCol::Status),
            ("UP TIME", SortCol::Uptime),
            ("NETWORK", SortCol::Network),
        ]
    } else {
        vec![
            ("PORT", SortCol::Port),
            ("PID", SortCol::Pid),
            ("PROCESS", SortCol::Process),
            ("PROJECT", SortCol::Project),
            ("CPU", SortCol::Cpu),
            ("MEM", SortCol::Mem),
            ("STATUS", SortCol::Status),
        ]
    };

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

            let compact_cells = vec![
                Cell::from(format!("{}", entry.port)).style(theme.port_number()),
                Cell::from(format!("{}", entry.pid)).style(theme.muted()),
                Cell::from(entry.display_name().to_string()).style(theme.process_name()),
                Cell::from(entry.project_display()).style(theme.info()),
                Cell::from(cpu_display).style(if entry.cpu_percent > 50.0 {
                    theme.warning()
                } else {
                    theme.muted()
                }),
                Cell::from(format!("{:.0}MB", entry.memory_mb.max(0.0))).style(theme.muted()),
                Cell::from(entry.status.to_string()).style(status_style),
            ];

            let cells = if marketing_layout {
                vec![
                    Cell::from(format!("{}", absolute_index + 1)).style(theme.muted()),
                    Cell::from(format!("{}", entry.port)).style(theme.port_number()),
                    Cell::from(format!("{}", entry.pid)).style(theme.muted()),
                    Cell::from(entry.display_name().to_string()).style(theme.process_name()),
                    Cell::from(entry.command.clone()).style(theme.info()),
                    Cell::from(entry.status.to_string()).style(status_style),
                    Cell::from(entry.uptime_display()).style(theme.muted()),
                    Cell::from(format!("{} :{}", entry.protocol, entry.port)).style(theme.muted()),
                ]
            } else {
                compact_cells
            };

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

    let widths = if marketing_layout {
        vec![
            Constraint::Length(4),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Min(18),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(17),
        ]
    } else {
        vec![
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(16),
            Constraint::Min(14),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Length(15),
        ]
    };
    let table = Table::new(rows, widths)
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
                        " ACTIVE PORTS: {}{} ",
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
            Constraint::Length(8), // Basic info
            Constraint::Length(5), // Sparklines
            Constraint::Min(4),    // Extra sections
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
            Span::styled(entry.display_name(), theme.process_name()),
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
                format!(
                    " avg:{:.1}% peak:{:.1}%",
                    history.avg_cpu(),
                    history.peak_cpu()
                ),
                theme.muted(),
            ),
        ]));
        sparklines_lines.push(Line::from(vec![
            Span::styled("  📈 Memory ", theme.muted()),
            Span::styled(mem_spark_str, theme.sparkline()),
            Span::styled(
                format!(
                    " avg:{:.0}MB peak:{:.0}MB",
                    history.avg_memory(),
                    history.peak_memory()
                ),
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

    // Extra sections (project, git, docker, Kubernetes, health)
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

    if let Some(ref kubernetes) = entry.kubernetes {
        extra_lines.push(Line::from(Span::styled("  ☸ Kubernetes", theme.title())));
        extra_lines.push(Line::from(vec![
            Span::styled("    Resource:  ", theme.muted()),
            Span::styled(kubernetes.resource_display(), theme.info()),
        ]));
        if let Some(namespace) = &kubernetes.namespace {
            extra_lines.push(Line::from(vec![
                Span::styled("    Namespace: ", theme.muted()),
                Span::raw(namespace),
            ]));
        }
        if let Some(context) = &kubernetes.context {
            extra_lines.push(Line::from(vec![
                Span::styled("    Context:   ", theme.muted()),
                Span::raw(context),
            ]));
        }
        extra_lines.push(Line::from(vec![
            Span::styled("    Ports:     ", theme.muted()),
            Span::raw(match kubernetes.remote_port {
                Some(remote_port) => format!("{} -> {}", kubernetes.local_port, remote_port),
                None => kubernetes.local_port.to_string(),
            }),
        ]));
        if let Some(bind_address) = &kubernetes.bind_address {
            extra_lines.push(Line::from(vec![
                Span::styled("    Address:   ", theme.muted()),
                Span::raw(bind_address),
            ]));
        }
        extra_lines.push(Line::from(""));
    }

    if let Some(ref health) = entry.health_check {
        extra_lines.push(Line::from(Span::styled("  🏥 Health Check", theme.title())));
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
    sorted.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let header = Row::new(["PID", "Process", "CPU %", "Memory MB", "Port", "Uptime"])
        .style(theme.header())
        .height(1);

    let rows: Vec<Row> = sorted
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let cpu_spark = if let Some(history) = app.resource_tracker.get(entry.pid) {
                if history.samples.len() > 1 {
                    format!(
                        " {:.1}% {}",
                        entry.cpu_percent,
                        sparkline_text(&history.cpu_values(), 8)
                    )
                } else {
                    format!(" {:.1}%", entry.cpu_percent)
                }
            } else {
                format!(" {:.1}%", entry.cpu_percent)
            };

            let cells = vec![
                Cell::from(format!("{}", entry.pid)).style(theme.muted()),
                Cell::from(entry.display_name().to_string()).style(theme.process_name()),
                Cell::from(cpu_spark).style(if entry.cpu_percent > 50.0 {
                    theme.warning()
                } else {
                    theme.muted()
                }),
                Cell::from(format!("{:.0}", entry.memory_mb.max(0.0))).style(theme.muted()),
                Cell::from(format!("{}", entry.port)).style(theme.port_number()),
                Cell::from(entry.uptime_display()).style(theme.muted()),
            ];

            let style = if i % 2 == 0 {
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
            Constraint::Length(8),
            Constraint::Min(16),
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(9),
        ],
    )
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

    let rows: Vec<Row> = docker_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let docker = entry.docker.as_ref().unwrap();
            let cells = vec![
                Cell::from(docker.container_name.clone()).style(theme.docker()),
                Cell::from(docker.image.clone()).style(theme.info()),
                Cell::from(format!("{}", entry.port)).style(theme.port_number()),
                Cell::from(entry.status.to_string()).style(theme.status_style(&entry.status)),
                Cell::from(docker.compose_project.as_deref().unwrap_or("—")).style(theme.muted()),
            ];
            let style = if i % 2 == 0 {
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
            Constraint::Min(16),
            Constraint::Min(16),
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Min(10),
        ],
    )
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
    Index,
    Port,
    Pid,
    Process,
    Command,
    Project,
    Uptime,
    Mem,
    Cpu,
    Status,
    Network,
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
    use crate::config::PortForgeConfig;
    use crate::models::{PortEntry, Protocol};
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn test_table_visible_rows_has_minimum_one() {
        assert_eq!(table_visible_rows(Rect::new(0, 0, 80, 0)), 1);
        assert_eq!(table_visible_rows(Rect::new(0, 0, 80, 4)), 1);
        assert_eq!(table_visible_rows(Rect::new(0, 0, 80, 12)), 8);
    }

    fn sample_app() -> App {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.loading = false;
        app.entries = vec![PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 18452,
            label: None,
            process_name: "node".into(),
            command: "node server.js".into(),
            cwd: None,
            memory_mb: 88.4,
            cpu_percent: 1.2,
            uptime_secs: 862,
            project: None,
            docker: None,
            git: None,
            tunnel: None,
            kubernetes: None,
            status: Status::Healthy,
            health_check: None,
        }];
        app.filtered_entries = vec![0];
        app.activity_log
            .push_back("[16:01:22] Port 3000 active".into());
        app
    }

    fn rendered_text(width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        let app = sample_app();
        terminal.draw(|frame| render(frame, &app)).expect("render");
        let buffer = terminal.backend().buffer();
        (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn marketing_workspace_renders_reference_hierarchy() {
        let output = rendered_text(140, 32);
        if std::env::var_os("PORTFORGE_PRINT_RENDER").is_some() {
            eprintln!("{output}");
        }
        assert!(output.contains("PortForge v"));
        assert!(output.contains("ACTIVE PORTS: 1"));
        assert!(output.contains("PROCESS"));
        assert!(output.contains("HEALTH"));
        assert!(output.contains("UP TIME"));
        assert!(output.contains("NETWORK"));
        assert!(output.contains("EVENTS LOG"));
        assert!(output.contains("PORT DETAILS [PID 18452]"));
        assert!(output.contains("node server.js"));
        assert!(output.contains("TCP :3000"));
    }

    #[test]
    fn compact_terminal_omits_secondary_panels_without_losing_ports() {
        let output = rendered_text(80, 14);
        assert!(output.contains("ACTIVE PORTS: 1"));
        assert!(output.contains("node"));
        assert!(!output.contains("EVENTS LOG"));
        assert!(!output.contains("PORT DETAILS"));
    }
}
