use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_free_command_single_port() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("free").arg("9999").arg("--count").arg("1");
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"^\d+\n$").unwrap());
}

#[test]
fn test_free_command_multiple_ports() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("free").arg("18000").arg("--count").arg("3");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Free ports starting from 18000:"))
        .stdout(predicate::str::contains("18000"));
}

#[test]
fn test_free_command_json_output() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("free")
        .arg("19000")
        .arg("--count")
        .arg("2")
        .arg("--json");
    cmd.assert().success().stdout(predicate::str::contains("["));
}

#[test]
fn test_conflicts_command() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("conflicts");
    cmd.assert().success(); // Should succeed whether or not there are conflicts
}

#[test]
fn test_conflicts_command_specific_port() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("conflicts").arg("--port").arg("3000");
    cmd.assert().success();
}

#[test]
fn test_help_shows_new_commands() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("free"))
        .stdout(predicate::str::contains("conflicts"));
}

#[test]
fn test_free_help() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("free").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Find free ports"))
        .stdout(predicate::str::contains("--count"));
}

#[test]
fn test_conflicts_help() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("conflicts").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Detect port conflicts"))
        .stdout(predicate::str::contains("--port"));
}

#[test]
fn test_version_shows_package_version() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
