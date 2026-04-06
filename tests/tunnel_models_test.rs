use portforge::models::{PortEntry, Protocol, Status, TunnelInfo};

#[test]
fn test_tunnel_display_with_url() {
    let entry = PortEntry {
        port: 3000,
        protocol: Protocol::Tcp,
        pid: 1234,
        process_name: "ngrok".to_string(),
        command: "ngrok http 3000".to_string(),
        cwd: None,
        memory_mb: 50.0,
        cpu_percent: 1.0,
        uptime_secs: 100,
        project: None,
        docker: None,
        git: None,
        tunnel: Some(TunnelInfo {
            kind: "ngrok".to_string(),
            public_url: Some("abc123.ngrok.io".to_string()),
        }),
        status: Status::Healthy,
        health_check: None,
    };

    assert_eq!(entry.tunnel_display(), "ngrok → abc123.ngrok.io");
}

#[test]
fn test_tunnel_display_without_url() {
    let entry = PortEntry {
        port: 3000,
        protocol: Protocol::Tcp,
        pid: 1234,
        process_name: "ssh".to_string(),
        command: "ssh -R 8080:localhost:3000 user@server.com".to_string(),
        cwd: None,
        memory_mb: 50.0,
        cpu_percent: 1.0,
        uptime_secs: 100,
        project: None,
        docker: None,
        git: None,
        tunnel: Some(TunnelInfo {
            kind: "ssh".to_string(),
            public_url: None,
        }),
        status: Status::Healthy,
        health_check: None,
    };

    assert_eq!(entry.tunnel_display(), "ssh");
}

#[test]
fn test_tunnel_display_none() {
    let entry = PortEntry {
        port: 3000,
        protocol: Protocol::Tcp,
        pid: 1234,
        process_name: "node".to_string(),
        command: "node server.js".to_string(),
        cwd: None,
        memory_mb: 50.0,
        cpu_percent: 1.0,
        uptime_secs: 100,
        project: None,
        docker: None,
        git: None,
        tunnel: None,
        status: Status::Healthy,
        health_check: None,
    };

    assert_eq!(entry.tunnel_display(), "—");
}
