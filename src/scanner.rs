use crate::config::PortForgeConfig;
#[cfg(feature = "docker")]
use crate::docker;
use crate::error::Result;
use crate::git;
use crate::health;
use crate::health::HealthCheckType;
use crate::models::*;
use crate::project;
use crate::tunnel;
use std::collections::HashMap;
use std::sync::Arc;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tokio::sync::Semaphore;
use tracing::{debug, warn};

/// Scans all listening ports and enriches them with metadata including:
/// - Process information (PID, name, command, CPU/memory usage)
/// - Project detection (framework, language, version)
/// - Git repository status (branch, dirty state)
/// - Docker container mapping
/// - Tunnel service detection (ngrok, cloudflared, etc.)
/// - Health check results
///
/// # Arguments
/// * `config` - PortForge configuration settings
/// * `show_all` - If true, show all ports; if false, filter to dev projects only
///
/// # Returns
/// A vector of `PortEntry` structs containing enriched port data
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

    // Query Docker containers (if enabled) first so we don't hold the !Send sys mutex across await
    #[cfg(feature = "docker")]
    let docker_map = if config.general.docker_enabled {
        match docker::get_container_port_map().await {
            Ok(map) => map,
            Err(e) => {
                warn!(
                    "Docker integration unavailable: {}. Disabling Docker info for this scan.",
                    e
                );
                HashMap::new()
            }
        }
    } else {
        HashMap::new()
    };
    #[cfg(not(feature = "docker"))]
    let docker_map: HashMap<u16, crate::models::DockerInfo> = HashMap::new();

    let mut entries: Vec<PortEntry> = Vec::new();

    // Scope the !Send MutexGuard so it drops before we await health checks
    {
        static SYSTEM: std::sync::OnceLock<std::sync::Mutex<System>> = std::sync::OnceLock::new();
        let mut sys = SYSTEM
            .get_or_init(|| std::sync::Mutex::new(System::new()))
            .lock()
            .unwrap();

        let is_new = sys.processes().is_empty();

        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .with_cwd(UpdateKind::Always)
                .with_cmd(UpdateKind::Always),
        );

        if is_new {
            std::thread::sleep(std::time::Duration::from_millis(200));
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing()
                    .with_cpu()
                    .with_cwd(UpdateKind::Always),
            );
        }

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

            let cwd = proc_info.and_then(|p| p.cwd().map(|cwd| cwd.to_path_buf()));

            let memory_mb = proc_info
                .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
                .unwrap_or(0.0);

            let cpu_percent = proc_info.map(|p| p.cpu_usage()).unwrap_or(0.0);

            let uptime_secs = proc_info.map(|p| p.run_time()).unwrap_or(0);

            let project_info = cwd.as_ref().and_then(|cwd| project::detect_project(cwd));
            let git_info = cwd.as_ref().and_then(|cwd| git::get_git_info(cwd));
            let docker_info = docker_map.get(&port).cloned();
            let tunnel_info = tunnel::detect_tunnel(&process_name, &command);

            let protocol = match listener.protocol {
                listeners::Protocol::TCP => Protocol::Tcp,
                listeners::Protocol::UDP => Protocol::Udp,
            };

            let status = determine_status(proc_info.is_some(), &project_info, &docker_info);

            entries.push(PortEntry {
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
                tunnel: tunnel_info,
                status,
                health_check: None,
            });
        }
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

/// Determines the status of a port entry based on process existence and enrichment data.
///
/// Returns `Zombie` if the process doesn't exist, `Healthy` if it has project/Docker info,
/// otherwise `Unknown`.
fn determine_status(
    process_exists: bool,
    project: &Option<ProjectInfo>,
    docker: &Option<DockerInfo>,
) -> Status {
    if !process_exists {
        return Status::Zombie;
    }
    if project.is_some() || docker.is_some() {
        return Status::Healthy;
    }
    Status::Unknown
}

/// Run health checks on all entries concurrently with limited concurrency.
async fn run_health_checks(entries: &mut [PortEntry], config: &PortForgeConfig) {
    let timeout_ms = config.health.timeout_ms;
    let max_concurrent = config.general.max_concurrent_health_checks;

    // Create a semaphore to limit concurrent health checks
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();

    for entry in entries.iter() {
        let port = entry.port;
        let (check_type, endpoint) = resolve_health_strategy(entry, config);

        let timeout = timeout_ms;
        let sem_permit = semaphore.clone().acquire_owned().await.unwrap();

        handles.push(tokio::spawn(async move {
            let result = health::check_health_typed(port, check_type, &endpoint, timeout).await;
            drop(sem_permit); // Release permit when done
            (port, result)
        }));
    }

    for handle in handles.into_iter() {
        if let Ok((port, result)) = handle.await {
            if let Some(entry) = entries.iter_mut().find(|e| e.port == port) {
                entry.health_check = Some(result);
                // Update status based on health check
                if let Some(ref hc) = entry.health_check {
                    if hc.status == HealthStatus::Healthy && entry.status == Status::Unknown {
                        entry.status = Status::Healthy;
                    } else if hc.status == HealthStatus::Unhealthy
                        && entry.status == Status::Healthy
                    {
                        // Mark as warning if health check fails but process exists
                        entry.status = Status::Warning("Health check failed".to_string());
                    }
                }
            }
        }
    }
}

