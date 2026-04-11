use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

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

fn run_forge(args: &[&str], config_dir: &Path) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_forge");
    Command::new(bin)
        .args(args)
        .env("FORGE_CONFIG_DIR", config_dir)
        .current_dir(repo_root())
        .output()
        .expect("run forge")
}

#[test]
fn cli_install_and_status_use_mainline_user_target() {
    let root = temp_path("install-status");
    let config_dir = root.join("config");
    let install_root = root.join("home").join(".agents").join("skills");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&install_root).expect("create install root");
    fs::write(
        config_dir.join("config.toml"),
        format!("skills_user_dir = {:?}\n", install_root.display().to_string()),
    )
    .expect("write config");

    let install = run_forge(
        &[
            "--json",
            "skills",
            "install",
            "--all",
            "--source",
            "repo",
            "--target",
            "user",
        ],
        &config_dir,
    );
    assert!(install.status.success(), "{}", String::from_utf8_lossy(&install.stderr));
    let install_stdout = String::from_utf8(install.stdout).expect("install stdout utf8");
    assert!(install_stdout.contains("\"target_kind\": \"user\""));
    assert!(install_stdout.contains("\"target_role\": \"mainline\""));

    let status = run_forge(&["--json", "skills", "status"], &config_dir);
    assert!(status.status.success(), "{}", String::from_utf8_lossy(&status.stderr));
    let status_stdout = String::from_utf8(status.stdout).expect("status stdout utf8");
    assert!(status_stdout.contains("\"scope\": \"mainline\""));
    assert!(status_stdout.contains("\"target_kind\": \"user\""));
    assert!(status_stdout.contains("\"target_role\": \"mainline\""));
    assert!(!status_stdout.contains("\"target_kind\": \"path\""));

    let _ = fs::remove_dir_all(root);
}
