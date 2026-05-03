use std::{
    env,
    fmt::Write as _,
    io,
    path::PathBuf,
    process::{Command as ProcessCommand, Output},
};

use anyhow::Result;
use clap::Args;
use cli_core::OutputMode;
use serde::Serialize;

use crate::UpdateProgress;

mod catalog;

use catalog::{TARGET_CATALOG, Target};

const GUM_GO_PACKAGE: &str = "github.com/charmbracelet/gum@latest";

#[derive(Args, Debug)]
pub(crate) struct UpdateArgs {
    #[arg(
        help = "Tool id or binary to update; repeat to select multiple (default: all known global tools)"
    )]
    tool: Vec<String>,
    #[arg(
        long,
        help = "Show planned global update commands without running them"
    )]
    dry_run: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct UpdateResult {
    dry_run: bool,
    requested: Vec<String>,
    summary: Summary,
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize)]
struct Summary {
    planned: usize,
    skipped: usize,
    succeeded: usize,
    failed: usize,
}

#[derive(Debug, Serialize)]
struct Entry {
    id: String,
    label: String,
    source: String,
    status: String,
    command: Vec<String>,
    env: Vec<String>,
    items: Vec<String>,
    exit_code: Option<i32>,
    detail: Option<String>,
}

#[derive(Debug, Clone)]
struct Plan {
    id: String,
    label: String,
    source: String,
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    items: Vec<String>,
}

pub(crate) fn update(args: UpdateArgs, output: OutputMode) -> Result<UpdateResult> {
    let requested = args.tool.clone();
    let selected = select_targets(&requested)?;
    let total_steps = selected.len();
    let progress = UpdateProgress::new(output);
    let mut entries = Vec::new();

    for (index, target) in selected.into_iter().enumerate() {
        progress.step(format!(
            "[{}/{}] {}",
            index + 1,
            total_steps,
            target_progress_message(target, args.dry_run)
        ));
        match target {
            Target::Packages => entries.push(run_plan(packages_plan(), args.dry_run)),
            Target::Rustup => entries.push(run_plan(rustup_plan(), args.dry_run)),
            Target::Uv => entries.push(update_uv(args.dry_run)),
            Target::UvTools => entries.push(run_plan(uv_tools_plan(), args.dry_run)),
            Target::CargoInstalls => entries.push(update_cargo_installs(args.dry_run)),
            Target::Gum => entries.push(update_gum(args.dry_run)),
        }
    }

    Ok(UpdateResult {
        dry_run: args.dry_run,
        requested,
        summary: summary(&entries),
        entries,
    })
}

fn select_targets(requested: &[String]) -> Result<Vec<Target>> {
    if requested.is_empty() {
        return Ok(TARGET_CATALOG.iter().map(|entry| entry.target).collect());
    }

    let mut selected = Vec::new();
    for item in requested {
        let target = Target::from_raw(item)?;
        if !selected.contains(&target) {
            selected.push(target);
        }
    }
    Ok(selected)
}

fn target_progress_message(target: Target, dry_run: bool) -> &'static str {
    if dry_run {
        return match target {
            Target::Packages => "Planning package-manager updates",
            Target::Rustup => "Planning Rust toolchain updates",
            Target::Uv => "Planning uv update",
            Target::UvTools => "Planning uv-installed tool updates",
            Target::CargoInstalls => "Planning cargo-installed binary updates",
            Target::Gum => "Planning gum command install",
        };
    }

    match target {
        Target::Packages => "Updating package-manager packages",
        Target::Rustup => "Updating Rust toolchains",
        Target::Uv => "Updating uv",
        Target::UvTools => "Updating uv-installed tools",
        Target::CargoInstalls => "Updating cargo-installed binaries",
        Target::Gum => "Ensuring gum command is installed",
    }
}

fn summary(entries: &[Entry]) -> Summary {
    Summary {
        planned: entries
            .iter()
            .filter(|entry| entry.status == "planned")
            .count(),
        skipped: entries
            .iter()
            .filter(|entry| entry.status == "skipped")
            .count(),
        succeeded: entries
            .iter()
            .filter(|entry| entry.status == "succeeded")
            .count(),
        failed: entries
            .iter()
            .filter(|entry| entry.status == "failed")
            .count(),
    }
}

