use crate::error::{PortForgeError, Result};
use crate::models::PortEntry;
use sysinfo::{Pid, Signal, System};
use tracing::{info, warn};

/// Kill the process listening on the given port with retry logic.
pub fn kill_process(entry: &PortEntry, force: bool) -> Result<()> {
    let pid = Pid::from_u32(entry.pid);
    
    // Retry logic with exponential backoff
    let max_retries = 3;
    let mut retries = 0;
    
    while retries < max_retries {
        let killed = {
            let mut sys = System::new();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

            if let Some(process) = sys.process(pid) {
                let signal = if force {
                    info!("Force killing PID {} (port {})", entry.pid, entry.port);
                    Signal::Kill
                } else {
                    info!(
                        "Gracefully stopping PID {} (port {})",
                        entry.pid, entry.port
                    );
                    Signal::Term
                };

                if process.kill_with(signal).unwrap_or(false) {
                    info!("Successfully sent {:?} to PID {}", signal, entry.pid);
                    
                    // Verify process is gone
                    drop(sys);
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    let mut verify_sys = System::new();
                    verify_sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                    if verify_sys.process(pid).is_none() {
                        return Ok(());
                    }
                    
                    // Process still exists, might need force kill
                    if !force && retries < max_retries - 1 {
                        warn!("SIGTERM failed for PID {}, trying SIGKILL", entry.pid);
                        let mut kill_sys = System::new();
                        kill_sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                        if let Some(proc) = kill_sys.process(pid) {
                            if proc.kill_with(Signal::Kill).unwrap_or(false) {
                                info!("Successfully killed PID {} with SIGKILL", entry.pid);
                                return Ok(());
                            }
                        }
                    } else if force {
                        return Err(PortForgeError::ProcessError(format!(
                            "Failed to kill PID {} even with SIGKILL",
                            entry.pid
                        )));
                    }
                    true
                } else {
                    // Failed to send signal
                    warn!("Failed to send signal to PID {}", entry.pid);
                    false
                }
            } else {
                // Process no longer exists (race condition handled)
                info!("PID {} already exited", entry.pid);
                return Ok(());
            }
        };
        
        retries += 1;
        if retries < max_retries && !killed {
            std::thread::sleep(std::time::Duration::from_millis(100 * retries));
        }
    }
    
    Err(PortForgeError::ProcessError(format!(
        "Failed to kill PID {} after {} attempts",
        entry.pid, max_retries
    )))
}

/// Find and clean orphaned/zombie processes that are listening on ports.
pub fn clean_orphans(entries: &[PortEntry], dry_run: bool) -> Result<Vec<CleanResult>> {
    let mut results = Vec::new();

    for entry in entries {
        let should_clean = matches!(
            entry.status,
            crate::models::Status::Zombie | crate::models::Status::Orphaned
        );

        if should_clean {
            if dry_run {
                info!(
                    "[DRY RUN] Would kill PID {} ({}) on port {}",
                    entry.pid, entry.process_name, entry.port
                );
                results.push(CleanResult {
                    port: entry.port,
                    pid: entry.pid,
                    process_name: entry.process_name.clone(),
                    action: CleanAction::WouldKill,
                    success: true,
                });
            } else {
                let success = kill_process(entry, false).is_ok();
                results.push(CleanResult {
                    port: entry.port,
                    pid: entry.pid,
                    process_name: entry.process_name.clone(),
                    action: CleanAction::Killed,
                    success,
                });
            }
        }
    }

    Ok(results)
}

/// Get process tree for a given PID.
pub fn get_process_tree(pid: u32) -> Vec<ProcessTreeEntry> {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let target_pid = Pid::from_u32(pid);
    let mut tree = Vec::new();

    // Find the target process and its children
    if let Some(process) = sys.process(target_pid) {
        tree.push(ProcessTreeEntry {
            pid,
            name: process.name().to_string_lossy().to_string(),
            cpu_percent: process.cpu_usage(),
            memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
            depth: 0,
        });

        // Find children recursively
        collect_children(&sys, target_pid, &mut tree, 1);
    }

    tree
}

fn collect_children(sys: &System, parent_pid: Pid, tree: &mut Vec<ProcessTreeEntry>, depth: usize) {
    for (pid, process) in sys.processes() {
        if process.parent() == Some(parent_pid) {
            tree.push(ProcessTreeEntry {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_percent: process.cpu_usage(),
                memory_mb: process.memory() as f64 / 1024.0 / 1024.0,
                depth,
            });
            collect_children(sys, *pid, tree, depth + 1);
        }
    }
}

/// Result of a clean operation.
#[derive(Debug, Clone)]
pub struct CleanResult {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub action: CleanAction,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub enum CleanAction {
    Killed,
    WouldKill,
}

impl std::fmt::Display for CleanAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CleanAction::Killed => write!(f, "Killed"),
            CleanAction::WouldKill => write!(f, "Would kill"),
        }
    }
}

/// A process in the process tree.
#[derive(Debug, Clone)]
pub struct ProcessTreeEntry {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_mb: f64,
    pub depth: usize,
}

impl ProcessTreeEntry {
    /// Render with tree indentation.
    pub fn display_line(&self) -> String {
        let indent = if self.depth == 0 {
            String::new()
        } else {
            format!("{}└─ ", "  ".repeat(self.depth - 1))
        };
        format!(
            "{}{} (PID: {}, CPU: {:.1}%, Mem: {:.1}MB)",
            indent, self.name, self.pid, self.cpu_percent, self.memory_mb
        )
    }
}
