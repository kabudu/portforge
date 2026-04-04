use portforge::models::*;
use portforge::scanner;

#[test]
fn test_sort_entries_by_port() {
    let mut entries = vec![
        create_entry(8080, "b-app", Status::Healthy),
        create_entry(3000, "a-app", Status::Unknown),
        create_entry(5000, "c-app", Status::Zombie),
    ];

    scanner::sort_entries(&mut entries, SortField::Port, SortDirection::Ascending);
    assert_eq!(entries[0].port, 3000);
    assert_eq!(entries[1].port, 5000);
    assert_eq!(entries[2].port, 8080);

    scanner::sort_entries(&mut entries, SortField::Port, SortDirection::Descending);
    assert_eq!(entries[0].port, 8080);
    assert_eq!(entries[2].port, 3000);
}

#[test]
fn test_sort_entries_by_status() {
    let mut entries = vec![
        create_entry(3000, "app1", Status::Healthy),
        create_entry(3001, "app2", Status::Zombie),
        create_entry(3002, "app3", Status::Unknown),
    ];

    scanner::sort_entries(&mut entries, SortField::Status, SortDirection::Ascending);
    // Zombie (0) < Unknown (3) < Healthy (4)
    assert_eq!(entries[0].status, Status::Zombie);
    assert_eq!(entries[2].status, Status::Healthy);
}

#[test]
fn test_sort_entries_by_process_name() {
    let mut entries = vec![
        create_entry(3000, "Zebra", Status::Healthy),
        create_entry(3001, "alpha", Status::Healthy),
        create_entry(3002, "Beta", Status::Healthy),
    ];

    scanner::sort_entries(&mut entries, SortField::Process, SortDirection::Ascending);
    assert_eq!(entries[0].process_name, "alpha");
    assert_eq!(entries[1].process_name, "Beta");
    assert_eq!(entries[2].process_name, "Zebra");
}

#[test]
fn test_sort_entries_by_memory() {
    let mut entries = vec![
        create_entry_with_mem(3000, "a", 100.0),
        create_entry_with_mem(3001, "b", 50.0),
        create_entry_with_mem(3002, "c", 200.0),
    ];

    scanner::sort_entries(&mut entries, SortField::Memory, SortDirection::Descending);
    assert_eq!(entries[0].port, 3002); // 200MB
    assert_eq!(entries[2].port, 3001); // 50MB
}

// ─── Helpers ───

fn create_entry(port: u16, name: &str, status: Status) -> PortEntry {
    PortEntry {
        port,
        protocol: Protocol::Tcp,
        pid: 1000 + port as u32,
        process_name: name.to_string(),
        command: String::new(),
        cwd: None,
        memory_mb: 50.0,
        cpu_percent: 1.0,
        uptime_secs: 100,
        project: None,
        docker: None,
        git: None,
        status,
        health_check: None,
    }
}

fn create_entry_with_mem(port: u16, name: &str, mem: f64) -> PortEntry {
    let mut e = create_entry(port, name, Status::Healthy);
    e.memory_mb = mem;
    e
}
