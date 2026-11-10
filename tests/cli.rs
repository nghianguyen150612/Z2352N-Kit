use std::process::Command;

#[test]
fn help_contains_safe_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_z2352n"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("Usage: z2352n"));
    assert!(text.contains("doctor"));
    assert!(text.contains("probe"));
    assert!(text.contains("unlock"));
}

#[test]
fn missing_file_hash_fails_without_writes() {
    let output = Command::new(env!("CARGO_BIN_EXE_z2352n"))
        .args(["research", "hash", "/nonexistent-z2352n-file"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn doctor_json_is_valid() {
    let output = Command::new(env!("CARGO_BIN_EXE_z2352n"))
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(parsed["adb"].is_boolean());
}
