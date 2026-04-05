use crate::config::PortForgeConfig;
#[cfg(feature = "docker")]
use crate::docker;
use crate::error::Result;
use crate::git;
use crate::health;
use crate::models::*;
use crate::project;
use std::collections::HashMap;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tracing::{debug, warn};

/// Scan all listening ports and enrich with metadata.
pub async fn scan_ports(config: &PortForgeConfig, show_all: bool) -> Result<Vec<PortEntry>> {
    debug!("Starting port scan...");

    // Get all listening sockets with PIDs
    let raw_listeners = match listeners::get_all() {
        Ok(l) => l,
        Err(e) => {
            warn!("Failed to get listeners: {}", e);
            return Ok(Vec::new());
        }
    };
    debug!("Found {} raw listeners", raw_listeners.len());

    // Refresh system info for process details
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing()
            .with_cpu()
            .with_memory()
            .with_cmd(UpdateKind::Always),
    );

    // Query Docker containers (if enabled)
    #[cfg(feature = "docker")]
    let docker_map = if config.general.docker_enabled {
        docker::get_container_port_map().await.unwrap_or_default()
    } else {
        HashMap::new()
    };
    #[cfg(not(feature = "docker"))]
    let docker_map: HashMap<u16, crate::models::DockerInfo> = HashMap::new();

    let mut entries: Vec<PortEntry> = Vec::new();

    for listener in &raw_listeners {
        let port = listener.socket.port();
        let pid = listener.process.pid;

        let sysinfo_pid = Pid::from_u32(pid);
        let proc_info = sys.process(sysinfo_pid);

        let process_name = proc_info
            .map(|p| p.name().to_string_lossy().to_string())
            .unwrap_or_else(|| listener.process.name.clone());

        let command = proc_info
            .map(|p| {
                p.cmd()
                    .iter()
                    .map(|s| s.to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        let cwd = proc_info.and_then(|p| {
            let cwd = p.cwd()?;
            Some(cwd.to_path_buf())
        });

        let memory_mb = proc_info
            .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
            .unwrap_or(0.0);

        let cpu_percent = proc_info.map(|p| p.cpu_usage()).unwrap_or(0.0);

        let uptime_secs = proc_info.map(|p| p.run_time()).unwrap_or(0);

        // Project detection
        let project_info = cwd.as_ref().and_then(|cwd| project::detect_project(cwd));

        // Git info
        let git_info = cwd.as_ref().and_then(|cwd| git::get_git_info(cwd));

        // Docker info
        let docker_info = docker_map.get(&port).cloned();

        // Map Protocol
        let protocol = match listener.protocol {
            listeners::Protocol::TCP => Protocol::Tcp,
            listeners::Protocol::UDP => Protocol::Udp,
        };

        // Determine status
        let status = determine_status(proc_info.is_some(), &project_info, &docker_info);

        let entry = PortEntry {
            port,
            protocol,
            pid,
            process_name,
            command,
            cwd,
            memory_mb,
            cpu_percent,
            uptime_secs,
            project: project_info,
            docker: docker_info,
            git: git_info,
            status,
            health_check: None,
        };

        entries.push(entry);
    }

    // Remove duplicates by port (keep the first occurrence)
    let mut seen_ports = std::collections::HashSet::new();
    entries.retain(|e| seen_ports.insert(e.port));

    // Filter to dev-only unless --all
    if !show_all {
        entries.retain(|e| e.project.is_some() || e.docker.is_some());
    }

    // Run health checks if enabled
    if config.general.health_checks_enabled {
        run_health_checks(&mut entries, config).await;
    }

    // Sort by port by default
    entries.sort_by_key(|e| e.port);

    debug!("Scan complete: {} entries", entries.len());
    Ok(entries)
}

/// Determine the status of a port entry.
fn determine_status(
    process_exists: bool,
    project: &Option<ProjectInfo>,
    _docker: &Option<DockerInfo>,
) -> Status {
    if !process_exists {
        return Status::Zombie;
    }
    if project.is_some() {
        return Status::Healthy;
    }
    Status::Unknown
}

/// Run health checks on all entries concurrently.
async fn run_health_checks(entries: &mut [PortEntry], config: &PortForgeConfig) {
    let timeout_ms = config.health.timeout_ms;
    let mut handles = Vec::new();

    for entry in entries.iter() {
        let port = entry.port;
        let framework = entry
            .project
            .as_ref()
            .map(|p| p.framework.to_lowercase())
            .unwrap_or_default();

        // Determine which endpoint to try
        let endpoint = config
            .health
            .framework_endpoints
            .get(&framework)
            .cloned()
            .unwrap_or_else(|| "/health".to_string());

        let timeout = timeout_ms;
        handles.push(tokio::spawn(async move {
            health::check_health(port, &endpoint, timeout).await
        }));
    }

    for (i, handle) in handles.into_iter().enumerate() {
        if let Ok(result) = handle.await {
            entries[i].health_check = Some(result);
            // Update status based on health check
            if let Some(ref hc) = entries[i].health_check {
                if hc.status == HealthStatus::Healthy && entries[i].status == Status::Unknown {
                    entries[i].status = Status::Healthy;
                }
            }
        }
    }
}

/// Sort entries by the given field and direction.
pub fn sort_entries(entries: &mut [PortEntry], field: SortField, direction: SortDirection) {
    entries.sort_by(|a, b| {
        let ordering = match field {
            SortField::Port => a.port.cmp(&b.port),
            SortField::Pid => a.pid.cmp(&b.pid),
            SortField::Process => a
                .process_name
                .to_lowercase()
                .cmp(&b.process_name.to_lowercase()),
            SortField::Project => a.project_display().cmp(&b.project_display()),
            SortField::Memory => a
                .memory_mb
                .partial_cmp(&b.memory_mb)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortField::Cpu => a
                .cpu_percent
                .partial_cmp(&b.cpu_percent)
                .unwrap_or(std::cmp::Ordering::Equal),
            SortField::Uptime => a.uptime_secs.cmp(&b.uptime_secs),
            SortField::Status => a.status.priority().cmp(&b.status.priority()),
        };
        match direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        }
    });
}