fn packages_plan() -> Plan {
    packages_plan_for(
        env::consts::OS,
        env::var("FORGE_TOOL_UPDATE_BREW_BIN").unwrap_or_else(|_| "brew".to_string()),
        env::var("FORGE_TOOL_UPDATE_WINGET_BIN").unwrap_or_else(|_| "winget".to_string()),
    )
}

fn packages_plan_for(os: &str, brew_bin: String, winget_bin: String) -> Plan {
    match os {
        "windows" => Plan {
            id: "packages".to_string(),
            label: "WinGet packages".to_string(),
            source: "winget".to_string(),
            program: winget_bin,
            args: vec![
                "upgrade".to_string(),
                "--all".to_string(),
                "--accept-package-agreements".to_string(),
                "--accept-source-agreements".to_string(),
            ],
            env: Vec::new(),
            items: Vec::new(),
        },
        _ => Plan {
            id: "packages".to_string(),
            label: "Homebrew packages".to_string(),
            source: "homebrew".to_string(),
            program: brew_bin,
            args: vec!["upgrade".to_string()],
            env: Vec::new(),
            items: Vec::new(),
        },
    }
}

fn rustup_plan() -> Plan {
    Plan {
        id: "rustup".to_string(),
        label: "Rust toolchains".to_string(),
        source: "rustup".to_string(),
        program: env::var("FORGE_TOOL_UPDATE_RUSTUP_BIN").unwrap_or_else(|_| "rustup".to_string()),
        args: vec!["update".to_string()],
        env: Vec::new(),
        items: Vec::new(),
    }
}

fn update_uv(dry_run: bool) -> Entry {
    let uv = env::var("FORGE_TOOL_UPDATE_UV_BIN").unwrap_or_else(|_| "uv".to_string());
    if command_succeeds(&uv, &["--version"]) {
        return run_plan(uv_plan(uv), dry_run);
    }

    run_plan(uv_install_plan(), dry_run)
}

fn uv_plan(uv_bin: String) -> Plan {
    Plan {
        id: "uv".to_string(),
        label: "uv standalone install".to_string(),
        source: "uv_self".to_string(),
        program: uv_bin,
        args: vec!["self".to_string(), "update".to_string()],
        env: vec![("UV_NO_MODIFY_PATH".to_string(), "1".to_string())],
        items: Vec::new(),
    }
}

fn uv_install_plan() -> Plan {
    uv_install_plan_for(env::consts::OS)
}

fn uv_install_plan_for(os: &str) -> Plan {
    match os {
        "windows" => Plan {
            id: "uv".to_string(),
            label: "uv command".to_string(),
            source: "uv_standalone_installer".to_string(),
            program: env::var("FORGE_TOOL_UPDATE_POWERSHELL_BIN")
                .unwrap_or_else(|_| "powershell".to_string()),
            args: vec![
                "-ExecutionPolicy".to_string(),
                "ByPass".to_string(),
                "-c".to_string(),
                "irm https://astral.sh/uv/install.ps1 | iex".to_string(),
            ],
            env: Vec::new(),
            items: vec!["uv".to_string()],
        },
        _ => Plan {
            id: "uv".to_string(),
            label: "uv command".to_string(),
            source: "uv_standalone_installer".to_string(),
            program: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "curl -LsSf https://astral.sh/uv/install.sh | sh".to_string(),
            ],
            env: Vec::new(),
            items: vec!["uv".to_string()],
        },
    }
}

fn uv_tools_plan() -> Plan {
    Plan {
        id: "uv-tools".to_string(),
        label: "uv-installed tools".to_string(),
        source: "uv_tool".to_string(),
        program: env::var("FORGE_TOOL_UPDATE_UV_BIN").unwrap_or_else(|_| "uv".to_string()),
        args: vec![
            "tool".to_string(),
            "upgrade".to_string(),
            "--all".to_string(),
        ],
        env: Vec::new(),
        items: Vec::new(),
    }
}

