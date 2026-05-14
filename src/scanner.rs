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
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
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
    let mut project_cache: HashMap<PathBuf, Option<ProjectInfo>> = HashMap::new();
    let mut git_cache: HashMap<PathBuf, Option<GitInfo>> = HashMap::new();

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

            if should_skip_listener_pid(pid) {
                debug!(
                    "Skipping pseudo-process listener on port {} with PID {}",
                    port, pid
                );
                continue;
            }

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

            let project_info = cwd
                .as_ref()
                .and_then(|cwd| cached_project(cwd, config, &mut project_cache));
            let git_info = cwd.as_ref().and_then(|cwd| cached_git(cwd, &mut git_cache));
            let docker_info = docker_map.get(&port).cloned();
            let tunnel_info = tunnel::detect_tunnel(&process_name, &command);

            let protocol = match listener.protocol {
                listeners::Protocol::TCP => Protocol::Tcp,
                listeners::Protocol::UDP => Protocol::Udp,
            };

            let status = determine_status(proc_info.is_some(), &project_info, &docker_info);
            let label = config
                .ports
                .get(&port)
                .and_then(|port_override| port_override.label.clone());

            entries.push(PortEntry {
                port,
                protocol,
                pid,
                label,
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

    dedupe_listener_entries(&mut entries);

    // Filter to dev-only unless --all
    apply_view_filter(&mut entries, config, show_all);

    // Run health checks if enabled
    if config.general.health_checks_enabled {
        run_health_checks(&mut entries, config).await;
    }

    // Sort by port by default
    entries.sort_by_key(|e| e.port);

    debug!("Scan complete: {} entries", entries.len());
    Ok(entries)
}

fn dedupe_listener_entries(entries: &mut Vec<PortEntry>) {
    let mut seen_listeners = HashSet::new();
    entries.retain(|e| seen_listeners.insert((e.port, e.protocol, e.pid)));
}

fn apply_view_filter(entries: &mut Vec<PortEntry>, config: &PortForgeConfig, show_all: bool) {
    if show_all {
        return;
    }

    entries.retain(|e| {
        !config
            .ports
            .get(&e.port)
            .map(|port_override| port_override.hidden)
            .unwrap_or(false)
            && (e.project.is_some() || e.docker.is_some() || e.label.is_some())
    });
}

fn cached_project(
    cwd: &Path,
    config: &PortForgeConfig,
    cache: &mut HashMap<PathBuf, Option<ProjectInfo>>,
) -> Option<ProjectInfo> {
    let key = cwd.to_path_buf();
    cache
        .entry(key)
        .or_insert_with(|| project::detect_project_with_custom(cwd, &config.detectors))
        .clone()
}

fn cached_git(cwd: &Path, cache: &mut HashMap<PathBuf, Option<GitInfo>>) -> Option<GitInfo> {
    let key = cwd.to_path_buf();
    cache
        .entry(key)
        .or_insert_with(|| git::get_git_info(cwd))
        .clone()
}

fn should_skip_listener_pid(pid: u32) -> bool {
    pid == 0
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
    let max_concurrent = config.general.max_concurrent_health_checks.max(1);
    let http_client = match health::build_client(timeout_ms) {
        Ok(client) => Some(Arc::new(client)),
        Err(e) => {
            warn!("Failed to build HTTP health client: {}", e);
            None
        }
    };

    // Create a semaphore to limit concurrent health checks
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();

    for entry in entries.iter() {
        let port = entry.port;
        let strategies = resolve_health_strategies(entry, config);

        let timeout = timeout_ms;
        let client = http_client.clone();
        let sem_permit = semaphore.clone().acquire_owned().await.unwrap();

        handles.push(tokio::spawn(async move {
            let result = run_health_strategies(port, strategies, timeout, client).await;
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

#[cfg(test)]
fn resolve_health_strategy(
    entry: &PortEntry,
    config: &PortForgeConfig,
) -> (HealthCheckType, String) {
    resolve_health_strategies(entry, config)
        .into_iter()
        .next()
        .unwrap_or((HealthCheckType::Http, "/health".to_string()))
}

fn resolve_health_strategies(
    entry: &PortEntry,
    config: &PortForgeConfig,
) -> Vec<(HealthCheckType, String)> {
    let framework = entry
        .project
        .as_ref()
        .map(|p| p.framework.to_lowercase())
        .unwrap_or_default();
    let process_name = entry.process_name.to_lowercase();
    let command = entry.command.to_lowercase();

    if let Some(port_override) = config.ports.get(&entry.port) {
        if let Some(endpoint) = &port_override.health_endpoint {
            return vec![parse_health_endpoint(endpoint)];
        }
    }

    if let Some(project) = &entry.project {
        if let Some(detector) = config.detectors.iter().find(|detector| {
            detector.kind.eq_ignore_ascii_case(&project.kind)
                && detector.framework.eq_ignore_ascii_case(&project.framework)
        }) {
            if let Some(endpoint) = &detector.health_endpoint {
                return vec![parse_health_endpoint(endpoint)];
            }
        }
    }

    if let Some(endpoint) = config.health.framework_endpoints.get(&framework) {
        return vec![parse_health_endpoint(endpoint)];
    }

    if command.contains("grpc") || process_name.contains("grpc") || framework.contains("grpc") {
        return vec![(HealthCheckType::Grpc, "gRPC".to_string())];
    }

    if command.contains("websocket")
        || command.contains("ws://")
        || command.contains("socket.io")
        || process_name.contains("websocket")
        || framework.contains("socket")
    {
        return vec![(HealthCheckType::WebSocket, "WebSocket".to_string())];
    }

    let endpoints = if config.health.default_endpoints.is_empty() {
        vec!["/health".to_string()]
    } else {
        config.health.default_endpoints.clone()
    };

    endpoints
        .into_iter()
        .map(|endpoint| (HealthCheckType::Http, endpoint))
        .collect()
}

async fn run_health_strategies(
    port: u16,
    strategies: Vec<(HealthCheckType, String)>,
    timeout_ms: u64,
    http_client: Option<Arc<reqwest::Client>>,
) -> HealthResult {
    let mut last_result = None;

    for (check_type, endpoint) in strategies {
        let result = match (check_type, http_client.as_ref()) {
            (HealthCheckType::Http, Some(client)) => {
                health::check_health_with_client(client, port, &endpoint).await
            }
            _ => health::check_health_typed(port, check_type, &endpoint, timeout_ms).await,
        };

        if result.status == HealthStatus::Healthy {
            return result;
        }
        last_result = Some(result);
    }

    last_result.unwrap_or(HealthResult {
        status: HealthStatus::Unknown,
        status_code: None,
        latency_ms: 0,
        endpoint: "/health".to_string(),
    })
}

fn parse_health_endpoint(endpoint: &str) -> (HealthCheckType, String) {
    let trimmed = endpoint.trim();
    if let Some(value) = trimmed.strip_prefix("grpc:") {
        return (
            HealthCheckType::Grpc,
            value
                .trim()
                .trim_start_matches('/')
                .to_string()
                .if_empty("gRPC"),
        );
    }
    if let Some(value) = trimmed.strip_prefix("grpc://") {
        return (
            HealthCheckType::Grpc,
            value.trim().to_string().if_empty("gRPC"),
        );
    }
    if let Some(value) = trimmed.strip_prefix("ws:") {
        return (
            HealthCheckType::WebSocket,
            value
                .trim()
                .trim_start_matches('/')
                .to_string()
                .if_empty("WebSocket"),
        );
    }
    if let Some(value) = trimmed.strip_prefix("websocket:") {
        return (
            HealthCheckType::WebSocket,
            value
                .trim()
                .trim_start_matches('/')
                .to_string()
                .if_empty("WebSocket"),
        );
    }
    if let Some(value) = trimmed.strip_prefix("ws://") {
        return (
            HealthCheckType::WebSocket,
            value.trim().to_string().if_empty("WebSocket"),
        );
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
            label: None,
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
    fn test_resolve_health_strategies_uses_all_default_endpoints() {
        let mut config = PortForgeConfig::default();
        config.health.default_endpoints = vec!["/ready".to_string(), "/".to_string()];

        let entry = entry_with_command(3000, "server", "server --port 3000");
        let strategies = resolve_health_strategies(&entry, &config);

        assert_eq!(
            strategies,
            vec![
                (HealthCheckType::Http, "/ready".to_string()),
                (HealthCheckType::Http, "/".to_string())
            ]
        );
    }

    #[test]
    fn test_dedupe_keeps_distinct_protocols_and_pids() {
        let mut entries = vec![
            entry_with_command(3000, "tcp-one", ""),
            entry_with_command(3000, "tcp-one-duplicate", ""),
            entry_with_command(3000, "tcp-two", ""),
            entry_with_command(3000, "udp-one", ""),
        ];
        entries[2].pid = 4242;
        entries[3].protocol = Protocol::Udp;

        dedupe_listener_entries(&mut entries);

        assert_eq!(entries.len(), 3);
        assert!(
            entries
                .iter()
                .any(|e| e.protocol == Protocol::Tcp && e.pid == 3000)
        );
        assert!(
            entries
                .iter()
                .any(|e| e.protocol == Protocol::Tcp && e.pid == 4242)
        );
        assert!(
            entries
                .iter()
                .any(|e| e.protocol == Protocol::Udp && e.pid == 3000)
        );
    }

    #[test]
    fn test_view_filter_keeps_labeled_ports_and_hides_hidden_ports() {
        let mut config = PortForgeConfig::default();
        config.ports.insert(
            3000,
            crate::config::PortOverride {
                label: Some("Frontend".to_string()),
                health_endpoint: None,
                hidden: false,
            },
        );
        config.ports.insert(
            4000,
            crate::config::PortOverride {
                label: Some("Hidden".to_string()),
                health_endpoint: None,
                hidden: true,
            },
        );

        let mut labeled = entry_with_command(3000, "node", "");
        labeled.label = Some("Frontend".to_string());
        let mut hidden = entry_with_command(4000, "node", "");
        hidden.label = Some("Hidden".to_string());
        let unknown = entry_with_command(5000, "system", "");
        let mut entries = vec![labeled, hidden, unknown];

        apply_view_filter(&mut entries, &config, false);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].port, 3000);
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

    #[test]
    fn test_should_skip_listener_pid_filters_pid_zero() {
        assert!(should_skip_listener_pid(0));
        assert!(!should_skip_listener_pid(1));
        assert!(!should_skip_listener_pid(4242));
    }
}
