use std::{path::PathBuf, process::Command};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn run_linear(args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_linear");
    Command::new(bin)
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("run linear")
}

#[test]
fn cli_parse_errors_honor_json_flag() {
    let output = run_linear(&["--json", "issue", "read"]);
    assert!(!output.status.success());

    let err: Value = serde_json::from_str(String::from_utf8(output.stderr).unwrap().trim())
        .expect("parse error json");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "invalid_usage");
    assert!(
        err["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("the following required arguments were not provided")
    );
}

#[test]
fn cli_parse_errors_default_to_human_output() {
    let output = run_linear(&["issue", "read"]);
    assert!(!output.status.success());

    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(stderr.contains("the following required arguments were not provided"));
    assert!(stderr.contains("Usage: linear issue read <ISSUE_ID>"));
    assert!(!stderr.contains("{\"ok\":false"));
}