fn update_cargo_installs(dry_run: bool) -> Entry {
    let cargo = env::var("FORGE_TOOL_UPDATE_CARGO_BIN").unwrap_or_else(|_| "cargo".to_string());
    let mut list_command = ProcessCommand::new(&cargo);
    list_command.args(["install", "--list"]);
    if let Some(cwd) = global_tool_update_cwd() {
        list_command.current_dir(cwd);
    }
    let list_output = match list_command.output() {
        Ok(output) => output,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Entry {
                id: "cargo-installs".to_string(),
                label: "Cargo-installed binaries".to_string(),
                source: "cargo_install".to_string(),
                status: "skipped".to_string(),
                command: vec![cargo, "install".to_string(), "--list".to_string()],
                env: Vec::new(),
                items: Vec::new(),
                exit_code: None,
                detail: Some("cargo is not available".to_string()),
            };
        }
        Err(err) => {
            return Entry {
                id: "cargo-installs".to_string(),
                label: "Cargo-installed binaries".to_string(),
                source: "cargo_install".to_string(),
                status: "failed".to_string(),
                command: vec![cargo, "install".to_string(), "--list".to_string()],
                env: Vec::new(),
                items: Vec::new(),
                exit_code: None,
                detail: Some(err.to_string()),
            };
        }
    };

    if !list_output.status.success() {
        return Entry {
            id: "cargo-installs".to_string(),
            label: "Cargo-installed binaries".to_string(),
            source: "cargo_install".to_string(),
            status: "failed".to_string(),
            command: vec![cargo, "install".to_string(), "--list".to_string()],
            env: Vec::new(),
            items: Vec::new(),
            exit_code: list_output.status.code(),
            detail: first_output_line(&list_output),
        };
    }

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    let packages = parse_cargo_install_list_packages(&stdout);
    if packages.is_empty() {
        return Entry {
            id: "cargo-installs".to_string(),
            label: "Cargo-installed binaries".to_string(),
            source: "cargo_install".to_string(),
            status: "skipped".to_string(),
            command: vec![cargo, "install".to_string(), "--list".to_string()],
            env: Vec::new(),
            items: Vec::new(),
            exit_code: Some(0),
            detail: Some("no cargo-installed packages found".to_string()),
        };
    }

    let mut command = vec![cargo.clone(), "install".to_string()];
    command.extend(packages.iter().cloned());
    let plan = Plan {
        id: "cargo-installs".to_string(),
        label: "Cargo-installed binaries".to_string(),
        source: "cargo_install".to_string(),
        program: cargo,
        args: command.iter().skip(1).cloned().collect(),
        env: Vec::new(),
        items: packages,
    };
    run_plan(plan, dry_run)
}

fn update_gum(dry_run: bool) -> Entry {
    let gum = env::var("FORGE_TOOL_UPDATE_GUM_BIN").unwrap_or_else(|_| "gum".to_string());
    if command_succeeds(&gum, &["--version"]) {
        return Entry {
            id: "gum".to_string(),
            label: "gum command".to_string(),
            source: "existing_path".to_string(),
            status: "skipped".to_string(),
            command: vec![gum, "--version".to_string()],
            env: Vec::new(),
            items: vec!["gum".to_string()],
            exit_code: Some(0),
            detail: Some("gum is already installed".to_string()),
        };
    }

    let plans = gum_install_plans();
    if dry_run {
        return run_plan(plans[0].clone(), true);
    }

    let mut last_skipped = None;
    for plan in plans {
        let entry = run_plan(plan, false);
        if entry.status != "skipped" {
            return entry;
        }
        last_skipped = Some(entry);
    }

    last_skipped.unwrap_or_else(|| Entry {
        id: "gum".to_string(),
        label: "gum command".to_string(),
        source: "none".to_string(),
        status: "skipped".to_string(),
        command: Vec::new(),
        env: Vec::new(),
        items: vec!["gum".to_string()],
        exit_code: None,
        detail: Some("no supported installer was available for gum".to_string()),
    })
}

fn gum_install_plans() -> Vec<Plan> {
    gum_install_plans_for(
        env::consts::OS,
        env::var("FORGE_TOOL_UPDATE_BREW_BIN").unwrap_or_else(|_| "brew".to_string()),
        env::var("FORGE_TOOL_UPDATE_WINGET_BIN").unwrap_or_else(|_| "winget".to_string()),
        env::var("FORGE_TOOL_UPDATE_GO_BIN").unwrap_or_else(|_| "go".to_string()),
    )
}

