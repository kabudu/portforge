use portforge::models::{PortEntry, Protocol, Status};
use portforge::process::{CleanResult, kill_process};

fn create_test_entry(port: u16, pid: u32) -> PortEntry {
    PortEntry {
        port,
        protocol: Protocol::Tcp,
        pid,
        label: None,
        process_name: "test_process".to_string(),
        command: "test --args".to_string(),
        cwd: None,
        memory_mb: 50.0,
        cpu_percent: 1.0,
        uptime_secs: 100,
        project: None,
        docker: None,
        git: None,
        tunnel: None,
        kubernetes: None,
        status: Status::Healthy,
        health_check: None,
    }
}

#[test]
fn test_kill_process_graceful() {
    let entry = create_test_entry(9999, u32::MAX);

    let result = kill_process(&entry, false);
    assert!(result.is_ok());
}

#[test]
fn test_kill_process_force() {
    let entry = create_test_entry(9998, u32::MAX);
    let result = kill_process(&entry, true);
    assert!(result.is_ok());
}

#[test]
fn test_clean_result_display() {
    let result = CleanResult {
        port: 3000,
        pid: 1234,
        process_name: "node".to_string(),
        action: portforge::process::CleanAction::Killed,
        success: true,
    };

    assert_eq!(format!("{}", result.action), "Killed");

    let would_kill = portforge::process::CleanAction::WouldKill;
    assert_eq!(format!("{would_kill}"), "Would kill");
}
