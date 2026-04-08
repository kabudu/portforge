use portforge::tui::app::Tab;

#[test]
fn test_tab_labels() {
    assert_eq!(Tab::Ports.label(), "Ports");
    assert_eq!(Tab::Processes.label(), "Processes");
    assert_eq!(Tab::Docker.label(), "Docker");
    assert_eq!(Tab::Logs.label(), "Logs");
}

#[test]
fn test_tab_next() {
    assert_eq!(Tab::Ports.next(), Tab::Processes);
    assert_eq!(Tab::Processes.next(), Tab::Docker);
    assert_eq!(Tab::Docker.next(), Tab::Logs);
    assert_eq!(Tab::Logs.next(), Tab::Ports); // wraps
}

#[test]
fn test_tab_prev() {
    assert_eq!(Tab::Ports.prev(), Tab::Logs); // wraps
    assert_eq!(Tab::Processes.prev(), Tab::Ports);
    assert_eq!(Tab::Docker.prev(), Tab::Processes);
    assert_eq!(Tab::Logs.prev(), Tab::Docker);
}

#[test]
fn test_tab_cycle_all() {
    let mut tab = Tab::Ports;
    tab = tab.next();
    assert_eq!(tab, Tab::Processes);
    tab = tab.next();
    assert_eq!(tab, Tab::Docker);
    tab = tab.next();
    assert_eq!(tab, Tab::Logs);
    tab = tab.next();
    assert_eq!(tab, Tab::Ports); // back to start
}