fn gum_install_plans_for(
    os: &str,
    brew_bin: String,
    winget_bin: String,
    go_bin: String,
) -> Vec<Plan> {
    match os {
        "windows" => vec![Plan {
            id: "gum".to_string(),
            label: "gum command".to_string(),
            source: "winget".to_string(),
            program: winget_bin,
            args: vec![
                "install".to_string(),
                "--id".to_string(),
                "charmbracelet.gum".to_string(),
                "-e".to_string(),
                "--accept-package-agreements".to_string(),
                "--accept-source-agreements".to_string(),
            ],
            env: Vec::new(),
            items: vec!["gum".to_string()],
        }],
        "macos" | "linux" => vec![
            Plan {
                id: "gum".to_string(),
                label: "gum command".to_string(),
                source: "homebrew".to_string(),
                program: brew_bin,
                args: vec!["install".to_string(), "gum".to_string()],
                env: Vec::new(),
                items: vec!["gum".to_string()],
            },
            Plan {
                id: "gum".to_string(),
                label: "gum command".to_string(),
                source: "go_install".to_string(),
                program: go_bin,
                args: vec!["install".to_string(), GUM_GO_PACKAGE.to_string()],
                env: Vec::new(),
                items: vec!["gum".to_string()],
            },
        ],
        _ => vec![Plan {
            id: "gum".to_string(),
            label: "gum command".to_string(),
            source: "go_install".to_string(),
            program: go_bin,
            args: vec!["install".to_string(), GUM_GO_PACKAGE.to_string()],
            env: Vec::new(),
            items: vec!["gum".to_string()],
        }],
    }
}

fn parse_cargo_install_list_packages(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(parse_cargo_install_list_package_line)
        .collect()
}

fn parse_cargo_install_list_package_line(line: &str) -> Option<String> {
    let line = line.trim();
    let line = line.strip_suffix(':')?;
    let (name, version) = line.split_once(" v")?;
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return None;
    }
    if version.is_empty() || !version.chars().all(|ch| ch.is_ascii_digit() || ch == '.') {
        return None;
    }
    Some(name.to_string())
}

