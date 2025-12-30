use std::process::Command;
use tempfile::NamedTempFile;

fn get_workspace_root() -> String {
    // Assuming this test runs from target/debug/deps/ or similar
    // Go up to workspace root
    std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[test]
fn test_cli_list_instances() {
    let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "-p", "toy-cli", "--bin", "toy", "--", "list"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("instances"));
    assert!(stdout.contains("triangle_attractor_v0"));
}

#[test]
fn test_cli_status() {
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "status"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("status"));
    assert!(stdout.contains("version"));
}

#[test]
fn test_cli_run_instance() {
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "run", "--instance", "triangle_attractor_v0", "--ticks", "1"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("run_complete"));
}

#[test]
fn test_cli_format_json() {
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "--format", "json", "status"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should be valid JSON
    serde_json::from_str::<serde_json::Value>(&stdout).expect("Not valid JSON");
}

#[test]
fn test_cli_format_yaml() {
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "--format", "yaml", "status"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain YAML-like structure
    assert!(stdout.contains("status"));
}

#[test]
fn test_cli_save_and_load_snapshot() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_str().unwrap();

    // Save snapshot
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "save-snapshot", "--path", path, "--instance", "test"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run save command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("saved"));

    // Load snapshot
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "load-snapshot", "--path", path])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run load command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("loaded"));
}

#[test]
fn test_cli_unknown_instance() {
        let workspace_root = get_workspace_root();
        let workspace_root = get_workspace_root();
    let output = Command::new("cargo")
        .args(&["run", "--bin", "toy", "--", "run", "--instance", "unknown"])
        .current_dir(&workspace_root)
        .output()
        .expect("Failed to run command");

    // Check if error message appears in output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("unknown instance") || stderr.contains("unknown instance") || !output.status.success());
}