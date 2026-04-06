use crate::error::Result;
use crate::models::PortEntry;
use serde_json;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct PortRow {
    #[tabled(rename = "PORT")]
    port: String,
    #[tabled(rename = "PID")]
    pid: u32,
    #[tabled(rename = "PROCESS")]
    process: String,
    #[tabled(rename = "PROJECT")]
    project: String,
    #[tabled(rename = "GIT")]
    git: String,
    #[tabled(rename = "TUNNEL")]
    tunnel: String,
    #[tabled(rename = "DOCKER")]
    docker: String,
    #[tabled(rename = "UPTIME")]
    uptime: String,
    #[tabled(rename = "MEM")]
    memory: String,
    #[tabled(rename = "CPU")]
    cpu: String,
    #[tabled(rename = "STATUS")]
    status: String,
}

impl From<&PortEntry> for PortRow {
    fn from(e: &PortEntry) -> Self {
        Self {
            port: format!("{}/{}", e.port, e.protocol),
            pid: e.pid,
            process: e.process_name.clone(),
            project: e.project_display(),
            git: e.git_display(),
            tunnel: e.tunnel_display(),
            docker: e.docker_display(),
            uptime: e.uptime_display(),
            memory: format!("{:.1}MB", e.memory_mb),
            cpu: format!("{:.1}%", e.cpu_percent),
            status: e.status.to_string(),
        }
    }
}

/// Export entries as a pretty table string.
pub fn to_table(entries: &[PortEntry]) -> String {
    if entries.is_empty() {
        return "No ports found.".to_string();
    }

    let rows: Vec<PortRow> = entries.iter().map(PortRow::from).collect();
    Table::new(rows).to_string()
}

/// Export entries as JSON string.
pub fn to_json(entries: &[PortEntry], pretty: bool) -> Result<String> {
    let result = if pretty {
        serde_json::to_string_pretty(entries)?
    } else {
        serde_json::to_string(entries)?
    };
    Ok(result)
}

/// Export entries as CSV string.
pub fn to_csv(entries: &[PortEntry]) -> String {
    let mut output = String::new();
    output.push_str("port,protocol,pid,process,project,framework,git_branch,git_dirty,tunnel,docker,uptime_secs,memory_mb,cpu_percent,status\n");

    for e in entries {
        let project = e.project.as_ref().map(|p| p.kind.as_str()).unwrap_or("");
        let framework = e
            .project
            .as_ref()
            .map(|p| p.framework.as_str())
            .unwrap_or("");
        let git_branch = e.git.as_ref().map(|g| g.branch.as_str()).unwrap_or("");
        let git_dirty = e.git.as_ref().map(|g| g.dirty).unwrap_or(false);
        let tunnel = e.tunnel.as_ref().map(|t| t.kind.as_str()).unwrap_or("");
        let docker = e
            .docker
            .as_ref()
            .map(|d| d.container_name.as_str())
            .unwrap_or("");

        let status_str = match &e.status {
            crate::models::Status::Healthy => "healthy",
            crate::models::Status::Orphaned => "orphaned",
            crate::models::Status::Zombie => "zombie",
            crate::models::Status::Warning(_) => "warning",
            crate::models::Status::Unknown => "unknown",
        };

        output.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{:.1},{:.1},{}\n",
            e.port,
            e.protocol,
            e.pid,
            escape_csv(&e.process_name),
            escape_csv(project),
            escape_csv(framework),
            escape_csv(git_branch),
            git_dirty,
            escape_csv(tunnel),
            escape_csv(docker),
            e.uptime_secs,
            e.memory_mb,
            e.cpu_percent,
            status_str
        ));
    }

    output
}

/// Escape a CSV field (quote if it contains commas or quotes).
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Print detailed inspection for a single port entry.
pub fn print_inspection(entry: &PortEntry) {
    println!("╔══════════════════════════════════════════════════╗");
    println!("║  ⚡ PortForge — Port Inspection                  ║");
    println!("╚══════════════════════════════════════════════════╝");
    println!();
    println!("  Port:        {}/{}", entry.port, entry.protocol);
    println!("  PID:         {}", entry.pid);
    println!("  Process:     {}", entry.process_name);
    println!("  Command:     {}", entry.command);
    println!(
        "  CWD:         {}",
        entry
            .cwd
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "—".to_string())
    );
    println!("  Memory:      {:.1} MB", entry.memory_mb);
    println!("  CPU:         {:.1}%", entry.cpu_percent);
    println!("  Uptime:      {}", entry.uptime_display());
    println!("  Status:      {}", entry.status);
    println!();

    if let Some(ref project) = entry.project {
        println!("  📦 Project");
        println!("    Kind:      {}", project.kind);
        if !project.framework.is_empty() {
            println!("    Framework: {}", project.framework);
        }
        if let Some(ref version) = project.version {
            println!("    Version:   {}", version);
        }
        println!("    Manifest:  {}", project.detected_file.display());
        println!();
    }

    if let Some(ref git) = entry.git {
        println!("  🔀 Git");
        println!("    Branch:    {}", git.branch);
        println!(
            "    Status:    {}",
            if git.dirty { "Modified" } else { "Clean" }
        );
        println!();
    }

    if let Some(ref docker) = entry.docker {
        println!("  🐳 Docker");
        println!("    Container: {}", docker.container_name);
        println!("    Image:     {}", docker.image);
        if let Some(ref compose) = docker.compose_project {
            println!("    Compose:   {}", compose);
        }
        println!();
    }

    if let Some(ref health) = entry.health_check {
        println!("  🏥 Health Check");
        println!("    Status:    {}", health.status);
        if let Some(code) = health.status_code {
            println!("    HTTP Code: {}", code);
        }
        println!("    Latency:   {}ms", health.latency_ms);
        println!("    Endpoint:  {}", health.endpoint);
        println!();
    }
}
