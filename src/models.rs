use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Represents a single port entry with all enriched metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortEntry {
    pub port: u16,
    pub protocol: Protocol,
    pub pid: u32,
    pub process_name: String,
    pub command: String,
    pub cwd: Option<PathBuf>,
    pub memory_mb: f64,
    pub cpu_percent: f32,
    pub uptime_secs: u64,
    pub project: Option<ProjectInfo>,
    pub docker: Option<DockerInfo>,
    pub git: Option<GitInfo>,
    pub status: Status,
    pub health_check: Option<HealthResult>,
}

impl PortEntry {
    /// Human-readable uptime string.
    pub fn uptime_display(&self) -> String {
        let secs = self.uptime_secs;
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else if secs < 86400 {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        } else {
            format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
        }
    }

    /// Returns a display string for the project/framework column.
    pub fn project_display(&self) -> String {
        match &self.project {
            Some(p) => {
                if p.framework.is_empty() {
                    p.kind.clone()
                } else {
                    format!("{} ({})", p.framework, p.kind)
                }
            }
            None => String::from("—"),
        }
    }

    /// Returns a display string for the git column.
    pub fn git_display(&self) -> String {
        match &self.git {
            Some(g) => {
                let dirty = if g.dirty { "*" } else { "" };
                format!("{}{}", g.branch, dirty)
            }
            None => String::from("—"),
        }
    }

    /// Returns a display string for the docker column.
    pub fn docker_display(&self) -> String {
        match &self.docker {
            Some(d) => d.container_name.clone(),
            None => String::from("—"),
        }
    }
}

/// Network protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
        }
    }
}

/// Detected project/framework information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub kind: String,
    pub framework: String,
    pub version: Option<String>,
    pub detected_file: PathBuf,
}

/// Docker/Podman container metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerInfo {
    pub container_name: String,
    pub image: String,
    pub compose_project: Option<String>,
    pub container_id: String,
}

/// Git repository information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub branch: String,
    pub dirty: bool,
}

/// Health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResult {
    pub status: HealthStatus,
    pub status_code: Option<u16>,
    pub latency_ms: u64,
    pub endpoint: String,
}

/// Health check status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "✓ Healthy"),
            HealthStatus::Unhealthy => write!(f, "✗ Unhealthy"),
            HealthStatus::Unknown => write!(f, "? Unknown"),
        }
    }
}

/// Overall port entry status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Healthy,
    Orphaned,
    Zombie,
    Warning(String),
    Unknown,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Healthy => write!(f, "● Healthy"),
            Status::Orphaned => write!(f, "◌ Orphaned"),
            Status::Zombie => write!(f, "✗ Zombie"),
            Status::Warning(msg) => write!(f, "⚠ {}", msg),
            Status::Unknown => write!(f, "? Unknown"),
        }
    }
}

impl Status {
    /// Returns a sort priority (lower = more urgent).
    pub fn priority(&self) -> u8 {
        match self {
            Status::Zombie => 0,
            Status::Orphaned => 1,
            Status::Warning(_) => 2,
            Status::Unknown => 3,
            Status::Healthy => 4,
        }
    }
}

/// Sort field for port entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    Port,
    Pid,
    Process,
    Project,
    Memory,
    Cpu,
    Uptime,
    Status,
}

impl SortField {
    pub fn label(&self) -> &'static str {
        match self {
            SortField::Port => "Port",
            SortField::Pid => "PID",
            SortField::Process => "Process",
            SortField::Project => "Project",
            SortField::Memory => "Memory",
            SortField::Cpu => "CPU",
            SortField::Uptime => "Uptime",
            SortField::Status => "Status",
        }
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    pub fn indicator(&self) -> &'static str {
        match self {
            SortDirection::Ascending => "▲",
            SortDirection::Descending => "▼",
        }
    }
}
