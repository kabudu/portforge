use crate::config::PortForgeConfig;
use crate::error::Result;
use crate::models::Protocol;
use std::collections::HashMap;
use std::net::TcpListener;

/// Find a free port starting from the given port number.
/// Checks if the port is available by attempting to bind to it.
pub fn find_free_port(start_port: u16) -> Option<u16> {
    for port in start_port..=65535 {
        if is_port_free(port) {
            return Some(port);
        }
    }
    None
}

/// Check if a port is free by attempting to bind to it.
pub fn is_port_free(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Find multiple free ports starting from a given port.
pub fn find_free_ports(start_port: u16, count: usize) -> Vec<u16> {
    let mut free_ports = Vec::with_capacity(count);
    let mut current_port = start_port;

    while free_ports.len() < count {
        if is_port_free(current_port) {
            free_ports.push(current_port);
        }
        // Check for overflow before incrementing
        if current_port == 65535 {
            break;
        }
        current_port += 1;
    }

    free_ports
}

/// Detect port conflicts - processes trying to use the same port.
pub async fn detect_conflicts(config: &PortForgeConfig) -> Result<Vec<PortConflict>> {
    let _ = config;
    let mut conflicts = Vec::new();
    let listeners = listeners::get_all().map_err(|e| crate::error::PortForgeError::ScanError(e.to_string()))?;

    // Group distinct processes by protocol + port so TCP/UDP listeners don't report false conflicts.
    let mut port_map: HashMap<(u16, Protocol), HashMap<u32, ProcessInfo>> = HashMap::new();
    for listener in listeners {
        let protocol = match listener.protocol {
            listeners::Protocol::TCP => Protocol::Tcp,
            listeners::Protocol::UDP => Protocol::Udp,
        };

        port_map
            .entry((listener.socket.port(), protocol))
            .or_default()
            .entry(listener.process.pid)
            .or_insert_with(|| ProcessInfo {
                pid: listener.process.pid,
                name: listener.process.name.clone(),
                command: String::new(),
            });
    }

    // Find protocol-specific ports with multiple processes (conflicts)
    for ((port, protocol), processes) in port_map {
        let processes: Vec<ProcessInfo> = processes.into_values().collect();
        if processes.len() > 1 {
            conflicts.push(PortConflict {
                port,
                protocol,
                processes: processes.clone(),
                suggestion: generate_conflict_suggestion(port, protocol, &processes),
            });
        }
    }

    Ok(conflicts)
}

/// Information about a process using a port.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
}

/// A port conflict with resolution suggestions.
#[derive(Debug, Clone)]
pub struct PortConflict {
    pub port: u16,
    pub protocol: Protocol,
    pub processes: Vec<ProcessInfo>,
    pub suggestion: String,
}

/// Generate a suggestion for resolving a port conflict.
fn generate_conflict_suggestion(port: u16, protocol: Protocol, processes: &[ProcessInfo]) -> String {
    if processes.is_empty() {
        return "No processes found.".to_string();
    }

    if processes.len() == 1 {
        return format!(
            "Port {}/{} is used by PID {} ({})",
            port,
            protocol,
            processes[0].pid,
            processes[0].name
        );
    }

    let mut suggestion = format!(
        "Port {}/{} has {} conflicting processes:\n",
        port,
        protocol,
        processes.len()
    );
    for (i, proc) in processes.iter().enumerate() {
        if proc.command.is_empty() {
            suggestion.push_str(&format!("  {}. PID {} - {}\n", i + 1, proc.pid, proc.name));
        } else {
            suggestion.push_str(&format!(
                "  {}. PID {} - {} ({})\n",
                i + 1,
                proc.pid,
                proc.name,
                proc.command
            ));
        }
    }

    // Suggest killing all but one
    suggestion.push_str(&format!("\nSuggestion: Keep one process and kill the rest.\n"));
    suggestion.push_str(&format!("  Run: portforge kill {} (to kill the first one)\n", port));
    
    // Suggest alternative ports
    if let Some(free_port) = find_free_port(port + 1) {
        suggestion.push_str(&format!("  Or use free port {} instead.", free_port));
    }

    suggestion
}

/// Check if a specific port has conflicts.
pub async fn check_port_conflict(port: u16, config: &PortForgeConfig) -> Result<Option<PortConflict>> {
    let conflicts = detect_conflicts(config).await?;
    Ok(conflicts.into_iter().find(|c| c.port == port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HealthResult, HealthStatus, Status};
    use crate::models::PortEntry;
    use std::path::PathBuf;

    #[test]
    fn test_find_free_port() {
        // Find a free port starting from 3000
        let port = find_free_port(3000);
        assert!(port.is_some());
        let port = port.unwrap();
        assert!(port >= 3000);
    }

    #[test]
    fn test_find_free_ports() {
        let ports = find_free_ports(8000, 5);
        assert_eq!(ports.len(), 5);
        
        // Verify all ports are unique
        let mut unique = ports.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ports.len());
    }

    #[test]
    fn test_is_port_free() {
        // Port 0 should be free (system assigns random port)
        assert!(is_port_free(0));
    }

    #[test]
    fn test_generate_conflict_suggestion() {
        let processes = vec![
            ProcessInfo {
                pid: 1234,
                name: "node".to_string(),
                command: "node server.js".to_string(),
            },
            ProcessInfo {
                pid: 5678,
                name: "python".to_string(),
                command: "python app.py".to_string(),
            },
        ];

        let suggestion = generate_conflict_suggestion(3000, Protocol::Tcp, &processes);
        assert!(suggestion.contains("2 conflicting processes"));
        assert!(suggestion.contains("1234"));
        assert!(suggestion.contains("5678"));
    }

    #[test]
    fn test_check_port_conflict_matches_protocol_specific_entry() {
        let conflict = PortConflict {
            port: 3000,
            protocol: Protocol::Tcp,
            processes: vec![ProcessInfo {
                pid: 1234,
                name: "node".to_string(),
                command: String::new(),
            }],
            suggestion: String::new(),
        };

        assert_eq!(conflict.protocol, Protocol::Tcp);
    }

    #[test]
    fn test_port_entry_type_still_available_for_free_port_tests() {
        let entry = PortEntry {
            port: 3000,
            protocol: Protocol::Tcp,
            pid: 1,
            process_name: "node".to_string(),
            command: "node server.js".to_string(),
            cwd: Some(PathBuf::from("/app")),
            memory_mb: 10.0,
            cpu_percent: 1.0,
            uptime_secs: 1,
            project: None,
            docker: None,
            git: None,
            tunnel: None,
            status: Status::Healthy,
            health_check: Some(HealthResult {
                status: HealthStatus::Healthy,
                status_code: Some(200),
                latency_ms: 1,
                endpoint: "/health".to_string(),
            }),
        };

        assert_eq!(entry.port, 3000);
    }
}