fn resolve_health_strategy(entry: &PortEntry, config: &PortForgeConfig) -> (HealthCheckType, String) {
    let framework = entry
        .project
        .as_ref()
        .map(|p| p.framework.to_lowercase())
        .unwrap_or_default();
    let process_name = entry.process_name.to_lowercase();
    let command = entry.command.to_lowercase();

    if let Some(port_override) = config.ports.get(&entry.port) {
        if let Some(endpoint) = &port_override.health_endpoint {
            return parse_health_endpoint(endpoint);
        }
    }

    if let Some(endpoint) = config.health.framework_endpoints.get(&framework) {
        return parse_health_endpoint(endpoint);
    }

    if command.contains("grpc") || process_name.contains("grpc") || framework.contains("grpc") {
        return (HealthCheckType::Grpc, "gRPC".to_string());
    }

    if command.contains("websocket")
        || command.contains("ws://")
        || command.contains("socket.io")
        || process_name.contains("websocket")
        || framework.contains("socket")
    {
        return (HealthCheckType::WebSocket, "WebSocket".to_string());
    }

    let endpoint = config
        .health
        .default_endpoints
        .first()
        .cloned()
        .unwrap_or_else(|| "/health".to_string());

    (HealthCheckType::Http, endpoint)
}

fn parse_health_endpoint(endpoint: &str) -> (HealthCheckType, String) {
    let trimmed = endpoint.trim();
    if let Some(value) = trimmed.strip_prefix("grpc:") {
        return (HealthCheckType::Grpc, value.trim().trim_start_matches('/').to_string().if_empty("gRPC"));
    }
    if let Some(value) = trimmed.strip_prefix("grpc://") {
        return (HealthCheckType::Grpc, value.trim().to_string().if_empty("gRPC"));
    }
    if let Some(value) = trimmed.strip_prefix("ws:") {
        return (HealthCheckType::WebSocket, value.trim().trim_start_matches('/').to_string().if_empty("WebSocket"));
    }
    if let Some(value) = trimmed.strip_prefix("websocket:") {
        return (HealthCheckType::WebSocket, value.trim().trim_start_matches('/').to_string().if_empty("WebSocket"));
    }
    if let Some(value) = trimmed.strip_prefix("ws://") {
        return (HealthCheckType::WebSocket, value.trim().to_string().if_empty("WebSocket"));
    }
    (HealthCheckType::Http, trimmed.to_string())
}

trait NonEmptyString {
    fn if_empty(self, fallback: &str) -> String;
}

impl NonEmptyString for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PortForgeConfig;

    fn entry_with_command(port: u16, process_name: &str, command: &str) -> PortEntry {
        PortEntry {
            port,
            protocol: Protocol::Tcp,
            pid: port as u32,
            process_name: process_name.to_string(),
            command: command.to_string(),
            cwd: None,
            memory_mb: 0.0,
            cpu_percent: 0.0,
            uptime_secs: 0,
            project: None,
            docker: None,
            git: None,
            tunnel: None,
            status: Status::Unknown,
            health_check: None,
        }
    }

    #[test]
    fn test_resolve_health_strategy_uses_port_override_prefix() {
        let mut config = PortForgeConfig::default();
        config.ports.insert(
            50051,
            crate::config::PortOverride {
                label: None,
                health_endpoint: Some("grpc:".to_string()),
                hidden: false,
            },
        );

        let entry = entry_with_command(50051, "server", "server --port 50051");
        let (kind, endpoint) = resolve_health_strategy(&entry, &config);

        assert_eq!(kind, HealthCheckType::Grpc);
        assert_eq!(endpoint, "gRPC");
    }

    #[test]
    fn test_resolve_health_strategy_detects_websocket_from_command() {
        let config = PortForgeConfig::default();
        let entry = entry_with_command(3001, "node", "node websocket-server.js --port 3001");
        let (kind, endpoint) = resolve_health_strategy(&entry, &config);

        assert_eq!(kind, HealthCheckType::WebSocket);
        assert_eq!(endpoint, "WebSocket");
    }

    #[test]
    fn test_parse_health_endpoint_preserves_http_paths() {
        let (kind, endpoint) = parse_health_endpoint("/readyz");
        assert_eq!(kind, HealthCheckType::Http);
        assert_eq!(endpoint, "/readyz");
    }
}
