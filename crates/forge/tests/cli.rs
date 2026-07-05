use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::Value;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    run_forge_with_env(args, config_dir, home_dir, &[])
}

fn run_forge_with_env(
    args: &[&str],
    config_dir: &Path,
    home_dir: &Path,
    extra_envs: &[(&str, &str)],
) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_forge");
    let mut command = Command::new(bin);
    command
        .args(args)
        .env("FORGE_CONFIG_DIR", config_dir)
        .env("HOME", home_dir)
        .current_dir(repo_root());
    for (key, value) in extra_envs {
        command.env(key, value);
    }
    command.output().expect("run forge")
}

#[cfg(unix)]
fn write_executable_script(path: &Path, body: &str) {
    fs::write(path, body).expect("write script");
    let mut perms = fs::metadata(path).expect("script metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod script");
}

#[cfg(unix)]
fn fake_pnpm_script() -> &'static str {
    r#"#!/bin/sh
set -eu
log="${FORGE_TEST_PNPM_LOG:?missing log path}"
printf '%s\n' "$@" > "$log"
out=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o)
      shift
      out="$1"
      ;;
  esac
  shift
done
if [ -n "$out" ]; then
  mkdir -p "$(dirname "$out")"
  printf '%s\n' '<svg>fake</svg>' > "$out"
fi
printf '%s\n' 'ok'
"#
}

#[cfg(unix)]
fn fake_tool_cargo_script() -> &'static str {
    r#"#!/bin/sh
set -eu
log="${FORGE_TEST_CARGO_LOG:-}"
if [ "${1:-}" = "install" ] && [ "${2:-}" = "--list" ]; then
  printf '%s\n' 'bat v0.25.0:'
  printf '%s\n' '  bat'
  printf '%s\n' 'ripgrep v14.1.1:'
  printf '%s\n' '  rg'
  exit 0
fi
if [ -n "$log" ]; then
  printf '%s\n' "$*" > "$log"
fi
printf '%s\n' 'updated'
"#
}

#[cfg(unix)]
fn fake_tool_success_script() -> &'static str {
    r#"#!/bin/sh
set -eu
if [ -n "${FORGE_TEST_TOOL_LOG:-}" ]; then
  printf '%s\n' "$0 $*" >> "$FORGE_TEST_TOOL_LOG"
