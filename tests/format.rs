// Integration tests for the formatters via the public binary crate path.
// We re-parse a representative snapshot JSON (as the plugin emits it) and check
// each formatter's output shape.

use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_opensessions")
}

#[test]
fn help_runs() {
    let out = Command::new(bin()).arg("help").output().unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("open-sessionbar"));
    assert!(s.contains("waybar"));
}

#[test]
fn version_runs() {
    let out = Command::new(bin()).arg("--version").output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("open-sessionbar"));
}

#[test]
fn bar_unreachable_is_empty_waybar() {
    // Use an unlikely port so the plugin is not reachable; waybar -> empty class.
    let out = Command::new(bin())
        .args(["bar", "--format", "waybar", "--port", "6553"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(s.trim()).unwrap();
    assert_eq!(v["class"], "empty");
    assert_eq!(v["text"], "");
}

#[test]
fn json_unreachable_is_empty_snapshot() {
    let out = Command::new(bin())
        .args(["json", "--port", "6553"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
    assert_eq!(v["sessions"].as_array().unwrap().len(), 0);
}

#[test]
fn bad_format_errors() {
    let out = Command::new(bin())
        .args(["bar", "--format", "nope"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
