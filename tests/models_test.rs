use portforge::models::*;

#[test]
fn test_port_entry_uptime_display() {
    let entry = create_test_entry(3000, "node", 30);
    assert_eq!(entry.uptime_display(), "30s");

    let entry = create_test_entry(3000, "node", 90);
    assert_eq!(entry.uptime_display(), "1m 30s");

    let entry = create_test_entry(3000, "node", 3661);
    assert_eq!(entry.uptime_display(), "1h 1m");

    let entry = create_test_entry(3000, "node", 90000);
    assert_eq!(entry.uptime_display(), "1d 1h");
}

#[test]
fn test_port_entry_project_display() {
    let mut entry = create_test_entry(3000, "node", 100);
    assert_eq!(entry.project_display(), "—");

    entry.project = Some(ProjectInfo {
        kind: "Node.js".to_string(),
        framework: "Next.js".to_string(),
        version: Some("14.0.0".to_string()),
        detected_file: std::path::PathBuf::from("package.json"),
    });
    assert_eq!(entry.project_display(), "Next.js (Node.js)");

    entry.project = Some(ProjectInfo {
        kind: "Rust".to_string(),
        framework: String::new(),
        version: None,
        detected_file: std::path::PathBuf::from("Cargo.toml"),
    });
    assert_eq!(entry.project_display(), "Rust");
}

#[test]
fn test_port_entry_git_display() {
    let mut entry = create_test_entry(3000, "node", 100);
    assert_eq!(entry.git_display(), "—");

    entry.git = Some(GitInfo {
        branch: "main".to_string(),
        dirty: false,
    });
    assert_eq!(entry.git_display(), "main");

    entry.git = Some(GitInfo {
        branch: "feature/test".to_string(),
        dirty: true,
    });
    assert_eq!(entry.git_display(), "feature/test*");
}

#[test]
fn test_port_entry_docker_display() {
    let mut entry = create_test_entry(3000, "node", 100);
    assert_eq!(entry.docker_display(), "—");

    entry.docker = Some(DockerInfo {
        container_name: "my-app".to_string(),
        image: "node:18".to_string(),
        compose_project: Some("myproject".to_string()),
        container_id: "abc123".to_string(),
    });
    assert_eq!(entry.docker_display(), "my-app");
}

#[test]
fn test_status_display() {
    assert_eq!(Status::Healthy.to_string(), "● Healthy");
    assert_eq!(Status::Zombie.to_string(), "✗ Zombie");
    assert_eq!(Status::Orphaned.to_string(), "◌ Orphaned");
    assert_eq!(
        Status::Warning("High CPU".to_string()).to_string(),
        "⚠ High CPU"
    );
    assert_eq!(Status::Unknown.to_string(), "? Unknown");
}

#[test]
fn test_status_priority() {
    assert!(Status::Zombie.priority() < Status::Healthy.priority());
    assert!(Status::Orphaned.priority() < Status::Healthy.priority());
    assert!(Status::Warning("test".into()).priority() < Status::Healthy.priority());
}

#[test]
fn test_protocol_display() {
    assert_eq!(Protocol::Tcp.to_string(), "TCP");
    assert_eq!(Protocol::Udp.to_string(), "UDP");
}

#[test]
fn test_sort_direction_toggle() {
    assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
    assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
}

#[test]
fn test_sort_direction_indicator() {
    assert_eq!(SortDirection::Ascending.indicator(), "▲");
    assert_eq!(SortDirection::Descending.indicator(), "▼");
}

#[test]
fn test_model_serialization() {
    let entry = create_test_entry(8080, "rust-app", 3600);
    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("8080"));
    assert!(json.contains("rust-app"));

    // Deserialize back
    let parsed: PortEntry = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.port, 8080);
    assert_eq!(parsed.process_name, "rust-app");
}

#[test]
fn test_health_status_display() {
    assert_eq!(HealthStatus::Healthy.to_string(), "✓ Healthy");
    assert_eq!(HealthStatus::Unhealthy.to_string(), "✗ Unhealthy");
    assert_eq!(HealthStatus::Unknown.to_string(), "? Unknown");
}

// ─── Helpers ───

fn create_test_entry(port: u16, name: &str, uptime: u64) -> PortEntry {
    PortEntry {
        port,
        protocol: Protocol::Tcp,
        pid: 1234,
        process_name: name.to_string(),
        command: format!("/usr/bin/{}", name),
        cwd: Some(std::path::PathBuf::from("/tmp")),
        memory_mb: 50.0,
        cpu_percent: 2.5,
        uptime_secs: uptime,
        project: None,
        docker: None,
        git: None,
        tunnel: None,
        status: Status::Unknown,
        health_check: None,
    }
}