fi
printf '%s\n' 'ok'
"#
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

    let skill_dir = unmanaged_root.join("forge-tools");
    fs::create_dir_all(&skill_dir).expect("create unmanaged skill dir");
    fs::write(skill_dir.join("SKILL.md"), "unmanaged").expect("write unmanaged skill file");

    let update = run_forge(&["--json", "self", "update"], &config_dir, &home_dir);
    assert!(!update.status.success());
    let stderr = String::from_utf8(update.stderr).expect("stderr utf8");
    let err_json: Value = serde_json::from_str(stderr.trim()).expect("error json");
    assert_eq!(err_json["ok"], false);
    assert_eq!(
        err_json["error"]["code"], "unmanaged_collision",
        "{err_json}"
    );
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
fn cli_version_is_available_in_json() {
    let root = temp_path("version-json");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let output = run_forge(&["--json", "version"], &config_dir, &home_dir);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let body: Value = serde_json::from_str(stdout.trim()).expect("version json");
    assert_eq!(body["ok"], true);
    let data = &body["data"];
    assert_eq!(
        data["release_version"].as_str(),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert!(data["latest_version"].is_string() || data["latest_version"].is_null());
    assert!(data["update_available"].is_boolean());
    assert!(data["git_hash"].is_string() || data["git_hash"].is_null());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_version_is_available_human() {
    let root = temp_path("version-human");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let output = run_forge(&["version"], &config_dir, &home_dir);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    assert!(stdout.contains("forge version"));
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));

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

#[cfg(unix)]
#[test]
fn cli_tool_update_dry_run_reports_global_commands() {
    let root = temp_path("tool-update-dry-run");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let fake_cargo = root.join("fake-cargo");
    let fake_uv = root.join("fake-uv");
    let fake_rustup = root.join("fake-rustup");
    let fake_brew = root.join("fake-brew");
    let missing_gum = root.join("missing-gum");
    write_executable_script(&fake_cargo, fake_tool_cargo_script());
    write_executable_script(&fake_uv, fake_tool_success_script());
    write_executable_script(&fake_rustup, fake_tool_success_script());
    write_executable_script(&fake_brew, fake_tool_success_script());
    let fake_cargo_str = fake_cargo.to_string_lossy().into_owned();
    let fake_uv_str = fake_uv.to_string_lossy().into_owned();
    let fake_rustup_str = fake_rustup.to_string_lossy().into_owned();
    let fake_brew_str = fake_brew.to_string_lossy().into_owned();
    let missing_gum_str = missing_gum.to_string_lossy().into_owned();

    let output = run_forge_with_env(
        &["--json", "tool", "update", "--dry-run"],
        &config_dir,
        &home_dir,
        &[
            ("FORGE_TOOL_UPDATE_CARGO_BIN", fake_cargo_str.as_str()),
            ("FORGE_TOOL_UPDATE_UV_BIN", fake_uv_str.as_str()),
            ("FORGE_TOOL_UPDATE_RUSTUP_BIN", fake_rustup_str.as_str()),
            ("FORGE_TOOL_UPDATE_BREW_BIN", fake_brew_str.as_str()),
            ("FORGE_TOOL_UPDATE_GUM_BIN", missing_gum_str.as_str()),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let body: Value = serde_json::from_str(stdout.trim()).expect("tool update json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["dry_run"], true);
    assert_eq!(body["data"]["summary"]["planned"], 6);

    let entries = body["data"]["entries"].as_array().expect("entries array");
    let ids = entries
        .iter()
        .map(|entry| entry["id"].as_str().unwrap_or(""))
        .collect::<Vec<_>>();
    assert_eq!(
        ids,
        vec![
            "packages",
            "rustup",
            "uv",
            "uv-tools",
            "cargo-installs",
            "gum"
        ]
    );
    assert_eq!(entries[0]["source"].as_str(), Some("homebrew"));
    assert_eq!(
        entries[1]["command"].as_array().unwrap()[0].as_str(),
        Some(fake_rustup_str.as_str())
    );
    assert_eq!(
        entries[1]["command"].as_array().unwrap()[1].as_str(),
        Some("update")
    );
    assert_eq!(
        entries[2]["env"].as_array().unwrap()[0].as_str(),
        Some("UV_NO_MODIFY_PATH=1")
    );
    assert_eq!(entries[2]["source"].as_str(), Some("uv_self"));
    assert_eq!(
        entries[4]["command"].as_array().unwrap()[0].as_str(),
        Some(fake_cargo_str.as_str())
    );
    assert!(
        entries[4]["command"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str() == Some("bat"))
    );
    assert!(
        entries[4]["command"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item.as_str() == Some("ripgrep"))
    );
    assert_eq!(
        entries[5]["command"].as_array().unwrap()[1].as_str(),
        Some("install")
    );
    assert_eq!(
        entries[5]["command"].as_array().unwrap()[2].as_str(),
        Some("gum")
    );

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn cli_tool_update_cargo_installs_runs_parsed_packages() {
    let root = temp_path("tool-update-cargo");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let fake_cargo = root.join("fake-cargo");
    let log_path = root.join("cargo.log");
    write_executable_script(&fake_cargo, fake_tool_cargo_script());
    let fake_cargo_str = fake_cargo.to_string_lossy().into_owned();
    let log_path_str = log_path.to_string_lossy().into_owned();

    let output = run_forge_with_env(
        &["--json", "tool", "update", "cargo-installs"],
        &config_dir,
        &home_dir,
        &[
            ("FORGE_TOOL_UPDATE_CARGO_BIN", fake_cargo_str.as_str()),
            ("FORGE_TEST_CARGO_LOG", log_path_str.as_str()),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let body: Value = serde_json::from_str(stdout.trim()).expect("tool update json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["summary"]["succeeded"], 1);
    assert_eq!(
        fs::read_to_string(&log_path).expect("read cargo log"),
        "install bat ripgrep\n"
    );

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn cli_tool_update_gum_installs_when_missing() {
    let root = temp_path("tool-update-gum");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let fake_brew = root.join("fake-brew");
    let missing_gum = root.join("missing-gum");
    let log_path = root.join("tool.log");
    write_executable_script(&fake_brew, fake_tool_success_script());
    let fake_brew_str = fake_brew.to_string_lossy().into_owned();
    let missing_gum_str = missing_gum.to_string_lossy().into_owned();
    let log_path_str = log_path.to_string_lossy().into_owned();

    let output = run_forge_with_env(
        &["--json", "tool", "update", "gum"],
        &config_dir,
        &home_dir,
        &[
            ("FORGE_TOOL_UPDATE_BREW_BIN", fake_brew_str.as_str()),
            ("FORGE_TOOL_UPDATE_GUM_BIN", missing_gum_str.as_str()),
            ("FORGE_TEST_TOOL_LOG", log_path_str.as_str()),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let body: Value = serde_json::from_str(stdout.trim()).expect("tool update json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["summary"]["succeeded"], 1);
    assert_eq!(
        fs::read_to_string(&log_path).expect("read tool log"),
        format!("{fake_brew_str} install gum\n")
    );

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn cli_bytefield_install_prefetches_pinned_runner() {
    let root = temp_path("bytefield-install");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let log_path = root.join("pnpm.log");
    let fake_pnpm = root.join("fake-pnpm");
    write_executable_script(&fake_pnpm, fake_pnpm_script());
    let fake_pnpm_str = fake_pnpm.to_string_lossy().into_owned();
    let log_path_str = log_path.to_string_lossy().into_owned();

    let output = run_forge_with_env(
        &["--json", "bytefield", "install"],
        &config_dir,
        &home_dir,
        &[
            ("FORGE_BYTEFIELD_PNPM_BIN", fake_pnpm_str.as_str()),
            ("FORGE_TEST_PNPM_LOG", log_path_str.as_str()),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let body: Value = serde_json::from_str(stdout.trim()).expect("install json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["ready"], true);
    assert_eq!(
        body["data"]["package_spec"].as_str(),
        Some("bytefield-svg@1.11.0")
    );

    let log = fs::read_to_string(&log_path).expect("read pnpm log");
    assert!(log.contains("--package=bytefield-svg@1.11.0"));
    assert!(log.contains("dlx"));
    assert!(log.contains("bytefield-svg"));
    assert!(log.contains("--help"));

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn cli_bytefield_render_writes_svg_via_wrapper() {
    let root = temp_path("bytefield-render");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let source = root.join("diagram.bf.clj");
    fs::write(
        &source,
        "(draw-column-headers)\n(draw-box \"Version\" {:span 4})\n",
    )
    .expect("write source");
    let output_svg = root.join("out").join("diagram.svg");

    let log_path = root.join("pnpm.log");
    let fake_pnpm = root.join("fake-pnpm");
    write_executable_script(&fake_pnpm, fake_pnpm_script());
    let fake_pnpm_str = fake_pnpm.to_string_lossy().into_owned();
    let log_path_str = log_path.to_string_lossy().into_owned();
    let source_str = source.to_string_lossy().into_owned();
    let output_svg_str = output_svg.to_string_lossy().into_owned();

    let output = run_forge_with_env(
        &[
            "--json",
            "bytefield",
            "render",
            "--source",
            source_str.as_str(),
            "--output",
            output_svg_str.as_str(),
            "--embedded",
        ],
        &config_dir,
        &home_dir,
        &[
            ("FORGE_BYTEFIELD_PNPM_BIN", fake_pnpm_str.as_str()),
            ("FORGE_TEST_PNPM_LOG", log_path_str.as_str()),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let body: Value = serde_json::from_str(stdout.trim()).expect("render json");
    assert_eq!(body["ok"], true);
    assert_eq!(body["data"]["embedded"], true);
    assert_eq!(
        body["data"]["output_path"].as_str(),
        Some(output_svg_str.as_str())
    );
    assert_eq!(
        fs::read_to_string(&output_svg).expect("read output svg"),
        "<svg>fake</svg>\n"
    );

    let log = fs::read_to_string(&log_path).expect("read pnpm log");
    assert!(log.contains("--package=bytefield-svg@1.11.0"));
    assert!(log.contains("dlx"));
    assert!(log.contains("bytefield-svg"));
    assert!(log.contains("-s"));
    assert!(log.contains(source_str.as_str()));
    assert!(log.contains("-o"));
    assert!(log.contains(output_svg_str.as_str()));
    assert!(log.contains("-e"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn cli_bytefield_render_reports_missing_source_stably() {
    let root = temp_path("bytefield-missing-source");
    let config_dir = root.join("config");
    let home_dir = root.join("home");
    fs::create_dir_all(&config_dir).expect("create config dir");
    fs::create_dir_all(&home_dir).expect("create home dir");

    let missing = root.join("missing.bf.clj");
    let output_svg = root.join("diagram.svg");
    let missing_str = missing.to_string_lossy().into_owned();
    let output_svg_str = output_svg.to_string_lossy().into_owned();

    let output = run_forge(
        &[
            "--json",
            "bytefield",
            "render",
            "--source",
            missing_str.as_str(),
            "--output",
            output_svg_str.as_str(),
        ],
        &config_dir,
        &home_dir,
    );
    assert!(!output.status.success());
    let err: Value =
        serde_json::from_str(String::from_utf8(output.stderr).unwrap().trim()).expect("error json");
    assert_eq!(err["ok"], false);
    assert_eq!(err["error"]["code"], "bytefield_source_not_found");

    let _ = fs::remove_dir_all(root);
}
