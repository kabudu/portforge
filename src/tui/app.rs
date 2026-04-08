use crate::config::PortForgeConfig;
use crate::error::{PortForgeError, Result};
use crate::models::*;
use crate::process;
use crate::resource_history::ResourceTracker;
use crate::scanner;
use crate::tui::theme::{Theme, ThemeName};
use crate::tui::ui;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
    },
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::collections::VecDeque;
use std::io;
use std::time::{Duration, Instant};

const HEADER_ROWS: u16 = 3;
const TAB_ROWS: u16 = 1;
const STATUS_ROWS: u16 = 1;
const TABLE_CHROME_ROWS: u16 = 4;
const TABLE_DATA_START_OFFSET: u16 = 2;
const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(500);

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

/// Tab-based view categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Ports,
    Processes,
    Docker,
    Logs,
}

impl Tab {
    pub fn label(&self) -> &'static str {
        match self {
            Tab::Ports => "Ports",
            Tab::Processes => "Processes",
            Tab::Docker => "Docker",
            Tab::Logs => "Logs",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Tab::Ports => Tab::Processes,
            Tab::Processes => Tab::Docker,
            Tab::Docker => Tab::Logs,
            Tab::Logs => Tab::Ports,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Tab::Ports => Tab::Logs,
            Tab::Processes => Tab::Ports,
            Tab::Docker => Tab::Processes,
            Tab::Logs => Tab::Docker,
        }
    }
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
    /// Recent activity log entries.
    pub activity_log: VecDeque<String>,
    /// Whether data is currently loading.
    pub loading: bool,
    /// Scroll offset for table.
    pub table_scroll_offset: usize,
    /// Current theme.
    pub theme: Theme,
    /// Resource history tracker.
    pub resource_tracker: ResourceTracker,
    /// Current active tab.
    pub active_tab: Tab,
    /// Whether mouse support is enabled.
    pub mouse_enabled: bool,
    /// Visible rows in the table (for mouse click calculations).
    pub visible_rows: usize,
    /// View to return to when a modal is cancelled.
    pub modal_return_view: ViewMode,
    /// Most recent table click for double-click detection.
    pub last_table_click: Option<(usize, Instant)>,
}