fn command_succeeds(program: &str, args: &[&str]) -> bool {
    ProcessCommand::new(program)
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_plan(plan: Plan, dry_run: bool) -> Entry {
    let command = command_for_display(&plan);
    let env = env_for_display(&plan);
    if dry_run {
        return Entry {
            id: plan.id,
            label: plan.label,
            source: plan.source,
            status: "planned".to_string(),
            command,
            env,
            items: plan.items,
            exit_code: None,
            detail: None,
        };
    }

    let mut process = ProcessCommand::new(&plan.program);
    process.args(&plan.args);
    if let Some(cwd) = global_tool_update_cwd() {
        process.current_dir(cwd);
    }
    for (key, value) in &plan.env {
        process.env(key, value);
    }

    match process.output() {
        Ok(output) if output.status.success() => Entry {
            id: plan.id,
            label: plan.label,
            source: plan.source,
            status: "succeeded".to_string(),
            command,
            env,
            items: plan.items,
            exit_code: output.status.code(),
            detail: first_output_line(&output),
        },
        Ok(output) => Entry {
            id: plan.id,
            label: plan.label,
            source: plan.source,
            status: "failed".to_string(),
            command,
            env,
            items: plan.items,
            exit_code: output.status.code(),
            detail: first_output_line(&output),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Entry {
            id: plan.id,
            label: plan.label,
            source: plan.source,
            status: "skipped".to_string(),
            command,
            env,
            items: plan.items,
            exit_code: None,
            detail: Some(format!("{} is not available", plan.program)),
        },
        Err(err) => Entry {
            id: plan.id,
            label: plan.label,
            source: plan.source,
            status: "failed".to_string(),
            command,
            env,
            items: plan.items,
            exit_code: None,
            detail: Some(err.to_string()),
        },
    }
}

fn command_for_display(plan: &Plan) -> Vec<String> {
    let mut command = vec![plan.program.clone()];
    command.extend(plan.args.iter().cloned());
    command
}

fn env_for_display(plan: &Plan) -> Vec<String> {
    plan.env
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect()
}

fn global_tool_update_cwd() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}

fn first_output_line(output: &Output) -> Option<String> {
    stdout_or_stderr_trimmed(output).and_then(|detail| {
        detail
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(ToOwned::to_owned)
    })
}

fn stdout_or_stderr_trimmed(output: &Output) -> Option<String> {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !stdout.is_empty() {
        return Some(stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        None
    } else {
        Some(stderr)
    }
}

pub(crate) fn format_human(result: &UpdateResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge tool update: {}",
        if result.dry_run {
            "dry run"
        } else {
            "completed"
        }
    );
    let _ = writeln!(
        out,
        "summary: {} planned, {} skipped, {} succeeded, {} failed",
        result.summary.planned,
        result.summary.skipped,
        result.summary.succeeded,
        result.summary.failed
    );
    for entry in &result.entries {
        let mut command = entry.command.join(" ");
        if !entry.env.is_empty() {
            command = format!("{} {}", entry.env.join(" "), command);
        }
        let _ = writeln!(
            out,
            "[{}] {}: {}",
            status_label(&entry.status),
            entry.id,
            command
        );
        if !entry.items.is_empty() {
            let _ = writeln!(out, "  items: {}", entry.items.join(", "));
        }
        if let Some(detail) = entry.detail.as_ref() {
            let _ = writeln!(out, "  detail: {detail}");
        }
    }
    out.trim_end().to_string()
}

fn status_label(status: &str) -> String {
    status.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_plans_use_homebrew_and_uv_installer_sources() {
        let packages =
            packages_plan_for("macos", "brew-test".to_string(), "winget-test".to_string());
        assert_eq!(packages.id, "packages");
        assert_eq!(packages.source, "homebrew");
        assert_eq!(packages.program, "brew-test");
        assert_eq!(packages.args, vec!["upgrade"]);

        let rustup = rustup_plan();
        assert_eq!(rustup.id, "rustup");
        assert_eq!(rustup.source, "rustup");
        assert_eq!(rustup.args, vec!["update"]);

        let uv = uv_install_plan_for("macos");
        assert_eq!(uv.source, "uv_standalone_installer");
        assert_eq!(uv.program, "sh");
        assert_eq!(
            uv.args,
            vec!["-c", "curl -LsSf https://astral.sh/uv/install.sh | sh"]
        );

        let gum = gum_install_plans_for(
            "macos",
            "brew-test".to_string(),
            "winget-test".to_string(),
            "go-test".to_string(),
        );
        assert_eq!(gum[0].source, "homebrew");
        assert_eq!(gum[0].args, vec!["install", "gum"]);
        assert_eq!(gum[1].source, "go_install");
        assert_eq!(gum[1].args, vec!["install", GUM_GO_PACKAGE]);
    }

    #[test]
    fn target_selection_has_stable_order_and_alias_dedupe() {
        let defaults = select_targets(&[]).expect("default targets");
        let ids = defaults
            .iter()
            .map(|target| target.id())
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

        let requested = vec![
            "brew".to_string(),
            "packages".to_string(),
            "rust".to_string(),
            "uv-self".to_string(),
            "uv-tools".to_string(),
            "cargo".to_string(),
            "cargo-installs".to_string(),
        ];
        let selected = select_targets(&requested).expect("selected targets");
        let ids = selected
            .iter()
            .map(|target| target.id())
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec!["packages", "rustup", "uv", "uv-tools", "cargo-installs"]
        );
    }

    #[test]
    fn windows_plans_use_winget_and_uv_installer_sources() {
        let packages = packages_plan_for(
            "windows",
            "brew-test".to_string(),
            "winget-test".to_string(),
        );
        assert_eq!(packages.id, "packages");
        assert_eq!(packages.source, "winget");
        assert_eq!(packages.program, "winget-test");
        assert_eq!(packages.args[0], "upgrade");
        assert!(packages.args.iter().any(|arg| arg == "--all"));

        let uv = uv_install_plan_for("windows");
        assert_eq!(uv.source, "uv_standalone_installer");
        assert_eq!(uv.program, "powershell");
        assert_eq!(
            uv.args,
            vec![
                "-ExecutionPolicy",
                "ByPass",
                "-c",
                "irm https://astral.sh/uv/install.ps1 | iex",
            ]
        );

        let gum = gum_install_plans_for(
            "windows",
            "brew-test".to_string(),
            "winget-test".to_string(),
            "go-test".to_string(),
        );
        assert_eq!(gum.len(), 1);
        assert_eq!(gum[0].source, "winget");
        assert_eq!(
            gum[0].args,
            vec![
                "install",
                "--id",
                "charmbracelet.gum",
                "-e",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ]
        );
    }
}
