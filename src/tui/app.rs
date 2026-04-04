use crate::config::PortForgeConfig;
use crate::error::{PortForgeError, Result};
use crate::models::*;
use crate::process;
use crate::scanner;
use crate::tui::ui;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

/// TUI application view mode.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    /// Main table view.
    Table,
    /// Detailed inspection of selected port.
    Detail,
    /// Process tree for selected port.
    ProcessTree,
    /// Search mode (typing search query).
    Search,
    /// Help overlay.
    Help,
    /// Kill confirmation dialog.
    KillConfirm,
}

/// TUI application state.
pub struct App {
    /// Configuration.
    pub config: PortForgeConfig,
    /// Whether to show all ports.
    pub show_all: bool,
    /// Current port entries.
    pub entries: Vec<PortEntry>,
    /// Filtered entries (after search).
    pub filtered_entries: Vec<usize>,
    /// Currently selected row index (into filtered_entries).
    pub selected: usize,
    /// Current view mode.
    pub view_mode: ViewMode,
    /// Sort field.
    pub sort_field: SortField,
    /// Sort direction.
    pub sort_direction: SortDirection,
    /// Search query.
    pub search_query: String,
    /// Refresh interval in seconds.
    pub refresh_interval: u64,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Process tree data for detail view.
    pub process_tree: Vec<process::ProcessTreeEntry>,
    /// Last scan time.
    pub last_scan: Option<Instant>,
    /// Status message (bottom bar).
    pub status_message: Option<(String, Instant)>,
    /// Whether data is currently loading.
    pub loading: bool,
    /// Scroll offset for table.
    pub table_scroll_offset: usize,
}

impl App {
    pub fn new(config: PortForgeConfig, show_all: bool) -> Self {
        let refresh = config.general.refresh_interval;
        Self {
            config,
            show_all,
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            selected: 0,
            view_mode: ViewMode::Table,
            sort_field: SortField::Port,
            sort_direction: SortDirection::Ascending,
            search_query: String::new(),
            refresh_interval: refresh,
            should_quit: false,
            process_tree: Vec::new(),
            last_scan: None,
            status_message: None,
            loading: true,
            table_scroll_offset: 0,
        }
    }

    pub fn set_refresh_interval(&mut self, interval: u64) {
        self.refresh_interval = interval;
    }

