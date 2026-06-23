use portforge::export;
use portforge::models::{KubernetesInfo, PortEntry, Protocol, Status};

#[test]
fn test_json_export_includes_kubernetes_when_present() {
    let entry = entry_with_kubernetes();
    let json = export::to_json(&[entry], true).unwrap();

    assert!(json.contains("\"kubernetes\""));
    assert!(json.contains("\"resource_kind\": \"service\""));
    assert!(json.contains("\"resource_name\": \"api\""));
    assert!(json.contains("\"namespace\": \"dev\""));
}

#[test]
fn test_json_export_omits_kubernetes_when_absent() {
    let entry = base_entry();
    let json = export::to_json(&[entry], false).unwrap();

    assert!(!json.contains("kubernetes"));
}

#[test]
fn test_csv_export_includes_kubernetes_columns() {
    let entry = entry_with_kubernetes();
    let csv = export::to_csv(&[entry]);

    assert!(csv.starts_with("port,protocol,pid,process,project,framework,git_branch,git_dirty,tunnel,kubernetes_resource,kubernetes_namespace,kubernetes_context,kubernetes_local_port,kubernetes_remote_port,kubernetes_bind_address,docker,uptime_secs,memory_mb,cpu_percent,status\n"));
    assert!(csv.contains("service/api,dev,staging,18080,80,127.0.0.1"));
}

#[test]
fn test_csv_export_escapes_kubernetes_resource() {
    let mut entry = entry_with_kubernetes();
    entry.kubernetes.as_mut().unwrap().resource_name = "api,blue".to_string();

    let csv = export::to_csv(&[entry]);

    assert!(csv.contains("\"service/api,blue\""));
}

#[test]
fn test_table_export_includes_kubernetes_display() {
    let entry = entry_with_kubernetes();
    let table = export::to_table(&[entry]);

    assert!(table.contains("KUBERNETES"));
    assert!(table.contains("dev service/api:18080->80"));
}

fn entry_with_kubernetes() -> PortEntry {
    let mut entry = base_entry();
    entry.kubernetes = Some(KubernetesInfo {
        tool: "kubectl".to_string(),
        resource_kind: "service".to_string(),
        resource_name: "api".to_string(),
        namespace: Some("dev".to_string()),
        context: Some("staging".to_string()),
        local_port: 18080,
        remote_port: Some(80),
        bind_address: Some("127.0.0.1".to_string()),
    });
    entry
}

fn base_entry() -> PortEntry {
    PortEntry {
        port: 18080,
        protocol: Protocol::Tcp,
        pid: 1234,
        label: None,
        process_name: "kubectl".to_string(),
        command: "kubectl port-forward svc/api 18080:80".to_string(),
        cwd: None,
        memory_mb: 25.0,
        cpu_percent: 1.0,
        uptime_secs: 60,
        project: None,
        docker: None,
        git: None,
        tunnel: None,
        kubernetes: None,
        status: Status::Healthy,
        health_check: None,
    }
}
