use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("portforge"));
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("inspector and manager"))
        .stdout(predicate::str::contains("kill"))
        .stdout(predicate::str::contains("clean"))
        .stdout(predicate::str::contains("watch"));
}

#[test]
fn test_cli_kill_help() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.args(["kill", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("force"));
}

#[test]
fn test_cli_clean_help() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.args(["clean", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dry-run"));
}

#[test]
fn test_cli_export_help() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.args(["export", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("format"));
}

#[test]
fn test_cli_json_output() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.args(["ps", "--json", "--all"]).assert().success();
}

#[test]
fn test_cli_inspect_nonexistent_port() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.args(["inspect", "59999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No process found"));
}

#[test]
fn test_cli_init_config() {
    let mut cmd = Command::cargo_bin("portforge").unwrap();
    cmd.arg("init-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("configuration"));
}
