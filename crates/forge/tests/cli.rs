use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;

fn temp_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    env::temp_dir().join(format!("forge-cli-test-{label}-{nanos}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn run_forge(args: &[&str], config_dir: &Path, home_dir: &Path) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_forge");
    Command::new(bin)
        .args(args)
        .env("FORGE_CONFIG_DIR", config_dir)
        .env("HOME", home_dir)
        .current_dir(repo_root())
        .output()
        .expect("run forge")
}

#[test]
fn cli_install_and_status_use_mainline_user_target() {
    let root = temp_path("install-status");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    let install_root = home_dir.join(".agents").join("skills");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&install_root).expect("create install root");
    let repo_root = repo_root();
    let repo_root_str = repo_root.to_string_lossy().into_owned();

    let install = run_forge(
        &[
            "--json",
            "skills",
            "install",
            "--all",
            "--source",
            "repo",
            "--repo-path",
            repo_root_str.as_str(),
        ],
        &config_dir,
        &home_dir,
    );
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let install_stdout = String::from_utf8(install.stdout).expect("install stdout utf8");
    let install_json: Value = serde_json::from_str(&install_stdout).expect("install json");
    assert_eq!(install_json["data"]["target_kind"], "user");
    assert_eq!(install_json["data"]["target_role"], "mainline");

    let status = run_forge(&["--json", "skills", "status"], &config_dir, &home_dir);
    assert!(
        status.status.success(),
        "{}",
        String::from_utf8_lossy(&status.stderr)
    );
    let status_stdout = String::from_utf8(status.stdout).expect("status stdout utf8");
    let status_json: Value = serde_json::from_str(&status_stdout).expect("status json");
    assert_eq!(status_json["data"]["scope"], "mainline");
    let entries = status_json["data"]["entries"]
        .as_array()
        .expect("status entries array");
    assert!(
        entries
            .iter()
            .any(|entry| { entry["target_kind"] == "user" && entry["target_role"] == "mainline" })
    );
    assert!(!entries.iter().any(|entry| entry["target_kind"] == "path"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_errors_have_stable_codes_for_common_mistakes() {
    let root = temp_path("error-codes");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let invalid_target = run_forge(
        &[
            "--json", "skills", "install", "--all", "--source", "release", "--target", "path:rel",
        ],
        &config_dir,
        &home_dir,
    );
    assert!(!invalid_target.status.success());
    let err: Value = serde_json::from_str(String::from_utf8(invalid_target.stderr).unwrap().trim())
        .expect("error json");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "invalid_target");

    let invalid_usage = run_forge(&["--json", "skills", "validate"], &config_dir, &home_dir);
    assert!(!invalid_usage.status.success());
    let err: Value = serde_json::from_str(String::from_utf8(invalid_usage.stderr).unwrap().trim())
        .expect("error json");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "invalid_usage");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_self_update_reports_all_unmanaged_collisions_actionably() {
    let root = temp_path("self-update-collisions");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    let unmanaged_root = home_dir.join(".agents").join("skills");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&unmanaged_root).expect("create unmanaged root");

    // Create an unmanaged skill dir that matches a repo-provided skill name.
    // This should produce an unmanaged_collision during self update.
    let skill_dir = unmanaged_root.join("forge-tools");
    fs::create_dir_all(&skill_dir).expect("create unmanaged skill dir");
    fs::write(skill_dir.join("SKILL.md"), "unmanaged").expect("write unmanaged skill file");

    let update = run_forge(&["--json", "self", "update"], &config_dir, &home_dir);
    assert!(!update.status.success());
    let stderr = String::from_utf8(update.stderr).expect("stderr utf8");
    let err_json: Value = serde_json::from_str(stderr.trim()).expect("error json");
    assert_eq!(err_json["ok"], false);
    assert_eq!(err_json["error"]["code"], "unmanaged_collision");
    let msg = err_json["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("forge-tools"));
    assert!(msg.contains("forge skills install --all --force-unmanaged"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_parse_errors_honor_json_flag() {
    let root = temp_path("parse-error-json");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let output = run_forge(
        &[
            "--json",
            "skills",
            "revert",
            "linear-cli",
            "--source",
            "release",
        ],
        &config_dir,
        &home_dir,
    );
    assert!(!output.status.success());
    let err: Value = serde_json::from_str(String::from_utf8(output.stderr).unwrap().trim())
        .expect("parse error json");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "invalid_usage");
    assert!(
        err["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("unexpected argument '--source'")
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_self_update_rejects_repo_mode_flags() {
    let root = temp_path("self-update-release-only");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let output = run_forge(
        &["--json", "self", "update", "--repo-path", "/tmp/forge"],
        &config_dir,
        &home_dir,
    );
    assert!(!output.status.success());
    let err: Value = serde_json::from_str(String::from_utf8(output.stderr).unwrap().trim())
        .expect("parse error json");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "invalid_usage");
    assert!(
        err["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("unexpected argument '--repo-path'")
    );

    let _ = fs::remove_dir_all(root);
}