    /// Run the TUI application event loop.
    pub async fn run(&mut self) -> Result<()> {
        // Terminal setup
        enable_raw_mode().map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal =
            Terminal::new(backend).map_err(|e| PortForgeError::TuiError(e.to_string()))?;

        // Initial scan
        self.refresh_data().await;

        let tick_rate = Duration::from_millis(100);
        let refresh_duration = Duration::from_secs(self.refresh_interval);
        let mut last_refresh = Instant::now();

        // Event loop
        loop {
            // Draw
            terminal
                .draw(|f| ui::render(f, self))
                .map_err(|e| PortForgeError::TuiError(e.to_string()))?;

            // Handle input
            if event::poll(tick_rate).map_err(|e| PortForgeError::TuiError(e.to_string()))? {
                if let Event::Key(key) = event::read().map_err(|e| PortForgeError::TuiError(e.to_string()))? {
                    self.handle_key_event(key);
                }
            }

            // Auto-refresh
            if last_refresh.elapsed() >= refresh_duration {
                self.refresh_data().await;
                last_refresh = Instant::now();
            }

            // Clear expired status messages (after 3 seconds)
            if let Some((_, created)) = &self.status_message {
                if created.elapsed() > Duration::from_secs(3) {
                    self.status_message = None;
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Terminal cleanup
        disable_raw_mode().map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        terminal
            .show_cursor()
            .map_err(|e| PortForgeError::TuiError(e.to_string()))?;

        Ok(())
    }

    /// Refresh port data.
    async fn refresh_data(&mut self) {
        self.loading = true;
        match scanner::scan_ports(&self.config, self.show_all).await {
            Ok(mut entries) => {
                scanner::sort_entries(&mut entries, self.sort_field, self.sort_direction);
                self.entries = entries;
                self.apply_filter();
                self.loading = false;
                self.last_scan = Some(Instant::now());
            }
            Err(e) => {
                self.loading = false;
                self.set_status(format!("Scan error: {}", e));
            }
        }
    }

    /// Apply search filter and rebuild filtered indices.
    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_entries = (0..self.entries.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_entries = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.port.to_string().contains(&query)
                        || e.process_name.to_lowercase().contains(&query)
                        || e.project_display().to_lowercase().contains(&query)
                        || e.git_display().to_lowercase().contains(&query)
                        || e.docker_display().to_lowercase().contains(&query)
                        || e.command.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Adjust selection
        if self.selected >= self.filtered_entries.len() {
            self.selected = self.filtered_entries.len().saturating_sub(1);
        }
    }

    /// Get the currently selected entry.
    pub fn selected_entry(&self) -> Option<&PortEntry> {
        self.filtered_entries
            .get(self.selected)
            .and_then(|&idx| self.entries.get(idx))
    }

    /// Set a status message.
    fn set_status(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }

    /// Handle keyboard input.
    fn handle_key_event(&mut self, key: KeyEvent) {
        // Global: Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match self.view_mode {
            ViewMode::Table => self.handle_table_keys(key),
            ViewMode::Detail => self.handle_detail_keys(key),
            ViewMode::ProcessTree => self.handle_tree_keys(key),
            ViewMode::Search => self.handle_search_keys(key),
            ViewMode::Help => self.handle_help_keys(key),
            ViewMode::KillConfirm => self.handle_kill_confirm_keys(key),
        }
    }

    fn handle_table_keys(&mut self, key: KeyEvent) {
        match key.code {
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => self.selected = 0,
            KeyCode::Char('G') => {
                self.selected = self.filtered_entries.len().saturating_sub(1);
            }
            KeyCode::Home => self.selected = 0,
            KeyCode::End => {
                self.selected = self.filtered_entries.len().saturating_sub(1);
            }
            KeyCode::PageDown => self.move_selection(20),
            KeyCode::PageUp => self.move_selection(-20),

            // Actions
            KeyCode::Enter | KeyCode::Char('d') => {
                if self.selected_entry().is_some() {
                    self.view_mode = ViewMode::Detail;
                }
            }
            KeyCode::Char('t') => {
                if let Some(entry) = self.selected_entry() {
                    self.process_tree = process::get_process_tree(entry.pid);
                    self.view_mode = ViewMode::ProcessTree;
                }
            }
            KeyCode::Char('K') => {
                if self.selected_entry().is_some() {
                    self.view_mode = ViewMode::KillConfirm;
                }
            }
            KeyCode::Char('/') => {
                self.view_mode = ViewMode::Search;
            }
            KeyCode::Char('?') => {
                self.view_mode = ViewMode::Help;
            }
            KeyCode::Char('a') => {
                self.show_all = !self.show_all;
                self.set_status(format!(
                    "Showing {}",
                    if self.show_all { "all ports" } else { "dev ports" }
                ));
            }

            // Sorting
            KeyCode::Char('1') => self.toggle_sort(SortField::Port),
            KeyCode::Char('2') => self.toggle_sort(SortField::Pid),
            KeyCode::Char('3') => self.toggle_sort(SortField::Process),
            KeyCode::Char('4') => self.toggle_sort(SortField::Project),
            KeyCode::Char('5') => self.toggle_sort(SortField::Memory),
            KeyCode::Char('6') => self.toggle_sort(SortField::Cpu),
            KeyCode::Char('7') => self.toggle_sort(SortField::Uptime),
            KeyCode::Char('8') => self.toggle_sort(SortField::Status),

            // Refresh
            KeyCode::Char('r') => {
                self.set_status("Refreshing...".to_string());
            }

            _ => {}
        }
    }

    fn handle_search_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                self.apply_filter();
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Enter => {
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_filter();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_filter();
            }
            _ => {}
        }
    }

    fn handle_detail_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Char('K') => {
                self.view_mode = ViewMode::KillConfirm;
            }
            KeyCode::Char('t') => {
                if let Some(entry) = self.selected_entry() {
                    self.process_tree = process::get_process_tree(entry.pid);
                    self.view_mode = ViewMode::ProcessTree;
                }
            }
            _ => {}
        }
    }

    fn handle_tree_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.view_mode = ViewMode::Table;
            }
            _ => {}
        }
    }

    fn handle_help_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.view_mode = ViewMode::Table;
            }
            _ => {}
        }
    }

    fn handle_kill_confirm_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(entry) = self.selected_entry().cloned() {
                    match process::kill_process(&entry, false) {
                        Ok(()) => {
                            self.set_status(format!(
                                "✓ Killed PID {} on port {}",
                                entry.pid, entry.port
                            ));
                        }
                        Err(e) => {
                            self.set_status(format!("✗ Failed to kill: {}", e));
                        }
                    }
                }
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                if let Some(entry) = self.selected_entry().cloned() {
                    match process::kill_process(&entry, true) {
                        Ok(()) => {
                            self.set_status(format!(
                                "✓ Force killed PID {} on port {}",
                                entry.pid, entry.port
                            ));
                        }
                        Err(e) => {
                            self.set_status(format!("✗ Failed to kill: {}", e));
                        }
                    }
                }
                self.view_mode = ViewMode::Table;
            }
            _ => {
                self.view_mode = ViewMode::Table;
            }
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.filtered_entries.len();
        if len == 0 {
            self.selected = 0;
            return;
        }

        if delta > 0 {
            self.selected = (self.selected + delta as usize).min(len - 1);
        } else {
            self.selected = self.selected.saturating_sub((-delta) as usize);
        }
    }

    fn toggle_sort(&mut self, field: SortField) {
        if self.sort_field == field {
            self.sort_direction = self.sort_direction.toggle();
        } else {
            self.sort_field = field;
            self.sort_direction = SortDirection::Ascending;
        }
        scanner::sort_entries(&mut self.entries, self.sort_field, self.sort_direction);
        self.apply_filter();
    }
}