impl App {
    pub fn new(config: PortForgeConfig, show_all: bool) -> Self {
        let refresh = config.general.refresh_interval;
        let theme_name = config.general.theme.parse().unwrap_or(ThemeName::Dark);
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
            activity_log: VecDeque::with_capacity(128),
            loading: true,
            table_scroll_offset: 0,
            theme: Theme::new(theme_name),
            resource_tracker: ResourceTracker::new(),
            active_tab: Tab::Ports,
            mouse_enabled: true,
            visible_rows: 20,
            modal_return_view: ViewMode::Table,
            last_table_click: None,
        }
    }

    pub fn set_refresh_interval(&mut self, interval: u64) {
        self.refresh_interval = interval;
    }

    /// Cycle to the next theme.
    pub fn next_theme(&mut self) {
        let all = ThemeName::all();
        let current_idx = all
            .iter()
            .position(|t| *t == self.theme.name())
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % all.len();
        self.theme = Theme::new(all[next_idx]);
        self.set_status(format!("Theme: {}", self.theme.name().as_str()));
    }

    /// Run the TUI application event loop.
    pub async fn run(&mut self) -> Result<()> {
        // Terminal setup
        enable_raw_mode().map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        let mut stdout = io::stdout();
        if self.mouse_enabled {
            execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
                .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        } else {
            execute!(stdout, EnterAlternateScreen)
                .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        }
        let backend = CrosstermBackend::new(stdout);
        let mut terminal =
            Terminal::new(backend).map_err(|e| PortForgeError::TuiError(e.to_string()))?;

        // Initial scan
        self.refresh_data().await;

        let tick_rate = Duration::from_millis(100);
        let refresh_duration = Duration::from_secs(self.refresh_interval);
        let mut last_refresh = Instant::now();
        let mut mouse_capture_enabled = self.mouse_enabled;

        // Event loop
        loop {
            if self.mouse_enabled != mouse_capture_enabled {
                if self.mouse_enabled {
                    execute!(terminal.backend_mut(), EnableMouseCapture)
                        .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
                } else {
                    execute!(terminal.backend_mut(), DisableMouseCapture)
                        .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
                }
                mouse_capture_enabled = self.mouse_enabled;
            }

            // Draw
            terminal
                .draw(|f| ui::render(f, self))
                .map_err(|e| PortForgeError::TuiError(e.to_string()))?;

            // Handle input
            if event::poll(tick_rate).map_err(|e| PortForgeError::TuiError(e.to_string()))? {
                match event::read().map_err(|e| PortForgeError::TuiError(e.to_string()))? {
                    Event::Key(key) => self.handle_key_event(key).await,
                    Event::Mouse(mouse) if self.mouse_enabled => self.handle_mouse_event(mouse),
                    _ => {}
                }
            }

            // Auto-refresh
            if last_refresh.elapsed() >= refresh_duration {
                self.refresh_data().await;
                last_refresh = Instant::now();
            }

            // Collect resource samples
            self.collect_resource_samples();

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
        if mouse_capture_enabled {
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )
            .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        } else {
            execute!(terminal.backend_mut(), LeaveAlternateScreen)
                .map_err(|e| PortForgeError::TuiError(e.to_string()))?;
        }
        terminal
            .show_cursor()
            .map_err(|e| PortForgeError::TuiError(e.to_string()))?;

        Ok(())
    }

    /// Collect resource samples for all entries.
    fn collect_resource_samples(&mut self) {
        let entries: Vec<(u32, f32, f64)> = self
            .entries
            .iter()
            .filter(|entry| {
                self.resource_tracker
                    .get(entry.pid)
                    .map(|history| history.should_sample())
                    .unwrap_or(true)
            })
            .map(|e| (e.pid, e.cpu_percent, e.memory_mb))
            .collect();
        self.resource_tracker.record_batch(&entries);

        // Prune old histories
        let active_pids: std::collections::HashSet<u32> =
            self.entries.iter().map(|e| e.pid).collect();
        self.resource_tracker.prune(&active_pids);
    }

    /// Handle mouse events.
    fn handle_mouse_event(&mut self, mouse: MouseEvent) {
        if self.active_tab != Tab::Ports
            || !matches!(self.view_mode, ViewMode::Table | ViewMode::Search)
        {
            return;
        }

        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.move_selection(-3);
            }
            MouseEventKind::ScrollDown => {
                self.move_selection(3);
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                if let Ok((_, terminal_rows)) = size() {
                    if let Some(clicked_row) =
                        clicked_table_index(mouse.row, terminal_rows, self.table_scroll_offset)
                    {
                        self.handle_table_click(clicked_row, Instant::now());
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_table_click(&mut self, clicked_row: usize, now: Instant) {
        if clicked_row >= self.filtered_entries.len() {
            return;
        }

        let is_double_click = self
            .last_table_click
            .as_ref()
            .map(|(last_row, last_at)| {
                *last_row == clicked_row && now.duration_since(*last_at) <= DOUBLE_CLICK_WINDOW
            })
            .unwrap_or(false);

        self.selected = clicked_row;

        if is_double_click {
            if self.selected_entry().is_some() {
                self.last_table_click = None;
                self.view_mode = ViewMode::Detail;
            }
            return;
        }

        self.last_table_click = Some((clicked_row, now));
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
                self.push_activity(format!(
                    "Scan completed: {} ports loaded{}",
                    self.entries.len(),
                    if self.show_all { " [all]" } else { " [dev]" }
                ));
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
                        || e.tunnel_display().to_lowercase().contains(&query)
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

        let max_offset = self
            .filtered_entries
            .len()
            .saturating_sub(self.visible_rows.max(1));
        if self.table_scroll_offset > max_offset {
            self.table_scroll_offset = max_offset;
        }
    }

    fn reset_viewport_to_selection(&mut self) {
        self.table_scroll_offset = self.selected.saturating_sub(self.visible_rows.max(1) / 2);
    }

    fn focus_first_filtered_entry(&mut self) {
        self.selected = 0;
        self.table_scroll_offset = 0;
    }

    /// Get the currently selected entry.
    pub fn selected_entry(&self) -> Option<&PortEntry> {
        self.filtered_entries
            .get(self.selected)
            .and_then(|&idx| self.entries.get(idx))
    }

    /// Set a status message.
    fn set_status(&mut self, message: String) {
        self.push_activity(message.clone());
        self.status_message = Some((message, Instant::now()));
    }

    fn push_activity(&mut self, message: String) {
        let timestamp = current_time_label();
        if self.activity_log.len() >= 128 {
            self.activity_log.pop_front();
        }
        self.activity_log
            .push_back(format!("[{}] {}", timestamp, message));
    }

    /// Handle keyboard input.
    async fn handle_key_event(&mut self, key: KeyEvent) {
        // Global: Ctrl+C always quits
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }

        match self.view_mode {
            ViewMode::Table => self.handle_table_keys(key).await,
            ViewMode::Detail => self.handle_detail_keys(key).await,
            ViewMode::ProcessTree => self.handle_tree_keys(key).await,
            ViewMode::Search => self.handle_search_keys(key).await,
            ViewMode::Help => self.handle_help_keys(key).await,
            ViewMode::KillConfirm => self.handle_kill_confirm_keys(key).await,
        }
    }

    async fn handle_table_keys(&mut self, key: KeyEvent) {
        match key.code {
            // Quit
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(-1),
            KeyCode::Char('g') => {
                self.selected = 0;
                self.table_scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                self.selected = self.filtered_entries.len().saturating_sub(1);
                self.table_scroll_offset = self.selected.saturating_sub(10);
            }
            KeyCode::Home => {
                self.selected = 0;
                self.table_scroll_offset = 0;
            }
            KeyCode::End => {
                self.selected = self.filtered_entries.len().saturating_sub(1);
                self.table_scroll_offset = self.selected.saturating_sub(10);
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
                    self.modal_return_view = ViewMode::Table;
                    self.view_mode = ViewMode::KillConfirm;
                }
            }
            KeyCode::Char('/') => {
                self.view_mode = ViewMode::Search;
            }
            KeyCode::Char('?') => {
                self.view_mode = ViewMode::Help;
            }
            KeyCode::Char('a') | KeyCode::Char('A') => {
                self.show_all = !self.show_all;
                self.set_status(format!(
                    "Showing {}",
                    if self.show_all {
                        "all ports"
                    } else {
                        "dev ports"
                    }
                ));
                self.refresh_data().await;
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
                self.refresh_data().await;
            }

            // Tab navigation
            KeyCode::Tab => {
                self.active_tab = self.active_tab.next();
                self.push_activity(format!("Switched to {} tab", self.active_tab.label()));
            }
            KeyCode::BackTab => {
                self.active_tab = self.active_tab.prev();
                self.push_activity(format!("Switched to {} tab", self.active_tab.label()));
            }

            // Theme cycling
            KeyCode::Char('T') => {
                self.next_theme();
            }

            // Toggle mouse support
            KeyCode::Char('m') => {
                self.mouse_enabled = !self.mouse_enabled;
                self.set_status(format!(
                    "Mouse {}",
                    if self.mouse_enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                ));
            }

            _ => {}
        }
    }

    async fn handle_search_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                self.apply_filter();
                self.focus_first_filtered_entry();
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Enter => {
                self.reset_viewport_to_selection();
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.apply_filter();
                self.focus_first_filtered_entry();
            }
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(self.visible_rows as i32),
            KeyCode::PageUp => self.move_selection(-(self.visible_rows as i32)),
            KeyCode::Home | KeyCode::Char('g') => {
                self.selected = 0;
                self.table_scroll_offset = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.selected = self.filtered_entries.len().saturating_sub(1);
                self.reset_viewport_to_selection();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.apply_filter();
                self.focus_first_filtered_entry();
            }
            _ => {}
        }
    }

    async fn handle_detail_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Char('K') => {
                self.modal_return_view = ViewMode::Detail;
                self.view_mode = ViewMode::KillConfirm;
            }
            KeyCode::Char('t') => {
                if let Some(entry) = self.selected_entry() {
                    self.process_tree = process::get_process_tree(entry.pid);
                    self.view_mode = ViewMode::ProcessTree;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
            }
            _ => {}
        }
    }

    async fn handle_tree_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.view_mode = ViewMode::Table;
            }
            _ => {}
        }
    }

    async fn handle_help_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => {
                self.view_mode = ViewMode::Table;
            }
            _ => {}
        }
    }

    async fn handle_kill_confirm_keys(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                if let Some(entry) = self.selected_entry().cloned() {
                    match process::kill_process(&entry, false) {
                        Ok(()) => {
                            self.set_status(format!(
                                "✓ Killed PID {} on port {}",
                                entry.pid, entry.port
                            ));
                            self.refresh_data().await;
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
                            self.refresh_data().await;
                        }
                        Err(e) => {
                            self.set_status(format!("✗ Failed to kill: {}", e));
                        }
                    }
                }
                self.view_mode = ViewMode::Table;
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') | KeyCode::Char('N') => {
                self.view_mode = self.modal_return_view.clone();
            }
            _ => {}
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.filtered_entries.len();
        if len == 0 {
            self.selected = 0;
            return;
        }

        let visible_rows = self.visible_rows.max(1);

        if delta > 0 {
            self.selected = (self.selected + delta as usize).min(len - 1);
            // Auto-scroll when selection goes below visible area
            if self.selected >= self.table_scroll_offset + visible_rows {
                self.table_scroll_offset = self.selected + 1 - visible_rows;
            }
        } else {
            self.selected = self.selected.saturating_sub((-delta) as usize);
            // Scroll up when selection moves above visible area
            if self.selected < self.table_scroll_offset {
                self.table_scroll_offset = self.selected;
            }
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

fn table_data_bounds(terminal_rows: u16) -> Option<(u16, u16)> {
    let content_height = terminal_rows.saturating_sub(HEADER_ROWS + TAB_ROWS + STATUS_ROWS);
    let visible_rows = content_height.saturating_sub(TABLE_CHROME_ROWS);
    if visible_rows == 0 {
        return None;
    }

    let start = HEADER_ROWS + TAB_ROWS + TABLE_DATA_START_OFFSET;
    Some((start, start + visible_rows))
}

fn clicked_table_index(mouse_row: u16, terminal_rows: u16, scroll_offset: usize) -> Option<usize> {
    let (start, end) = table_data_bounds(terminal_rows)?;
    if mouse_row < start || mouse_row >= end {
        return None;
    }

    Some(scroll_offset + (mouse_row - start) as usize)
}

fn current_time_label() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs = now % 86_400;
    let hours = secs / 3_600;
    let minutes = (secs % 3_600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(pid: u32) -> PortEntry {
        PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            pid,
            process_name: "node".to_string(),
            command: "node server.js".to_string(),
            cwd: None,
            memory_mb: 12.0,
            cpu_percent: 1.0,
            uptime_secs: 5,
            project: None,
            docker: None,
            git: None,
            tunnel: None,
            status: Status::Healthy,
            health_check: None,
        }
    }

    #[test]
    fn test_collect_resource_samples_respects_sampling_interval() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(42)];

        app.collect_resource_samples();
        app.collect_resource_samples();

        let history = app.resource_tracker.get(42).unwrap();
        assert_eq!(history.samples.len(), 1);
    }

    #[test]
    fn test_set_status_records_activity() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.set_status("Theme: nord".to_string());

        assert_eq!(app.activity_log.len(), 1);
        assert!(app.activity_log.back().unwrap().contains("Theme: nord"));
    }

    #[test]
    fn test_move_selection_uses_visible_rows_for_scroll() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.filtered_entries = (0..20).collect();
        app.visible_rows = 5;

        app.move_selection(6);

        assert_eq!(app.selected, 6);
        assert_eq!(app.table_scroll_offset, 2);
    }

    #[test]
    fn test_apply_filter_clamps_offset_after_results_shrink() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(1), sample_entry(2), sample_entry(3)];
        app.visible_rows = 2;
        app.filtered_entries = vec![0, 1, 2];
        app.table_scroll_offset = 2;
        app.search_query = "3000".to_string();

        app.apply_filter();

        assert_eq!(app.filtered_entries.len(), 3);
        assert_eq!(app.table_scroll_offset, 1);
    }

    #[tokio::test]
    async fn test_kill_confirm_cancel_returns_to_detail() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(42)];
        app.filtered_entries = vec![0];
        app.view_mode = ViewMode::Detail;
        app.modal_return_view = ViewMode::Detail;

        app.handle_kill_confirm_keys(KeyEvent::from(KeyCode::Esc))
            .await;

        assert_eq!(app.view_mode, ViewMode::Detail);
    }

    #[tokio::test]
    async fn test_search_navigation_keeps_search_mode() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(1), sample_entry(2), sample_entry(3)];
        app.filtered_entries = vec![0, 1, 2];
        app.view_mode = ViewMode::Search;

        app.handle_search_keys(KeyEvent::from(KeyCode::Down)).await;

        assert_eq!(app.selected, 1);
        assert_eq!(app.view_mode, ViewMode::Search);
    }

    #[test]
    fn test_clicked_table_index_maps_visible_rows_only() {
        assert_eq!(clicked_table_index(6, 30, 0), Some(0));
        assert_eq!(clicked_table_index(7, 30, 0), Some(1));
        assert_eq!(clicked_table_index(6, 30, 5), Some(5));
    }

    #[test]
    fn test_clicked_table_index_ignores_non_data_rows() {
        assert_eq!(clicked_table_index(5, 30, 0), None);
        assert_eq!(clicked_table_index(0, 30, 0), None);
        assert_eq!(clicked_table_index(29, 30, 0), None);
    }

    #[test]
    fn test_table_data_bounds_returns_none_for_tiny_terminal() {
        assert_eq!(table_data_bounds(5), None);
    }

    #[test]
    fn test_single_click_only_selects_row() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(1), sample_entry(2)];
        app.filtered_entries = vec![0, 1];

        let now = Instant::now();
        app.handle_table_click(1, now);

        assert_eq!(app.selected, 1);
        assert_eq!(app.view_mode, ViewMode::Table);
        assert_eq!(app.last_table_click.map(|(row, _)| row), Some(1));
    }

    #[test]
    fn test_double_click_opens_detail_for_same_row() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(1), sample_entry(2)];
        app.filtered_entries = vec![0, 1];

        let now = Instant::now();
        app.handle_table_click(1, now);
        app.handle_table_click(1, now + Duration::from_millis(200));

        assert_eq!(app.selected, 1);
        assert_eq!(app.view_mode, ViewMode::Detail);
        assert!(app.last_table_click.is_none());
    }

    #[test]
    fn test_second_click_after_timeout_does_not_open_detail() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(1), sample_entry(2)];
        app.filtered_entries = vec![0, 1];

        let now = Instant::now();
        app.handle_table_click(1, now);
        app.handle_table_click(1, now + DOUBLE_CLICK_WINDOW + Duration::from_millis(1));

        assert_eq!(app.view_mode, ViewMode::Table);
        assert_eq!(app.last_table_click.map(|(row, _)| row), Some(1));
    }

    #[test]
    fn test_clicking_different_row_resets_double_click_target() {
        let mut app = App::new(PortForgeConfig::default(), true);
        app.entries = vec![sample_entry(1), sample_entry(2), sample_entry(3)];
        app.filtered_entries = vec![0, 1, 2];

        let now = Instant::now();
        app.handle_table_click(0, now);
        app.handle_table_click(1, now + Duration::from_millis(200));

        assert_eq!(app.selected, 1);
        assert_eq!(app.view_mode, ViewMode::Table);
        assert_eq!(app.last_table_click.map(|(row, _)| row), Some(1));
    }
}
