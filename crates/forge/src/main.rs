use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    fmt::Write as _,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

const FORGE_REPO_SLUG: &str = "iancleary/forge";
const FORGE_REPO_URL: &str = "https://github.com/iancleary/forge";
const DEFAULT_FORGE_REPO_INSTALL_SUBPATH: &str = ".agents/skills-installed";
const REPO_SKILLS_SUBPATH: &str = ".agents/skills";
const REPO_CODEX_USER_SUBPATH: &str = "codex/user";
const CODEX_AGENTS_REL_PATH: &str = "AGENTS.md";
const CODEX_RULES_REL_PATH: &str = "rules/user-policy.rules";
const RELEASE_PACKAGES: &[&str] = &["forge", "codex-threads", "linear", "slack-cli"];

macro_rules! embedded_skill {
    ($name:literal) => {
        EmbeddedSkill {
            name: $name,
            skill_md: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../.agents/skills/",
                $name,
                "/SKILL.md"
            )),
        }
    };
}

macro_rules! embedded_codex_asset {
    ($name:literal, $relative_path:literal) => {
        EmbeddedCodexAsset {
            name: $name,
            relative_path: $relative_path,
            contents: include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../codex/user/",
                $relative_path
            )),
        }
    };
}

#[derive(Parser, Debug)]
#[command(name = "forge")]
#[command(about = "Forge manager CLI")]
#[command(after_help = "Output:\n  - Default output is human-readable.\n  - Use --json for compact (token-efficient) machine-readable JSON.\n  - Errors follow the same rule: human-readable by default, compact JSON with --json.")]
struct Cli {
    #[arg(
        long,
        global = true,
        help = "Emit compact (token-efficient) machine-readable JSON"
    )]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(about = "Check whether the local Forge environment is ready")]
    Doctor,
    #[command(
        name = "self",
        about = "Check for Forge updates and reconcile managed skills",
        subcommand
    )]
    Self_(SelfCommand),
    #[command(about = "Check or repair local Forge file permissions", subcommand)]
    Permissions(PermissionsCommand),
    #[command(
        about = "Install, validate, diff, and inspect Forge-managed Codex skills",
        subcommand
    )]
    Skills(SkillsCommand),
    #[command(
        about = "Render, diff, and install Forge-managed Codex user config",
        subcommand
    )]
    Codex(CodexCommand),
}

#[derive(Subcommand, Debug)]
enum SelfCommand {
    #[command(
        name = "update-check",
        about = "Check whether an update is available (cached by default)"
    )]
    UpdateCheck(UpdateCheckArgs),
    #[command(about = "Apply updates and reconcile managed installs")]
    Update(UpdateArgs),
}

#[derive(Subcommand, Debug)]
enum PermissionsCommand {
    #[command(about = "Inspect whether Forge-managed paths have expected permissions")]
    Check,
    #[command(about = "Repair permissions for Forge-managed paths")]
    Fix,
}

#[derive(Subcommand, Debug)]
enum SkillsCommand {
    #[command(about = "List available Forge skills from repo and/or release sources")]
    List(SkillsListArgs),
    #[command(about = "Show managed install status; defaults to mainline targets only")]
    Status(SkillsStatusArgs),
    #[command(about = "Validate SKILL.md metadata and router references")]
    Validate(SkillsValidateArgs),
    #[command(about = "Install Forge-managed skills to a target location")]
    Install(SkillsInstallArgs),
    #[command(about = "Diff one installed skill against the selected source")]
    Diff(SkillsDiffArgs),
    #[command(
        about = "Reinstall skills from the release source, switching back from repo-sourced testing"
    )]
    Revert(SkillsRevertArgs),
}

#[derive(Subcommand, Debug)]
enum CodexCommand {
    #[command(about = "Render Forge-managed Codex assets for the selected target")]
    Render(CodexRenderArgs),
    #[command(about = "Diff Forge-managed Codex assets against the selected target")]
    Diff(CodexDiffArgs),
    #[command(about = "Install Forge-managed Codex assets to the selected target")]
    Install(CodexInstallArgs),
}

#[derive(Args, Debug)]
struct UpdateCheckArgs {
    #[arg(long, help = "Force a fresh check instead of using cached state")]
    force: bool,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct UpdateArgs {
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
    #[arg(
        long,
        help = "Remote branch to update from; defaults to the remote default branch when known"
    )]
    branch: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum SkillsListSource {
    Repo,
    Release,
    All,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum SkillSourceArg {
    Repo,
    Release,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum SkillTargetRoleArg {
    Mainline,
    Development,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum SkillsStatusScope {
    Mainline,
    Development,
    All,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum CodexAssetArg {
    Agents,
    Rules,
}

#[derive(Args, Debug)]
struct SkillsListArgs {
    #[arg(long, value_enum, default_value = "all")]
    source: SkillsListSource,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct SkillsStatusArgs {
    #[arg(long, value_enum, default_value = "mainline")]
    scope: SkillsStatusScope,
    #[arg(
        long,
        help = "Filter to one target: user, forge_repo, or path:/absolute/path"
    )]
    target: Option<String>,
    #[arg(
        long,
        value_enum,
        help = "Optionally restrict the target filter to one role"
    )]
    target_role: Option<SkillTargetRoleArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct SkillsValidateArgs {
    #[arg(help = "Specific skill name to validate")]
    skill: Option<String>,
    #[arg(long, help = "Validate every available skill")]
    all: bool,
    #[arg(long, value_enum, help = "Force repo or release as the source")]
    source: Option<SkillSourceArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct SkillsInstallArgs {
    #[arg(help = "Specific skill to install")]
    skill: Option<String>,
    #[arg(long, help = "Install every available skill")]
    all: bool,
    #[arg(
        long,
        default_value = "user",
        help = "Target: user, forge_repo, or path:/absolute/path"
    )]
    target: String,
    #[arg(long, value_enum, help = "Use repo or release as the source")]
    source: Option<SkillSourceArg>,
    #[arg(long, value_enum, help = "Mark the install as mainline or development")]
    target_role: Option<SkillTargetRoleArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
    #[arg(long, help = "Overwrite an existing Forge-managed install")]
    force: bool,
    #[arg(
        long,
        help = "Take ownership of an unmanaged destination with the same skill name"
    )]
    force_unmanaged: bool,
}

#[derive(Args, Debug)]
struct SkillsDiffArgs {
    #[arg(help = "Skill name to diff")]
    skill: String,
    #[arg(
        long,
        default_value = "user",
        help = "Target: user, forge_repo, or path:/absolute/path"
    )]
    target: String,
    #[arg(long, value_enum, help = "Use repo or release as the source")]
    source: Option<SkillSourceArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct SkillsRevertArgs {
    #[arg(help = "Specific skill to revert")]
    skill: Option<String>,
    #[arg(long, help = "Revert every installed Forge-managed skill")]
    all: bool,
    #[arg(
        long,
        default_value = "user",
        help = "Target: user, forge_repo, or path:/absolute/path"
    )]
    target: String,
    #[arg(
        long,
        value_enum,
        help = "Mark the reverted install as mainline or development"
    )]
    target_role: Option<SkillTargetRoleArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
    #[arg(long, help = "Overwrite an existing Forge-managed install")]
    force: bool,
    #[arg(
        long,
        help = "Take ownership of an unmanaged destination with the same skill name"
    )]
    force_unmanaged: bool,
}

#[derive(Args, Debug)]
struct CodexRenderArgs {
    #[arg(
        long,
        value_enum,
        help = "Render one asset: agents or rules; repeat to select multiple assets (default: all)"
    )]
    asset: Vec<CodexAssetArg>,
    #[arg(long, default_value = "user", help = "Target: user or path:/absolute/path")]
    target: String,
    #[arg(long, value_enum, help = "Use repo or release as the source")]
    source: Option<SkillSourceArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct CodexDiffArgs {
    #[arg(
        long,
        value_enum,
        help = "Diff one asset: agents or rules; repeat to select multiple assets (default: all)"
    )]
    asset: Vec<CodexAssetArg>,
    #[arg(long, default_value = "user", help = "Target: user or path:/absolute/path")]
    target: String,
    #[arg(long, value_enum, help = "Use repo or release as the source")]
    source: Option<SkillSourceArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Args, Debug)]
struct CodexInstallArgs {
    #[arg(
        long,
        value_enum,
        help = "Install one asset: agents or rules; repeat to select multiple assets (default: all)"
    )]
    asset: Vec<CodexAssetArg>,
    #[arg(long, default_value = "user", help = "Target: user or path:/absolute/path")]
    target: String,
    #[arg(long, value_enum, help = "Use repo or release as the source")]
    source: Option<SkillSourceArg>,
    #[arg(long, help = "Override the forge repo path")]
    repo_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct Envelope<T>
where
    T: Serialize,
{
    ok: bool,
    data: T,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    ok: bool,
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

#[allow(dead_code)]
#[derive(Debug, Default, Deserialize)]
struct ForgeConfig {
    #[serde(default)]
    auto_check_updates: Option<bool>,
    #[serde(default)]
    auto_update: Option<bool>,
    #[serde(default)]
    update_check_ttl_minutes: Option<u64>,
    #[serde(default)]
    repo_path: Option<String>,
    #[serde(default)]
    forge_repo_install_subpath: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ForgeState {
    #[serde(default)]
    last_checked_unix: Option<u64>,
    #[serde(default)]
    repo_path: Option<String>,
    #[serde(default)]
    local_head: Option<String>,
    #[serde(default)]
    remote_head: Option<String>,
    #[serde(default)]
    remote_default_branch: Option<String>,
    #[serde(default)]
    update_available: Option<bool>,
    #[serde(default)]
    managed_skill_installs: Vec<ManagedSkillInstall>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SkillSourceKind {
    RepoCheckout,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SkillTargetKind {
    User,
    ForgeRepo,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum SkillTargetRole {
    Mainline,
    Development,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManagedSkillInstall {
    skill_name: String,
    managed_by: String,
    source_kind: SkillSourceKind,
    source_repo_slug: String,
    source_ref: String,
    source_hash: String,
    #[serde(default)]
    source_repo_path: Option<String>,
    target_kind: SkillTargetKind,
    target_role: SkillTargetRole,
    target_root: String,
    target_path: String,
    installed_at_unix: u64,
}

#[derive(Debug, Serialize)]
struct UpdateCheckResult {
    source_kind: String,
    repo_path: Option<String>,
    cached: bool,
    local_head: Option<String>,
    remote_head: Option<String>,
    remote_default_branch: Option<String>,
    current_version: Option<String>,
    latest_version: Option<String>,
    update_available: bool,
    checked_at_unix: u64,
    skills_out_of_date: bool,
    codex_out_of_date: bool,
    skills: Vec<SkillStatusEntry>,
}

#[derive(Debug, Serialize)]
struct UpdateResult {
    source_kind: String,
    repo_path: Option<String>,
    branch: Option<String>,
    before_head: Option<String>,
    after_head: Option<String>,
    before_version: Option<String>,
    after_version: Option<String>,
    changed: bool,
    skills_reconciled: usize,
    codex_reconciled: usize,
}

#[derive(Debug, Serialize)]
struct PermissionsResult {
    items: Vec<PermissionItem>,
}

#[derive(Debug, Serialize)]
struct PermissionItem {
    path: String,
    kind: String,
    exists: bool,
    expected_mode: String,
    actual_mode: Option<String>,
    ok: bool,
    changed: bool,
}

#[derive(Debug, Serialize)]
struct DoctorResult {
    summary: DoctorSummary,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
struct DoctorSummary {
    status: String,
    ready: bool,
    passed: usize,
    warnings: usize,
    failures: usize,
}

#[derive(Debug, Serialize)]
struct DoctorCheck {
    id: String,
    category: String,
    status: String,
    summary: String,
    detail: Option<String>,
    remediation: Vec<String>,
    upgrades: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct LinearDoctorConfig {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    token_file: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct SlackDoctorConfig {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    token_file: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsListResult {
    source: String,
    skills: Vec<SkillListEntry>,
}

#[derive(Debug, Serialize)]
struct SkillListEntry {
    name: String,
    source_kind: String,
    source_path: Option<String>,
    installed_targets: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkillsValidateResult {
    source_kind: String,
    valid: bool,
    skills: Vec<SkillValidationEntry>,
}

#[derive(Debug, Serialize)]
struct SkillValidationEntry {
    name: String,
    valid: bool,
    path: Option<String>,
    issues: Vec<String>,
}

#[derive(Debug, Serialize)]
struct SkillsInstallResult {
    source_kind: String,
    target_kind: String,
    target_role: String,
    target_root: String,
    installs: Vec<SkillInstallEntry>,
}

#[derive(Debug, Serialize)]
struct SkillInstallEntry {
    name: String,
    source_hash: String,
    target_path: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct SkillsDiffResult {
    name: String,
    source_kind: String,
    target_kind: String,
    target_path: String,
    identical: bool,
    files: Vec<SkillDiffFile>,
}

#[derive(Debug, Serialize)]
struct SkillDiffFile {
    path: String,
    status: String,
    source_hash: Option<String>,
    target_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct SkillsStatusResult {
    source_kind: String,
    scope: String,
    entries: Vec<SkillStatusEntry>,
}

#[derive(Debug, Serialize)]
struct CodexRenderResult {
    source_kind: String,
    target_kind: String,
    target_root: String,
    assets: Vec<CodexRenderEntry>,
}

#[derive(Debug, Serialize)]
struct CodexRenderEntry {
    name: String,
    relative_path: String,
    source_path: Option<String>,
    target_path: String,
    source_hash: String,
    contents: String,
}

#[derive(Debug, Serialize)]
struct CodexDiffResult {
    source_kind: String,
    target_kind: String,
    target_root: String,
    identical: bool,
    assets: Vec<CodexDiffEntry>,
}

#[derive(Debug, Serialize)]
struct CodexDiffEntry {
    name: String,
    relative_path: String,
    target_path: String,
    status: String,
    source_hash: String,
    target_hash: Option<String>,
}

#[derive(Debug, Serialize)]
struct CodexInstallResult {
    source_kind: String,
    target_kind: String,
    target_root: String,
    assets: Vec<CodexInstallEntry>,
}

#[derive(Debug, Serialize)]
struct CodexInstallEntry {
    name: String,
    relative_path: String,
    target_path: String,
    source_hash: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct SkillStatusEntry {
    name: String,
    target_kind: String,
    target_role: String,
    target_path: String,
    state: String,
    source_kind: String,
    source_hash: Option<String>,
    target_hash: Option<String>,
}

#[derive(Debug, Clone)]
struct SkillDefinition {
    name: String,
    source_kind: SkillSourceKind,
    source_path: Option<PathBuf>,
    source_ref: String,
    source_repo_path: Option<PathBuf>,
    files: BTreeMap<String, Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
struct EmbeddedSkill {
    name: &'static str,
    skill_md: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct EmbeddedCodexAsset {
    name: &'static str,
    relative_path: &'static str,
    contents: &'static str,
}

#[derive(Debug, Clone)]
struct ResolvedTarget {
    kind: SkillTargetKind,
    role: SkillTargetRole,
    root: PathBuf,
}

#[derive(Debug, Clone)]
struct TargetFilter {
    kind: SkillTargetKind,
    role: Option<SkillTargetRole>,
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CodexTargetKind {
    User,
    Path,
}

#[derive(Debug, Clone)]
struct CodexAssetDefinition {
    name: String,
    relative_path: String,
    source_path: Option<PathBuf>,
    contents: Vec<u8>,
}

#[derive(Debug, Clone)]
struct ResolvedCodexTarget {
    kind: CodexTargetKind,
    root: PathBuf,
}

fn main() {
    let args = env::args_os().collect::<Vec<_>>();
    let wants_json = args.iter().any(|arg| arg.to_str() == Some("--json"));
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            let exit_code = err.exit_code();
            match err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    let _ = err.print();
                }
                _ if wants_json => {
                    let error = ErrorEnvelope {
                        ok: false,
                        error: ErrorBody {
                            code: "invalid_usage".to_string(),
                            message: err.to_string(),
                        },
                    };
                    eprintln!(
                        "{}",
                        serde_json::to_string(&error).unwrap_or_else(|_| {
                            "{\"ok\":false,\"error\":{\"code\":\"internal_error\",\"message\":\"failed to serialize error\"}}".to_string()
                        })
                    );
                }
                _ => {
                    let _ = err.print();
                }
            }
            std::process::exit(exit_code);
        }
    };
    let output = OutputMode::from_json_flag(cli.json);
    let result = run(cli);

    if let Err(err) = result {
        let error = ErrorEnvelope {
            ok: false,
            error: classify_error(&err),
        };
        match output {
            OutputMode::Json => {
                eprintln!(
                    "{}",
                    serde_json::to_string(&error).unwrap_or_else(|_| {
                        "{\"ok\":false,\"error\":{\"code\":\"internal_error\",\"message\":\"failed to serialize error\"}}".to_string()
                    })
                );
            }
            OutputMode::Human => eprintln!("{}", format_error_human(&error.error)),
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let output = OutputMode::from_json_flag(cli.json);

    match cli.command {
        Command::Doctor => {
            let data = doctor()?;
            emit_output(output, data, format_doctor_human)?;
        }
        Command::Self_(SelfCommand::UpdateCheck(args)) => {
            let data = update_check(args)?;
            emit_output(output, data, format_update_check_human)?;
        }
        Command::Self_(SelfCommand::Update(args)) => {
            let data = update(args)?;
            emit_output(output, data, format_update_human)?;
        }
        Command::Permissions(PermissionsCommand::Check) => {
            let data = inspect_permissions(false)?;
            emit_output(output, data, |data| format_permissions_human("check", data))?;
        }
        Command::Permissions(PermissionsCommand::Fix) => {
            let data = inspect_permissions(true)?;
            emit_output(output, data, |data| format_permissions_human("fix", data))?;
        }
        Command::Skills(SkillsCommand::List(args)) => {
            let data = skills_list(args)?;
            emit_output(output, data, format_skills_list_human)?;
        }
        Command::Skills(SkillsCommand::Status(args)) => {
            let data = skills_status(args)?;
            emit_output(output, data, format_skills_status_human)?;
        }
        Command::Skills(SkillsCommand::Validate(args)) => {
            let data = skills_validate(args)?;
            emit_output(output, data, format_skills_validate_human)?;
        }
        Command::Skills(SkillsCommand::Install(args)) => {
            let data = skills_install(args)?;
            emit_output(output, data, |data| format_skills_install_human("install", data))?;
        }
        Command::Skills(SkillsCommand::Diff(args)) => {
            let data = skills_diff(args)?;
            emit_output(output, data, format_skills_diff_human)?;
        }
        Command::Skills(SkillsCommand::Revert(args)) => {
            let data = skills_revert(args)?;
            emit_output(output, data, |data| format_skills_install_human("revert", data))?;
        }
        Command::Codex(CodexCommand::Render(args)) => {
            let data = codex_render(args)?;
            emit_output(output, data, format_codex_render_human)?;
        }
        Command::Codex(CodexCommand::Diff(args)) => {
            let data = codex_diff(args)?;
            emit_output(output, data, format_codex_diff_human)?;
        }
        Command::Codex(CodexCommand::Install(args)) => {
            let data = codex_install(args)?;
            emit_output(output, data, format_codex_install_human)?;
        }
    }

    Ok(())
}

fn doctor() -> Result<DoctorResult> {
    let config_dir = config_dir_path()?;
    let checks = vec![
        doctor_command_check("cargo", "tool", "cargo", &["--version"]),
        doctor_command_check("git", "tool", "git", &["--version"]),
        doctor_command_check("gh", "tool", "gh", &["--version"]),
        doctor_command_check("rg", "tool", "rg", &["--version"]),
        doctor_command_check("jq", "tool", "jq", &["--version"]),
        doctor_gh_auth_check(),
        doctor_linear_auth_check(),
        doctor_slack_auth_check(),
        doctor_config_dir_check(&config_dir),
    ];

    let passed = checks.iter().filter(|check| check.status == "pass").count();
    let warnings = checks.iter().filter(|check| check.status == "warn").count();
    let failures = checks.iter().filter(|check| check.status == "fail").count();
    let ready = failures == 0 && warnings == 0;
    let status = if failures > 0 {
        "fail"
    } else if warnings > 0 {
        "warn"
    } else {
        "pass"
    };

    Ok(DoctorResult {
        summary: DoctorSummary {
            status: status.to_string(),
            ready,
            passed,
            warnings,
            failures,
        },
        checks,
    })
}

fn update_check(args: UpdateCheckArgs) -> Result<UpdateCheckResult> {
    let config = load_config()?;
    let checked_at_unix = now_unix()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = if repo_path.is_some() {
        SkillSourceKind::RepoCheckout
    } else {
        SkillSourceKind::Release
    };
    let mut current_version = None;
    let mut latest_version = None;

    let (local_head, remote_head, remote_default_branch, update_available) =
        if let Some(path) = repo_path.as_ref() {
            ensure_git_repo(path)?;
            let local_head = git_stdout(path, &["rev-parse", "HEAD"])?;
            let (remote_default_branch, remote_head) = remote_default_branch_and_head(path)?;
            let update_available = remote_head
                .as_ref()
                .is_some_and(|remote| remote != &local_head);
            (
                Some(local_head),
                remote_head,
                remote_default_branch,
                update_available,
            )
        } else {
            current_version = Some(env!("CARGO_PKG_VERSION").to_string());
            latest_version = latest_release_version()?;
            let update_available = latest_version
                .as_ref()
                .zip(current_version.as_ref())
                .is_some_and(|(latest, current)| latest != current);
            (
                current_version.clone(),
                latest_version.clone(),
                None,
                update_available,
            )
        };

    let mut state = load_state_or_default()?;
    state.last_checked_unix = Some(checked_at_unix);
    state.repo_path = repo_path.as_ref().map(|path| path.display().to_string());
    state.local_head = local_head.clone();
    state.remote_head = remote_head.clone();
    state.remote_default_branch = remote_default_branch.clone();
    state.update_available = Some(update_available);
    save_state(&state_file_path()?, &state)?;

    let status = skills_status_with_source(
        &config,
        &state,
        source_kind.clone(),
        repo_path.clone(),
        SkillsStatusScope::Mainline,
        None,
    )?;
    let codex_status = codex_diff(CodexDiffArgs {
        asset: Vec::new(),
        target: "user".to_string(),
        source: Some(match source_kind {
            SkillSourceKind::RepoCheckout => SkillSourceArg::Repo,
            SkillSourceKind::Release => SkillSourceArg::Release,
        }),
        repo_path: repo_path.clone(),
    })?;
    let _ = args.force;

    Ok(UpdateCheckResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        repo_path: repo_path.map(|path| path.display().to_string()),
        cached: false,
        local_head,
        remote_head,
        remote_default_branch,
        current_version,
        latest_version,
        update_available,
        checked_at_unix,
        skills_out_of_date: status
            .entries
            .iter()
            .any(|entry| entry.state == "out_of_date" || entry.state == "missing"),
        codex_out_of_date: !codex_status.identical,
        skills: status.entries,
    })
}

fn update(args: UpdateArgs) -> Result<UpdateResult> {
    let config = load_config()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = if repo_path.is_some() {
        SkillSourceKind::RepoCheckout
    } else {
        SkillSourceKind::Release
    };

    let mut before_head = None;
    let mut after_head = None;
    let mut branch = None;
    let mut before_version = None;
    let mut after_version = None;

    let mut state = load_state_or_default()?;
    // Before attempting updates/installs, detect unmanaged collisions and provide a single actionable error.
    // This keeps the default safety posture (no implicit takeover) while making remediation obvious.
    let status = skills_status_with_source(
        &config,
        &state,
        source_kind.clone(),
        repo_path.clone(),
        SkillsStatusScope::Mainline,
        None,
    )?;
    let collisions = status
        .entries
        .iter()
        .filter(|entry| entry.state == "unmanaged_collision")
        .collect::<Vec<_>>();
    if !collisions.is_empty() {
        let source_flag = match source_kind {
            SkillSourceKind::RepoCheckout => "repo",
            SkillSourceKind::Release => "release",
        };
        let mut lines = Vec::new();
        for entry in &collisions {
            lines.push(format!("- {}: {}", entry.name, entry.target_path));
        }
        let list = lines.join("\n");
        bail!(
            "unmanaged collisions detected for {} skills:\n{}\n\nTo take ownership once:\nforge skills install --all --force-unmanaged --source {}\n\nThen:\nforge codex diff\nforge codex install\nforge self update-check\nforge self update",
            collisions.len(),
            list,
            source_flag
        );
    }

    if let Some(path) = repo_path.as_ref() {
        ensure_git_repo(path)?;
        before_head = Some(git_stdout(path, &["rev-parse", "HEAD"])?);
        branch = Some(match args.branch {
            Some(branch) => branch,
            None => remote_default_branch_and_head(path)?
                .0
                .unwrap_or_else(|| "main".to_string()),
        });
        run_git(
            path,
            &[
                "pull",
                "--rebase",
                "origin",
                branch.as_deref().unwrap_or("main"),
            ],
        )?;
        after_head = Some(git_stdout(path, &["rev-parse", "HEAD"])?);
    } else {
        if args.branch.is_some() {
            bail!("--branch is only supported in repo-checkout mode");
        }
        before_version = Some(env!("CARGO_PKG_VERSION").to_string());
        let target_version = latest_release_version()?
            .ok_or_else(|| anyhow!("failed to determine the latest Forge release tag"))?;
        if before_version.as_deref() != Some(target_version.as_str()) {
            install_release_packages(&target_version)?;
            after_version = Some(target_version);
        } else {
            after_version = before_version.clone();
        }
    }

    let targets = mainline_targets_for_reconcile(&config, &state, repo_path.as_deref())?;
    let mut installs = Vec::new();
    for target in targets {
        let result = skills_install_internal(
            &config,
            &mut state,
            InstallRequest {
                skill_names: Vec::new(),
                all: true,
                source_kind: Some(source_kind.clone()),
                repo_path: repo_path.clone(),
                target: None,
                target_role: None,
                resolved_target: Some(target),
                force: true,
                force_unmanaged: false,
                restrict_to_targets: None,
            },
        )?;
        installs.extend(result.installs);
    }
    let codex_result = codex_install(CodexInstallArgs {
        asset: Vec::new(),
        target: "user".to_string(),
        source: Some(match source_kind {
            SkillSourceKind::RepoCheckout => SkillSourceArg::Repo,
            SkillSourceKind::Release => SkillSourceArg::Release,
        }),
        repo_path: repo_path.clone(),
    })?;
    save_state(&state_file_path()?, &state)?;

    let changed = before_head != after_head
        || before_version != after_version
        || installs.iter().any(|item| item.status != "unchanged")
        || codex_result
            .assets
            .iter()
            .any(|item| item.status != "unchanged");

    Ok(UpdateResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        repo_path: repo_path.map(|path| path.display().to_string()),
        branch,
        before_head,
        after_head,
        before_version,
        after_version,
        changed,
        skills_reconciled: installs.len(),
        codex_reconciled: codex_result.assets.len(),
    })
}

fn inspect_permissions(apply_fixes: bool) -> Result<PermissionsResult> {
    let items = managed_permission_targets()?
        .into_iter()
        .map(|target| inspect_permission_target(&target, apply_fixes))
        .collect::<Result<Vec<_>>>()?;
    Ok(PermissionsResult { items })
}

fn skills_list(args: SkillsListArgs) -> Result<SkillsListResult> {
    let config = load_config()?;
    let state = load_state_or_default()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let mut skills = Vec::new();

    match args.source {
        SkillsListSource::Repo | SkillsListSource::All => {
            if let Some(path) = repo_path.clone() {
                for def in load_repo_skills(&path)? {
                    skills.push(skill_list_entry(def, &state));
                }
            }
        }
        _ => {}
    }

    match args.source {
        SkillsListSource::Release | SkillsListSource::All => {
            for def in load_release_skills() {
                skills.push(skill_list_entry(def, &state));
            }
        }
        _ => {}
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name).then(a.source_kind.cmp(&b.source_kind)));

    Ok(SkillsListResult {
        source: match args.source {
            SkillsListSource::Repo => "repo".to_string(),
            SkillsListSource::Release => "release".to_string(),
            SkillsListSource::All => "all".to_string(),
        },
        skills,
    })
}

fn skill_list_entry(def: SkillDefinition, state: &ForgeState) -> SkillListEntry {
    let installed_targets = state
        .managed_skill_installs
        .iter()
        .filter(|entry| entry.skill_name == def.name)
        .map(|entry| entry.target_path.clone())
        .collect::<Vec<_>>();

    SkillListEntry {
        name: def.name,
        source_kind: source_kind_name(&def.source_kind).to_string(),
        source_path: def.source_path.map(|path| path.display().to_string()),
        installed_targets,
    }
}

fn skills_validate(args: SkillsValidateArgs) -> Result<SkillsValidateResult> {
    if args.skill.is_none() && !args.all {
        bail!("provide a skill name or --all");
    }

    let config = load_config()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = resolve_source_kind(args.source, repo_path.as_deref())?;
    let skills = load_skills_for_source(&source_kind, repo_path.as_deref())?;
    let selected = select_skill_defs(skills, args.skill.as_deref(), args.all)?;
    let known_names = selected
        .iter()
        .map(|def| def.name.clone())
        .collect::<BTreeSet<_>>();

    let mut results = Vec::new();
    for def in selected {
        let mut issues = Vec::new();
        let skill_md = def
            .files
            .get("SKILL.md")
            .ok_or_else(|| anyhow!("skill {} is missing SKILL.md", def.name))?;
        let body = String::from_utf8(skill_md.clone())
            .with_context(|| format!("skill {} SKILL.md was not UTF-8", def.name))?;
        let metadata = parse_skill_frontmatter(&body)
            .with_context(|| format!("skill {} frontmatter invalid", def.name))?;
        if metadata.name.is_empty() {
            issues.push("frontmatter field `name` is required".to_string());
        }
        if metadata.description.is_empty() {
            issues.push("frontmatter field `description` is required".to_string());
        }
        if def.name == "forge-tools" {
            for required in [
                "design-algorithm",
                "linear-cli",
                "slack-cli-research",
                "codex-threads-cli",
                "forge-cli-admin",
            ] {
                if !body.contains(required) {
                    issues.push(format!("router skill should reference `{required}`"));
                }
                if !known_names.contains(required) {
                    issues.push(format!(
                        "referenced skill `{required}` is not available from the selected source"
                    ));
                }
            }
        }

        results.push(SkillValidationEntry {
            name: def.name.clone(),
            valid: issues.is_empty(),
            path: def.source_path.map(|path| path.display().to_string()),
            issues,
        });
    }

    let valid = results.iter().all(|entry| entry.valid);
    Ok(SkillsValidateResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        valid,
        skills: results,
    })
}

fn skills_install(args: SkillsInstallArgs) -> Result<SkillsInstallResult> {
    let config = load_config()?;
    let mut state = load_state_or_default()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let result = skills_install_internal(
        &config,
        &mut state,
        InstallRequest {
            skill_names: args.skill.into_iter().collect(),
            all: args.all,
            source_kind: args.source.map(map_cli_source),
            repo_path,
            target: Some(args.target),
            target_role: args.target_role.map(map_target_role),
            resolved_target: None,
            force: args.force,
            force_unmanaged: args.force_unmanaged,
            restrict_to_targets: None,
        },
    )?;
    save_state(&state_file_path()?, &state)?;
    Ok(result)
}

fn skills_diff(args: SkillsDiffArgs) -> Result<SkillsDiffResult> {
    let config = load_config()?;
    let state = load_state_or_default()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = resolve_source_kind(args.source, repo_path.as_deref())?;
    let def = load_skill_definition(&source_kind, repo_path.as_deref(), &args.skill)?;
    let target = resolve_target(Some(&args.target), &config, repo_path.as_deref(), None)?;
    let target_path = target.root.join(&args.skill);
    let target_files = if target_path.exists() {
        load_skill_files_from_dir(&target_path)?
    } else {
        BTreeMap::new()
    };
    let files = build_diff_files(&def.files, &target_files);
    let identical = files.iter().all(|entry| entry.status == "same");
    let _ = state;

    Ok(SkillsDiffResult {
        name: args.skill,
        source_kind: source_kind_name(&source_kind).to_string(),
        target_kind: target_kind_name(&target.kind).to_string(),
        target_path: target_path.display().to_string(),
        identical,
        files,
    })
}

fn skills_revert(args: SkillsRevertArgs) -> Result<SkillsInstallResult> {
    let config = load_config()?;
    let mut state = load_state_or_default()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let result = skills_install_internal(
        &config,
        &mut state,
        InstallRequest {
            skill_names: args.skill.into_iter().collect(),
            all: args.all,
            source_kind: Some(SkillSourceKind::Release),
            repo_path,
            target: Some(args.target),
            target_role: args.target_role.map(map_target_role),
            resolved_target: None,
            force: args.force,
            force_unmanaged: args.force_unmanaged,
            restrict_to_targets: None,
        },
    )?;
    save_state(&state_file_path()?, &state)?;
    Ok(result)
}

fn codex_render(args: CodexRenderArgs) -> Result<CodexRenderResult> {
    let config = load_config()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = resolve_source_kind(args.source, repo_path.as_deref())?;
    let target = resolve_codex_target(Some(&args.target))?;
    let assets = select_codex_assets(
        load_codex_assets_for_source(&source_kind, repo_path.as_deref())?,
        &args.asset,
    )?;

    let assets = assets
        .into_iter()
        .map(|asset| {
            let contents = String::from_utf8(asset.contents.clone())
                .with_context(|| format!("codex asset {} was not UTF-8", asset.name))?;
            Ok(CodexRenderEntry {
                name: asset.name,
                relative_path: asset.relative_path.clone(),
                source_path: asset.source_path.map(|path| path.display().to_string()),
                target_path: target.root.join(&asset.relative_path).display().to_string(),
                source_hash: hash_bytes(&asset.contents),
                contents,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(CodexRenderResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        target_kind: codex_target_kind_name(&target.kind).to_string(),
        target_root: target.root.display().to_string(),
        assets,
    })
}

fn codex_diff(args: CodexDiffArgs) -> Result<CodexDiffResult> {
    let config = load_config()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = resolve_source_kind(args.source, repo_path.as_deref())?;
    let target = resolve_codex_target(Some(&args.target))?;
    let assets = select_codex_assets(
        load_codex_assets_for_source(&source_kind, repo_path.as_deref())?,
        &args.asset,
    )?;

    let assets = assets
        .into_iter()
        .map(|asset| {
            let target_path = target.root.join(&asset.relative_path);
            let target_contents = if target_path.exists() {
                Some(
                    fs::read(&target_path)
                        .with_context(|| format!("failed to read {}", target_path.display()))?,
                )
            } else {
                None
            };
            let status = match target_contents.as_ref() {
                Some(existing) if existing == &asset.contents => "same",
                Some(_) => "changed",
                None => "missing",
            };
            Ok(CodexDiffEntry {
                name: asset.name,
                relative_path: asset.relative_path,
                target_path: target_path.display().to_string(),
                status: status.to_string(),
                source_hash: hash_bytes(&asset.contents),
                target_hash: target_contents.as_ref().map(hash_bytes),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let identical = assets.iter().all(|asset| asset.status == "same");
    Ok(CodexDiffResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        target_kind: codex_target_kind_name(&target.kind).to_string(),
        target_root: target.root.display().to_string(),
        identical,
        assets,
    })
}

fn codex_install(args: CodexInstallArgs) -> Result<CodexInstallResult> {
    let config = load_config()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = resolve_source_kind(args.source, repo_path.as_deref())?;
    let target = resolve_codex_target(Some(&args.target))?;
    let assets = select_codex_assets(
        load_codex_assets_for_source(&source_kind, repo_path.as_deref())?,
        &args.asset,
    )?;

    fs::create_dir_all(&target.root)
        .with_context(|| format!("failed to create {}", target.root.display()))?;

    let mut entries = Vec::new();
    for asset in assets {
        let target_path = target.root.join(&asset.relative_path);
        let status = if target_path.exists() {
            let existing = fs::read(&target_path)
                .with_context(|| format!("failed to read {}", target_path.display()))?;
            if existing == asset.contents {
                "unchanged"
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)
                        .with_context(|| format!("failed to create {}", parent.display()))?;
                }
                fs::write(&target_path, &asset.contents)
                    .with_context(|| format!("failed to write {}", target_path.display()))?;
                "updated"
            }
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            fs::write(&target_path, &asset.contents)
                .with_context(|| format!("failed to write {}", target_path.display()))?;
            "installed"
        };

        entries.push(CodexInstallEntry {
            name: asset.name,
            relative_path: asset.relative_path,
            target_path: target_path.display().to_string(),
            source_hash: hash_bytes(&asset.contents),
            status: status.to_string(),
        });
    }

    Ok(CodexInstallResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        target_kind: codex_target_kind_name(&target.kind).to_string(),
        target_root: target.root.display().to_string(),
        assets: entries,
    })
}

#[derive(Debug)]
struct InstallRequest {
    skill_names: Vec<String>,
    all: bool,
    source_kind: Option<SkillSourceKind>,
    repo_path: Option<PathBuf>,
    target: Option<String>,
    target_role: Option<SkillTargetRole>,
    resolved_target: Option<ResolvedTarget>,
    force: bool,
    force_unmanaged: bool,
    restrict_to_targets: Option<Vec<String>>,
}

fn skills_install_internal(
    config: &ForgeConfig,
    state: &mut ForgeState,
    req: InstallRequest,
) -> Result<SkillsInstallResult> {
    let source_kind = match req.source_kind {
        Some(kind) => kind,
        None => auto_source_kind(req.repo_path.as_deref()),
    };
    let target = if let Some(target) = req.resolved_target {
        target
    } else {
        resolve_target(
            req.target.as_deref(),
            config,
            req.repo_path.as_deref(),
            req.target_role.clone(),
        )?
    };
    fs::create_dir_all(&target.root)
        .with_context(|| format!("failed to create {}", target.root.display()))?;

    let defs = load_skills_for_source(&source_kind, req.repo_path.as_deref())?;
    let selected = if req.all {
        defs
    } else if let Some(restrict) = req.restrict_to_targets.as_ref() {
        let allowed_target = target_to_flag(&target.kind, &target.role, &target.root);
        if !restrict.contains(&allowed_target) {
            Vec::new()
        } else if req.skill_names.is_empty() {
            let installed_names = state
                .managed_skill_installs
                .iter()
                .filter(|entry| entry.target_root == target.root.display().to_string())
                .map(|entry| entry.skill_name.clone())
                .collect::<BTreeSet<_>>();
            defs.into_iter()
                .filter(|def| installed_names.contains(&def.name))
                .collect::<Vec<_>>()
        } else {
            select_skill_defs(defs, req.skill_names.first().map(String::as_str), false)?
        }
    } else {
        if req.skill_names.is_empty() {
            bail!("provide a skill name or --all");
        }
        let wanted = req
            .skill_names
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        defs.into_iter()
            .filter(|def| wanted.contains(&def.name.as_str()))
            .collect::<Vec<_>>()
    };

    if selected.is_empty() {
        return Ok(SkillsInstallResult {
            source_kind: source_kind_name(&source_kind).to_string(),
            target_kind: target_kind_name(&target.kind).to_string(),
            target_role: target_role_name(&target.role).to_string(),
            target_root: target.root.display().to_string(),
            installs: Vec::new(),
        });
    }

    let installed_at_unix = now_unix()?;
    let mut installs = Vec::new();
    for def in selected {
        let target_path = target.root.join(&def.name);
        let source_hash = hash_skill_files(&def.files);
        let managed = state.managed_skill_installs.iter().any(|entry| {
            entry.skill_name == def.name && entry.target_path == target_path.display().to_string()
        });

        if target_path.exists() {
            if !managed && !req.force_unmanaged {
                bail!(
                    "destination already exists for unmanaged skill {}: {}",
                    def.name,
                    target_path.display()
                );
            }
            let existing_files = load_skill_files_from_dir(&target_path)?;
            let existing_hash = hash_skill_files(&existing_files);
            if managed && existing_hash == source_hash && !req.force {
                let entry = ManagedSkillInstall {
                    skill_name: def.name.clone(),
                    managed_by: "forge".to_string(),
                    source_kind: def.source_kind.clone(),
                    source_repo_slug: FORGE_REPO_SLUG.to_string(),
                    source_ref: def.source_ref.clone(),
                    source_hash: source_hash.clone(),
                    source_repo_path: def
                        .source_repo_path
                        .as_ref()
                        .map(|path| path.display().to_string()),
                    target_kind: target.kind.clone(),
                    target_role: target.role.clone(),
                    target_root: target.root.display().to_string(),
                    target_path: target_path.display().to_string(),
                    installed_at_unix,
                };
                upsert_managed_install(state, entry);
                installs.push(SkillInstallEntry {
                    name: def.name,
                    source_hash,
                    target_path: target_path.display().to_string(),
                    status: "unchanged".to_string(),
                });
                continue;
            }
            if managed || req.force_unmanaged || req.force {
                fs::remove_dir_all(&target_path)
                    .with_context(|| format!("failed to replace {}", target_path.display()))?;
            }
        }
        write_skill_definition(&target_path, &def)?;

        let entry = ManagedSkillInstall {
            skill_name: def.name.clone(),
            managed_by: "forge".to_string(),
            source_kind: def.source_kind.clone(),
            source_repo_slug: FORGE_REPO_SLUG.to_string(),
            source_ref: def.source_ref.clone(),
            source_hash: source_hash.clone(),
            source_repo_path: def
                .source_repo_path
                .as_ref()
                .map(|path| path.display().to_string()),
            target_kind: target.kind.clone(),
            target_role: target.role.clone(),
            target_root: target.root.display().to_string(),
            target_path: target_path.display().to_string(),
            installed_at_unix,
        };
        upsert_managed_install(state, entry);
        installs.push(SkillInstallEntry {
            name: def.name,
            source_hash,
            target_path: target_path.display().to_string(),
            status: "installed".to_string(),
        });
    }

    Ok(SkillsInstallResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        target_kind: target_kind_name(&target.kind).to_string(),
        target_role: target_role_name(&target.role).to_string(),
        target_root: target.root.display().to_string(),
        installs,
    })
}

fn managed_permission_targets() -> Result<Vec<PermissionTarget>> {
    let forge_dir = config_dir_path()?;
    Ok(vec![
        PermissionTarget::dir(forge_dir.clone(), 0o700),
        PermissionTarget::file(forge_dir.join("config.toml"), 0o600),
        PermissionTarget::file(forge_dir.join("state.toml"), 0o600),
        PermissionTarget::dir(forge_dir.join("slack-cli"), 0o700),
        PermissionTarget::file(forge_dir.join("slack-cli").join("config.toml"), 0o600),
        PermissionTarget::file(forge_dir.join("slack-cli").join("token"), 0o600),
        PermissionTarget::dir(forge_dir.join("linear"), 0o700),
        PermissionTarget::file(forge_dir.join("linear").join("token"), 0o600),
        PermissionTarget::file(forge_dir.join("linear").join("config.toml"), 0o600),
    ])
}

#[derive(Debug)]
struct PermissionTarget {
    path: PathBuf,
    kind: PermissionKind,
    expected_mode: u32,
}

impl PermissionTarget {
    fn dir(path: PathBuf, expected_mode: u32) -> Self {
        Self {
            path,
            kind: PermissionKind::Dir,
            expected_mode,
        }
    }

    fn file(path: PathBuf, expected_mode: u32) -> Self {
        Self {
            path,
            kind: PermissionKind::File,
            expected_mode,
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum PermissionKind {
    Dir,
    File,
}

fn inspect_permission_target(
    target: &PermissionTarget,
    apply_fixes: bool,
) -> Result<PermissionItem> {
    let exists = target.path.exists();
    let expected_mode = format_mode(target.expected_mode);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if !exists {
            return Ok(PermissionItem {
                path: target.path.display().to_string(),
                kind: permission_kind_name(target.kind).to_string(),
                exists: false,
                expected_mode,
                actual_mode: None,
                ok: true,
                changed: false,
            });
        }

        let metadata = fs::metadata(&target.path)
            .with_context(|| format!("failed to read metadata for {}", target.path.display()))?;
        let actual_mode_bits = metadata.permissions().mode() & 0o777;
        let mut changed = false;

        if apply_fixes && actual_mode_bits != target.expected_mode {
            let permissions = PermissionsExt::from_mode(target.expected_mode);
            fs::set_permissions(&target.path, permissions).with_context(|| {
                format!("failed to set permissions on {}", target.path.display())
            })?;
            changed = true;
        }

        let final_mode_bits = if changed {
            target.expected_mode
        } else {
            actual_mode_bits
        };

        return Ok(PermissionItem {
            path: target.path.display().to_string(),
            kind: permission_kind_name(target.kind).to_string(),
            exists: true,
            expected_mode,
            actual_mode: Some(format_mode(final_mode_bits)),
            ok: final_mode_bits == target.expected_mode,
            changed,
        });
    }

    #[cfg(not(unix))]
    {
        let _ = apply_fixes;
        let _ = target;
        Ok(PermissionItem {
            path: String::new(),
            kind: String::new(),
            exists,
            expected_mode,
            actual_mode: None,
            ok: true,
            changed: false,
        })
    }
}

fn permission_kind_name(kind: PermissionKind) -> &'static str {
    match kind {
        PermissionKind::Dir => "dir",
        PermissionKind::File => "file",
    }
}

fn doctor_command_check(
    id: &str,
    category: &str,
    program: &str,
    version_args: &[&str],
) -> DoctorCheck {
    let remediation = tool_remediation(program);
    match run_command_capture(program, version_args) {
        Ok(output) if output.status.success() => {
            let detail = stdout_or_stderr_trimmed(&output);
            DoctorCheck {
                id: id.to_string(),
                category: category.to_string(),
                status: "pass".to_string(),
                summary: format!("{program} is available"),
                detail,
                remediation: Vec::new(),
                upgrades: tool_upgrade_commands(program),
            }
        }
        Ok(output) => DoctorCheck {
            id: id.to_string(),
            category: category.to_string(),
            status: "fail".to_string(),
            summary: format!("{program} is installed but not working"),
            detail: output_failure_detail(&output),
            remediation,
            upgrades: tool_upgrade_commands(program),
        },
        Err(err) => DoctorCheck {
            id: id.to_string(),
            category: category.to_string(),
            status: "fail".to_string(),
            summary: format!("{program} is not installed"),
            detail: Some(err.to_string()),
            remediation,
            upgrades: Vec::new(),
        },
    }
}

fn doctor_gh_auth_check() -> DoctorCheck {
    let remediation = gh_auth_remediation();
    match run_command_capture("gh", &["auth", "status"]) {
        Ok(output) if output.status.success() => DoctorCheck {
            id: "gh_auth".to_string(),
            category: "auth".to_string(),
            status: "pass".to_string(),
            summary: "GitHub CLI authentication is ready".to_string(),
            detail: stdout_or_stderr_trimmed(&output),
            remediation: Vec::new(),
            upgrades: Vec::new(),
        },
        Ok(output) => DoctorCheck {
            id: "gh_auth".to_string(),
            category: "auth".to_string(),
            status: "warn".to_string(),
            summary: "GitHub CLI auth could not be confirmed in this non-interactive context"
                .to_string(),
            detail: output_failure_detail(&output),
            remediation,
            upgrades: Vec::new(),
        },
        Err(err) => DoctorCheck {
            id: "gh_auth".to_string(),
            category: "auth".to_string(),
            status: "warn".to_string(),
            summary:
                "GitHub CLI authentication could not be checked in this non-interactive context"
                    .to_string(),
            detail: Some(err.to_string()),
            remediation,
            upgrades: Vec::new(),
        },
    }
}

fn doctor_config_dir_check(config_dir: &Path) -> DoctorCheck {
    if config_dir.exists() {
        return DoctorCheck {
            id: "forge_config_dir".to_string(),
            category: "config".to_string(),
            status: "pass".to_string(),
            summary: "Forge config directory is present".to_string(),
            detail: Some(config_dir.display().to_string()),
            remediation: Vec::new(),
            upgrades: Vec::new(),
        };
    }

    DoctorCheck {
        id: "forge_config_dir".to_string(),
        category: "config".to_string(),
        status: "warn".to_string(),
        summary: "Forge config directory has not been created yet".to_string(),
        detail: Some(config_dir.display().to_string()),
        remediation: vec![
            format!("mkdir -p {}", shell_escape_path(config_dir)),
            "Run a Forge command that writes config or state, such as `forge self update-check --force --json`.".to_string(),
        ],
        upgrades: Vec::new(),
    }
}

fn doctor_linear_auth_check() -> DoctorCheck {
    let sources = linear_auth_sources();
    if sources.is_empty() {
        return DoctorCheck {
            id: "linear_auth".to_string(),
            category: "auth".to_string(),
            status: "warn".to_string(),
            summary: "Linear auth token source is not configured".to_string(),
            detail: Some("No Linear token source was found.".to_string()),
            remediation: linear_auth_remediation(),
            upgrades: Vec::new(),
        };
    }

    DoctorCheck {
        id: "linear_auth".to_string(),
        category: "auth".to_string(),
        status: "pass".to_string(),
        summary: "Linear auth token source is configured".to_string(),
        detail: Some(describe_auth_sources(&sources)),
        remediation: vec![
            "To validate the token, run `linear --json viewer` in an interactive terminal."
                .to_string(),
        ],
        upgrades: Vec::new(),
    }
}

fn doctor_slack_auth_check() -> DoctorCheck {
    let sources = slack_auth_sources();

    if sources.is_empty() {
        return DoctorCheck {
            id: "slack_auth".to_string(),
            category: "auth".to_string(),
            status: "warn".to_string(),
            summary: "Slack auth is not configured".to_string(),
            detail: Some("No Slack token source was found.".to_string()),
            remediation: slack_auth_remediation(),
            upgrades: Vec::new(),
        };
    }

    DoctorCheck {
        id: "slack_auth".to_string(),
        category: "auth".to_string(),
        status: "pass".to_string(),
        summary: "Slack auth source is configured".to_string(),
        detail: Some(describe_auth_sources(&sources)),
        remediation: vec![
            "To validate the token, run a Slack read command such as `slack-cli search \"hello\" --limit 1` in an interactive terminal.".to_string(),
        ],
        upgrades: Vec::new(),
    }
}

fn format_doctor_human(result: &DoctorResult) -> String {
    let mut out = String::new();
    let headline = if result.summary.failures > 0 {
        "not ready"
    } else if result.summary.warnings > 0 {
        "usable with warnings"
    } else {
        "ready"
    };
    let _ = writeln!(
        out,
        "forge doctor: {} ({} passed, {} warnings, {} failures)",
        headline, result.summary.passed, result.summary.warnings, result.summary.failures
    );

    for check in &result.checks {
        let _ = writeln!(
            out,
            "[{}] {}: {}",
            doctor_status_label(&check.status),
            check.id,
            check.summary
        );
        if let Some(detail) = check.detail.as_ref() {
            let detail = detail.lines().next().unwrap_or(detail);
            if !detail.is_empty() {
                let _ = writeln!(out, "  {}", detail);
            }
        }
        for item in &check.remediation {
            let _ = writeln!(out, "  fix: {item}");
        }
        for item in &check.upgrades {
            let _ = writeln!(out, "  upgrade: {item}");
        }
    }

    out.trim_end().to_string()
}

fn doctor_status_label(status: &str) -> &'static str {
    match status {
        "pass" => "PASS",
        "warn" => "WARN",
        "fail" => "FAIL",
        _ => "INFO",
    }
}

fn emit_output<T, F>(mode: OutputMode, data: T, human: F) -> Result<()>
where
    T: Serialize,
    F: FnOnce(&T) -> String,
{
    match mode {
        OutputMode::Json => print_json(&Envelope { ok: true, data }),
        OutputMode::Human => {
            print_human_text(&human(&data));
            Ok(())
        }
    }
}

fn print_human_text(text: &str) {
    if text.ends_with('\n') {
        print!("{text}");
    } else {
        println!("{text}");
    }
}

fn format_error_human(error: &ErrorBody) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "forge error [{}]", error.code);
    for line in error.message.lines() {
        let _ = writeln!(out, "{line}");
    }
    out.trim_end().to_string()
}

fn format_update_check_human(result: &UpdateCheckResult) -> String {
    let mut out = String::new();
    let headline = if result.update_available || result.skills_out_of_date || result.codex_out_of_date
    {
        "updates available"
    } else {
        "up to date"
    };
    let _ = writeln!(out, "forge self update-check: {headline}");
    let _ = writeln!(out, "source: {}", result.source_kind);
    if let Some(path) = result.repo_path.as_ref() {
        let _ = writeln!(out, "repo: {path}");
    }
    if let Some(version) = result.current_version.as_ref() {
        let _ = writeln!(out, "current version: {version}");
    }
    if let Some(version) = result.latest_version.as_ref() {
        let _ = writeln!(out, "latest version: {version}");
    }
    if let Some(branch) = result.remote_default_branch.as_ref() {
        let _ = writeln!(out, "remote default branch: {branch}");
    }
    if let Some(head) = result.local_head.as_ref() {
        let _ = writeln!(out, "local head: {}", shorten_hash(head));
    }
    if let Some(head) = result.remote_head.as_ref() {
        let _ = writeln!(out, "remote head: {}", shorten_hash(head));
    }
    let _ = writeln!(out, "checked at unix: {}", result.checked_at_unix);
    let _ = writeln!(
        out,
        "skills: {}",
        summarize_counts(result.skills.iter().map(|entry| entry.state.as_str()))
    );
    let _ = writeln!(
        out,
        "codex: {}",
        if result.codex_out_of_date {
            "out_of_date"
        } else {
            "up_to_date"
        }
    );

    let noteworthy = result
        .skills
        .iter()
        .filter(|entry| entry.state != "up_to_date")
        .collect::<Vec<_>>();
    if !noteworthy.is_empty() {
        out.push('\n');
        let _ = writeln!(out, "skill details:");
        append_skill_status_entries(&mut out, noteworthy.into_iter());
    }

    out.trim_end().to_string()
}

fn format_update_human(result: &UpdateResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge self update: {}",
        if result.changed {
            "applied changes"
        } else {
            "already up to date"
        }
    );
    let _ = writeln!(out, "source: {}", result.source_kind);
    if let Some(path) = result.repo_path.as_ref() {
        let _ = writeln!(out, "repo: {path}");
    }
    if let Some(branch) = result.branch.as_ref() {
        let _ = writeln!(out, "branch: {branch}");
    }
    if let Some(head) = result.before_head.as_ref() {
        let _ = writeln!(out, "before head: {}", shorten_hash(head));
    }
    if let Some(head) = result.after_head.as_ref() {
        let _ = writeln!(out, "after head: {}", shorten_hash(head));
    }
    if let Some(version) = result.before_version.as_ref() {
        let _ = writeln!(out, "before version: {version}");
    }
    if let Some(version) = result.after_version.as_ref() {
        let _ = writeln!(out, "after version: {version}");
    }
    let _ = writeln!(out, "skills reconciled: {}", result.skills_reconciled);
    let _ = writeln!(out, "codex reconciled: {}", result.codex_reconciled);
    out.trim_end().to_string()
}

fn format_permissions_human(command: &str, result: &PermissionsResult) -> String {
    let mut out = String::new();
    let ok = result.items.iter().filter(|item| item.ok).count();
    let fixed = result.items.iter().filter(|item| item.changed).count();
    let missing = result.items.iter().filter(|item| !item.exists).count();
    let mismatched = result
        .items
        .iter()
        .filter(|item| item.exists && !item.ok && !item.changed)
        .count();
    let _ = writeln!(
        out,
        "forge permissions {command}: {} ok, {} fixed, {} mismatched, {} missing",
        ok, fixed, mismatched, missing
    );
    for item in &result.items {
        let _ = writeln!(out, "{}", format_permission_item_human(item));
    }
    out.trim_end().to_string()
}

fn format_permission_item_human(item: &PermissionItem) -> String {
    if !item.exists {
        return format!(
            "[MISSING] {} expected {} {}",
            item.kind, item.expected_mode, item.path
        );
    }
    if item.changed {
        return format!("[FIXED] {} {} {}", item.kind, item.expected_mode, item.path);
    }
    if item.ok {
        return format!(
            "[OK] {} {} {}",
            item.kind,
            item.actual_mode.as_deref().unwrap_or(&item.expected_mode),
            item.path
        );
    }
    format!(
        "[MISMATCH] {} expected {} actual {} {}",
        item.kind,
        item.expected_mode,
        item.actual_mode.as_deref().unwrap_or("-"),
        item.path
    )
}

fn format_skills_list_human(result: &SkillsListResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge skills list: {} entries from {}",
        result.skills.len(),
        result.source
    );
    if result.skills.is_empty() {
        let _ = writeln!(out, "no Forge-managed skills were found");
        return out.trim_end().to_string();
    }
    for entry in &result.skills {
        let _ = writeln!(out, "{} [{}]", entry.name, entry.source_kind);
        if let Some(path) = entry.source_path.as_ref() {
            let _ = writeln!(out, "  source: {path}");
        }
        if entry.installed_targets.is_empty() {
            let _ = writeln!(out, "  installed: none");
        } else {
            for target in &entry.installed_targets {
                let _ = writeln!(out, "  installed: {target}");
            }
        }
    }
    out.trim_end().to_string()
}

fn format_skills_validate_human(result: &SkillsValidateResult) -> String {
    let mut out = String::new();
    let invalid = result.skills.iter().filter(|entry| !entry.valid).count();
    let _ = writeln!(
        out,
        "forge skills validate: {} ({} skills from {}, {} invalid)",
        if result.valid { "valid" } else { "invalid" },
        result.skills.len(),
        result.source_kind,
        invalid
    );
    for entry in &result.skills {
        let _ = writeln!(
            out,
            "[{}] {}",
            if entry.valid { "OK" } else { "INVALID" },
            entry.name
        );
        if let Some(path) = entry.path.as_ref() {
            let _ = writeln!(out, "  path: {path}");
        }
        for issue in &entry.issues {
            let _ = writeln!(out, "  issue: {issue}");
        }
    }
    out.trim_end().to_string()
}

fn format_skills_install_human(command: &str, result: &SkillsInstallResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge skills {command}: {} entries to {}@{} {} from {}",
        result.installs.len(),
        result.target_kind,
        result.target_role,
        result.target_root,
        result.source_kind
    );
    let _ = writeln!(
        out,
        "summary: {}",
        summarize_counts(result.installs.iter().map(|entry| entry.status.as_str()))
    );
    for entry in &result.installs {
        let _ = writeln!(
            out,
            "[{}] {} -> {}",
            status_label(&entry.status),
            entry.name,
            entry.target_path
        );
    }
    out.trim_end().to_string()
}

fn format_skills_diff_human(result: &SkillsDiffResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge skills diff: {} ({})",
        result.name,
        if result.identical { "identical" } else { "different" }
    );
    let _ = writeln!(out, "source: {}", result.source_kind);
    let _ = writeln!(out, "target: {} {}", result.target_kind, result.target_path);
    let _ = writeln!(
        out,
        "summary: {}",
        summarize_counts(result.files.iter().map(|entry| entry.status.as_str()))
    );
    for entry in &result.files {
        let _ = writeln!(out, "[{}] {}", status_label(&entry.status), entry.path);
    }
    out.trim_end().to_string()
}

fn format_skills_status_human(result: &SkillsStatusResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge skills status: {} entries from {} (scope {})",
        result.entries.len(),
        result.source_kind,
        result.scope
    );
    let _ = writeln!(
        out,
        "summary: {}",
        summarize_counts(result.entries.iter().map(|entry| entry.state.as_str()))
    );
    append_skill_status_entries(&mut out, result.entries.iter());
    out.trim_end().to_string()
}

fn append_skill_status_entries<'a, I>(out: &mut String, entries: I)
where
    I: IntoIterator<Item = &'a SkillStatusEntry>,
{
    for entry in entries {
        let _ = writeln!(
            out,
            "[{}] {} {}@{} -> {}",
            status_label(&entry.state),
            entry.name,
            entry.target_kind,
            entry.target_role,
            entry.target_path
        );
    }
}

fn format_codex_render_human(result: &CodexRenderResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge codex render: {} assets from {} for {} {}",
        result.assets.len(),
        result.source_kind,
        result.target_kind,
        result.target_root
    );
    for (index, asset) in result.assets.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let _ = writeln!(
            out,
            "--- {} {} -> {} [{}]",
            asset.name, asset.relative_path, asset.target_path, asset.source_hash
        );
        if let Some(path) = asset.source_path.as_ref() {
            let _ = writeln!(out, "source: {path}");
        }
        out.push('\n');
        out.push_str(&asset.contents);
        if !asset.contents.ends_with('\n') {
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

fn format_codex_diff_human(result: &CodexDiffResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge codex diff: {} ({}) for {} {} from {}",
        result.assets.len(),
        if result.identical { "identical" } else { "different" },
        result.target_kind,
        result.target_root,
        result.source_kind
    );
    let _ = writeln!(
        out,
        "summary: {}",
        summarize_counts(result.assets.iter().map(|entry| entry.status.as_str()))
    );
    for entry in &result.assets {
        let _ = writeln!(
            out,
            "[{}] {} -> {}",
            status_label(&entry.status),
            entry.relative_path,
            entry.target_path
        );
    }
    out.trim_end().to_string()
}

fn format_codex_install_human(result: &CodexInstallResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "forge codex install: {} assets to {} {} from {}",
        result.assets.len(),
        result.target_kind,
        result.target_root,
        result.source_kind
    );
    let _ = writeln!(
        out,
        "summary: {}",
        summarize_counts(result.assets.iter().map(|entry| entry.status.as_str()))
    );
    for entry in &result.assets {
        let _ = writeln!(
            out,
            "[{}] {} -> {}",
            status_label(&entry.status),
            entry.relative_path,
            entry.target_path
        );
    }
    out.trim_end().to_string()
}

fn summarize_counts<'a, I>(values: I) -> String
where
    I: IntoIterator<Item = &'a str>,
{
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        *counts.entry(value.to_string()).or_default() += 1;
    }

    if counts.is_empty() {
        return "none".to_string();
    }

    counts
        .into_iter()
        .map(|(status, count)| format!("{count} {status}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn status_label(status: &str) -> String {
    status.to_ascii_uppercase()
}

fn shorten_hash(value: &str) -> String {
    if value.len() > 12 {
        value[..12].to_string()
    } else {
        value.to_string()
    }
}

fn describe_auth_sources(sources: &[String]) -> String {
    if sources.is_empty() {
        "No configured auth sources were detected.".to_string()
    } else {
        format!("Detected auth sources: {}", sources.join(", "))
    }
}

fn run_command_capture(program: &str, args: &[&str]) -> Result<std::process::Output> {
    ProcessCommand::new(program)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {program} {}", args.join(" ")))
}

fn stdout_or_stderr_trimmed(output: &std::process::Output) -> Option<String> {
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

fn output_failure_detail(output: &std::process::Output) -> Option<String> {
    let base = stdout_or_stderr_trimmed(output);
    match (base, output.status.code()) {
        (Some(detail), Some(code)) => Some(format!("{detail} (exit code {code})")),
        (Some(detail), None) => Some(format!("{detail} (terminated by signal)")),
        (None, Some(code)) => Some(format!("exit code {code}")),
        (None, None) => Some("terminated by signal".to_string()),
    }
}

fn tool_remediation(program: &str) -> Vec<String> {
    platform_tool_remediation(env::consts::OS, program)
}

fn platform_tool_remediation(os: &str, program: &str) -> Vec<String> {
    match (os, program) {
        ("windows", "git") => vec!["winget install --id Git.Git".to_string()],
        ("windows", "gh") => vec!["winget install --id GitHub.cli".to_string()],
        ("windows", "rg") => vec!["cargo install ripgrep".to_string()],
        ("windows", "jq") => vec!["cargo install jq-cli".to_string()],
        ("macos", "git") => vec!["xcode-select --install".to_string()],
        ("macos", "gh") => vec!["brew install gh".to_string()],
        ("macos", "rg") => vec!["cargo install ripgrep".to_string()],
        ("macos", "jq") => vec!["cargo install jq-cli".to_string()],
        ("linux", "git") => vec!["sudo apt install git".to_string()],
        ("linux", "gh") => vec![
            "See https://cli.github.com for the recommended install path on your distro."
                .to_string(),
        ],
        ("linux", "rg") => vec!["cargo install ripgrep".to_string()],
        ("linux", "jq") => vec!["cargo install jq-cli".to_string()],
        (_, "cargo") => vec!["Install Rust with rustup from https://rustup.rs.".to_string()],
        _ => Vec::new(),
    }
}

fn gh_auth_remediation() -> Vec<String> {
    vec![
        "Verify interactively in your terminal with `gh auth status`.".to_string(),
        "If interactive `gh auth status` still fails, run `gh auth login`.".to_string(),
        "Forge should continue with a warning even when this check cannot be confirmed from a non-interactive subprocess.".to_string(),
    ]
}

fn linear_auth_remediation() -> Vec<String> {
    vec![
        "Initialize config with `linear config` if needed.".to_string(),
        "Store credentials with `linear auth login` or by writing ~/.config/forge/linear/token."
            .to_string(),
        "See docs/linear.md for the supported auth layout and command contract.".to_string(),
    ]
}

fn slack_auth_remediation() -> Vec<String> {
    vec![
        "Store credentials with `slack-cli auth login` or by writing ~/.config/forge/slack-cli/token.".to_string(),
        "See docs/slack-cli.md for the supported token layout, scopes, and setup flow.".to_string(),
    ]
}

fn linear_auth_sources() -> Vec<String> {
    auth_sources_from_dir(
        linear_config_dir_path(),
        env_var_present("LINEAR_API_KEY"),
        "LINEAR_API_KEY",
        parse_linear_doctor_config,
    )
}

fn slack_auth_sources() -> Vec<String> {
    auth_sources_from_dir(
        slack_config_dir_path(),
        env_var_present("SLACK_API_TOKEN"),
        "SLACK_API_TOKEN",
        parse_slack_doctor_config,
    )
}

fn auth_sources_from_dir<F>(
    config_dir: PathBuf,
    env_present: bool,
    env_var_name: &str,
    parse_config: F,
) -> Vec<String>
where
    F: Fn(&str) -> Option<(bool, Option<String>)>,
{
    let mut sources = Vec::new();
    if env_present {
        sources.push(format!("env:{env_var_name}"));
    }

    let config_path = config_dir.join("config.toml");
    if config_path.exists() {
        sources.push(format!("file:{}", config_path.display()));
        if let Ok(body) = fs::read_to_string(&config_path) {
            if let Some((has_inline_token, token_file)) = parse_config(&body) {
                if has_inline_token {
                    sources.push("config:inline_token".to_string());
                }
                if let Some(token_file) = token_file {
                    let path = expand_path(&token_file);
                    if path.exists() {
                        sources.push(format!("config:token_file:{}", path.display()));
                    }
                }
            }
        }
    }

    let token_path = config_dir.join("token");
    if token_path.exists() {
        sources.push(format!("file:{}", token_path.display()));
    }

    dedup_strings(sources)
}

fn parse_linear_doctor_config(body: &str) -> Option<(bool, Option<String>)> {
    toml::from_str::<LinearDoctorConfig>(body)
        .ok()
        .map(|config| {
            (
                has_nonempty_option(config.token.as_ref()),
                config.token_file,
            )
        })
}

fn parse_slack_doctor_config(body: &str) -> Option<(bool, Option<String>)> {
    toml::from_str::<SlackDoctorConfig>(body)
        .ok()
        .map(|config| {
            (
                has_nonempty_option(config.token.as_ref()),
                config.token_file,
            )
        })
}

fn env_var_present(name: &str) -> bool {
    env::var(name)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn has_nonempty_option(value: Option<&String>) -> bool {
    value.is_some_and(|item| !item.trim().is_empty())
}

fn dedup_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

fn linear_config_dir_path() -> PathBuf {
    if let Ok(path) = env::var("FORGE_LINEAR_CLI_CONFIG_DIR") {
        return expand_path(&path);
    }
    base_forge_config_dir().join("linear")
}

fn slack_config_dir_path() -> PathBuf {
    if let Ok(path) = env::var("FORGE_SLACK_CLI_CONFIG_DIR") {
        return expand_path(&path);
    }
    base_forge_config_dir().join("slack-cli")
}

fn base_forge_config_dir() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("forge");
    }
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("forge")
}

fn tool_upgrade_commands(program: &str) -> Vec<String> {
    tool_upgrade_commands_for(env::consts::OS, program)
}

fn tool_upgrade_commands_for(os: &str, program: &str) -> Vec<String> {
    match (os, program) {
        ("windows", "git") => vec!["winget upgrade --id Git.Git".to_string()],
        ("windows", "gh") => vec!["winget upgrade --id GitHub.cli".to_string()],
        ("windows", "rg") => vec!["cargo install ripgrep --force".to_string()],
        ("windows", "jq") => vec!["cargo install jq-cli --force".to_string()],
        ("macos", "gh") => vec!["brew upgrade gh".to_string()],
        ("macos", "rg") => vec!["cargo install ripgrep --force".to_string()],
        ("macos", "jq") => vec!["cargo install jq-cli --force".to_string()],
        ("linux", "gh") => vec![
            "See https://cli.github.com for the recommended upgrade path on your distro."
                .to_string(),
        ],
        ("linux", "rg") => vec!["cargo install ripgrep --force".to_string()],
        ("linux", "jq") => vec!["cargo install jq-cli --force".to_string()],
        _ => Vec::new(),
    }
}

fn shell_escape_path(path: &Path) -> String {
    let value = path.display().to_string();
    if value.contains(' ') {
        format!("\"{value}\"")
    } else {
        value
    }
}

fn format_mode(mode: u32) -> String {
    format!("{mode:04o}")
}

fn print_json<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    println!(
        "{}",
        serde_json::to_string(value).context("failed to render JSON output")?
    );
    Ok(())
}

fn load_config() -> Result<ForgeConfig> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(ForgeConfig::default());
    }

    let body = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file at {}", path.display()))?;
    toml::from_str(&body)
        .with_context(|| format!("failed to parse config file at {}", path.display()))
}

fn load_state(path: &Path) -> Result<ForgeState> {
    let body = fs::read_to_string(path)
        .with_context(|| format!("failed to read state file at {}", path.display()))?;
    toml::from_str(&body).with_context(|| {
        format!(
            "failed to parse state file at {}. if the Forge state schema changed during development, remove or migrate this file and reinstall managed skills",
            path.display()
        )
    })
}

fn load_state_or_default() -> Result<ForgeState> {
    let path = state_file_path()?;
    if !path.exists() {
        return Ok(ForgeState::default());
    }
    load_state(&path)
}

fn save_state(path: &Path, state: &ForgeState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let body = toml::to_string_pretty(state).context("failed to serialize state file")?;
    fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))
}

fn discover_repo_path(cli_repo_path: Option<PathBuf>, config: &ForgeConfig) -> Option<PathBuf> {
    if let Some(path) = cli_repo_path {
        return Some(path);
    }
    if let Some(path) = config.repo_path.as_ref() {
        return Some(expand_path(path));
    }
    if let Ok(cwd) = env::current_dir() {
        if cwd.join(".git").exists() && repo_skills_dir(&cwd).exists() {
            return Some(cwd);
        }
    }
    None
}

fn ensure_git_repo(path: &Path) -> Result<()> {
    if path.join(".git").exists() {
        return Ok(());
    }
    bail!("repo_path is not a git repository: {}", path.display())
}

fn remote_default_branch_and_head(path: &Path) -> Result<(Option<String>, Option<String>)> {
    let output = run_command(path, "git", &["ls-remote", "--symref", "origin", "HEAD"])?;
    if !output.status.success() {
        return Ok((None, None));
    }

    let stdout = String::from_utf8(output.stdout).context("git ls-remote output was not UTF-8")?;
    let mut branch = None;
    let mut head = None;

    for line in stdout.lines() {
        if let Some(stripped) = line.strip_prefix("ref: refs/heads/") {
            let parts: Vec<&str> = stripped.split('\t').collect();
            if let Some(name) = parts.first() {
                branch = Some((*name).to_string());
            }
        } else if line.ends_with("\tHEAD") {
            if let Some((sha, _)) = line.split_once('\t') {
                head = Some(sha.to_string());
            }
        }
    }

    Ok((branch, head))
}

fn git_stdout(path: &Path, args: &[&str]) -> Result<String> {
    let output = run_command(path, "git", args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    let stdout = String::from_utf8(output.stdout).context("git output was not UTF-8")?;
    Ok(stdout.trim().to_string())
}

fn run_git(path: &Path, args: &[&str]) -> Result<()> {
    let output = run_command(path, "git", args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(())
}

fn run_command(path: &Path, program: &str, args: &[&str]) -> Result<std::process::Output> {
    ProcessCommand::new(program)
        .args(args)
        .current_dir(path)
        .output()
        .with_context(|| format!("failed to run {program} {}", args.join(" ")))
}

fn run_command_capture_lines(program: &str, args: &[&str]) -> Result<Vec<String>> {
    let output = run_command_capture(program, args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("{program} {} failed: {}", args.join(" "), stderr.trim());
    }
    let stdout = String::from_utf8(output.stdout).context("command output was not UTF-8")?;
    Ok(stdout.lines().map(|line| line.to_string()).collect())
}

fn config_dir_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("FORGE_CONFIG_DIR") {
        return Ok(expand_path(&path));
    }
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("forge"));
    }
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".config").join("forge"))
}

fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join("config.toml"))
}

fn state_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join("state.toml"))
}

fn expand_path(path: &str) -> PathBuf {
    if path == "~" {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(path)
}

fn now_unix() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| anyhow!("system clock is before UNIX_EPOCH"))?
        .as_secs())
}

fn latest_release_version() -> Result<Option<String>> {
    let lines = run_command_capture_lines("git", &["ls-remote", "--tags", FORGE_REPO_URL])?;
    Ok(latest_release_version_from_lines(&lines))
}

fn latest_release_version_from_lines(lines: &[String]) -> Option<String> {
    lines
        .iter()
        .filter_map(|line| line.split_once('\t').map(|(_, refname)| refname))
        .filter_map(|refname| refname.strip_prefix("refs/tags/"))
        .filter(|tag| !tag.ends_with("^{}"))
        .filter_map(|tag| parse_calver(tag).map(|parts| (parts, tag.to_string())))
        .max_by_key(|(parts, _)| *parts)
        .map(|(_, tag)| tag)
}

fn parse_calver(value: &str) -> Option<(u32, u32)> {
    let parts = value.split('.').collect::<Vec<_>>();
    match parts.as_slice() {
        [date, "0", sequence] => {
            let date = date.parse::<u32>().ok()?;
            let sequence = sequence.parse::<u32>().ok()?;
            if date < 10000000 {
                return None;
            }
            Some((date, sequence))
        }
        [year, month_day, sequence] => {
            let year = year.parse::<u32>().ok()?;
            let month_day = month_day.parse::<u32>().ok()?;
            let sequence = sequence.parse::<u32>().ok()?;
            if month_day > 1231 {
                return None;
            }
            let date = year
                .checked_mul(10000)?
                .checked_add(month_day)?;
            Some((date, sequence))
        }
        _ => None,
    }
}

fn install_release_packages(version: &str) -> Result<()> {
    for package in RELEASE_PACKAGES {
        let output = run_command_capture(
            "cargo",
            &[
                "install",
                "--git",
                FORGE_REPO_URL,
                "--tag",
                version,
                "--locked",
                "--force",
                package,
            ],
        )?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "cargo install --git {} --tag {} --locked --force {} failed: {}",
                FORGE_REPO_URL,
                version,
                package,
                stderr.trim()
            );
        }
    }
    Ok(())
}

fn classify_error(error: &anyhow::Error) -> ErrorBody {
    let message = error.to_string();
    let code = match message.as_str() {
        // User mistakes (stable codes for agent branching).
        msg if msg.contains("provide a skill name or --all") => "invalid_usage",
        msg if msg.contains("--branch is only supported in repo-checkout mode") => "invalid_usage",
        msg if msg.contains("invalid target:") || msg.contains("path target must be absolute:") => {
            "invalid_target"
        }
        msg if msg.contains("repo source requires a Forge repo checkout")
            || msg.contains("forge_repo target requires a configured Forge repo path") =>
        {
            "repo_required"
        }
        msg if msg.contains("HOME is not set") => "env_error",
        msg if msg.contains("Operation not permitted")
            || msg.contains("Permission denied")
            || msg.contains("failed to write")
            || msg.contains("failed to create") =>
        {
            "permission_error"
        }
        msg if msg.contains("not a git repository") => "not_git_repo",
        msg if msg.contains("failed to run git") => "git_unavailable",
        msg if msg.contains("git pull --rebase") => "update_failed",
        msg if msg.contains("failed to read config file")
            || msg.contains("failed to parse config file") =>
        {
            "config_error"
        }
        msg if msg.contains("skill not found:") => "skill_not_found",
        msg if msg.contains("skill") && msg.contains("not found") => "skill_not_found",
        msg if msg.contains("unmanaged") => "unmanaged_collision",
        msg if msg.contains("frontmatter") => "validation_error",
        msg if msg.contains("codex asset not found") => "codex_asset_not_found",
        _ => "internal_error",
    };

    ErrorBody {
        code: code.to_string(),
        message,
    }
}

fn release_codex_assets() -> &'static [EmbeddedCodexAsset] {
    &[
        embedded_codex_asset!("agents", "AGENTS.md"),
        embedded_codex_asset!("rules", "rules/user-policy.rules"),
    ]
}

fn load_release_codex_assets() -> Vec<CodexAssetDefinition> {
    release_codex_assets()
        .iter()
        .map(|asset| CodexAssetDefinition {
            name: asset.name.to_string(),
            relative_path: asset.relative_path.to_string(),
            source_path: None,
            contents: asset.contents.as_bytes().to_vec(),
        })
        .collect()
}

fn load_repo_codex_assets(repo_path: &Path) -> Result<Vec<CodexAssetDefinition>> {
    codex_asset_specs()
        .iter()
        .map(|spec| {
            let path = repo_codex_user_dir(repo_path).join(spec.relative_path);
            let contents =
                fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
            Ok(CodexAssetDefinition {
                name: spec.name.to_string(),
                relative_path: spec.relative_path.to_string(),
                source_path: Some(path),
                contents,
            })
        })
        .collect()
}

fn load_codex_assets_for_source(
    source_kind: &SkillSourceKind,
    repo_path: Option<&Path>,
) -> Result<Vec<CodexAssetDefinition>> {
    match source_kind {
        SkillSourceKind::RepoCheckout => {
            let path =
                repo_path.ok_or_else(|| anyhow!("repo source requires a Forge repo checkout"))?;
            load_repo_codex_assets(path)
        }
        SkillSourceKind::Release => Ok(load_release_codex_assets()),
    }
}

fn select_codex_assets(
    defs: Vec<CodexAssetDefinition>,
    assets: &[CodexAssetArg],
) -> Result<Vec<CodexAssetDefinition>> {
    if assets.is_empty() {
        return Ok(defs);
    }

    let wanted = assets
        .iter()
        .map(codex_asset_name_from_arg)
        .collect::<BTreeSet<_>>();
    let selected = defs
        .into_iter()
        .filter(|asset| wanted.contains(asset.name.as_str()))
        .collect::<Vec<_>>();

    if selected.is_empty() {
        bail!("codex asset not found");
    }

    Ok(selected)
}

fn release_skills() -> &'static [EmbeddedSkill] {
    &[
        embedded_skill!("forge-tools"),
        embedded_skill!("design-algorithm"),
        embedded_skill!("gh-body-file"),
        embedded_skill!("linear-cli"),
        embedded_skill!("slack-cli-research"),
        embedded_skill!("codex-threads-cli"),
        embedded_skill!("forge-cli-admin"),
    ]
}

fn load_release_skills() -> Vec<SkillDefinition> {
    release_skills()
        .iter()
        .map(|skill| {
            let mut files = BTreeMap::new();
            files.insert("SKILL.md".to_string(), skill.skill_md.as_bytes().to_vec());
            SkillDefinition {
                name: skill.name.to_string(),
                source_kind: SkillSourceKind::Release,
                source_path: None,
                source_ref: env!("CARGO_PKG_VERSION").to_string(),
                source_repo_path: None,
                files,
            }
        })
        .collect()
}

fn load_repo_skills(repo_path: &Path) -> Result<Vec<SkillDefinition>> {
    let skills_root = repo_skills_dir(repo_path);
    let mut defs = Vec::new();
    for entry in fs::read_dir(&skills_root)
        .with_context(|| format!("failed to read {}", skills_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid skill directory name: {}", path.display()))?
            .to_string();
        defs.push(SkillDefinition {
            name,
            source_kind: SkillSourceKind::RepoCheckout,
            source_path: Some(path.clone()),
            source_ref: git_repo_ref(repo_path).unwrap_or_else(|| "repo".to_string()),
            source_repo_path: Some(repo_path.to_path_buf()),
            files: load_skill_files_from_dir(&path)?,
        });
    }
    defs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(defs)
}

fn load_skills_for_source(
    source_kind: &SkillSourceKind,
    repo_path: Option<&Path>,
) -> Result<Vec<SkillDefinition>> {
    match source_kind {
        SkillSourceKind::RepoCheckout => {
            let path =
                repo_path.ok_or_else(|| anyhow!("repo source requires a Forge repo checkout"))?;
            load_repo_skills(path)
        }
        SkillSourceKind::Release => Ok(load_release_skills()),
    }
}

fn load_skill_definition(
    source_kind: &SkillSourceKind,
    repo_path: Option<&Path>,
    name: &str,
) -> Result<SkillDefinition> {
    load_skills_for_source(source_kind, repo_path)?
        .into_iter()
        .find(|def| def.name == name)
        .ok_or_else(|| anyhow!("skill not found: {name}"))
}

fn git_repo_ref(repo_path: &Path) -> Option<String> {
    git_stdout(repo_path, &["rev-parse", "HEAD"]).ok()
}

fn load_skill_files_from_dir(root: &Path) -> Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    collect_files(root, root, &mut files)?;
    Ok(files)
}

fn collect_files(root: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .with_context(|| format!("failed to strip prefix from {}", path.display()))?;
        if path.is_dir() {
            collect_files(root, &path, files)?;
        } else {
            files.insert(
                rel.to_string_lossy().replace('\\', "/"),
                fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?,
            );
        }
    }
    Ok(())
}

fn write_skill_definition(target_path: &Path, def: &SkillDefinition) -> Result<()> {
    fs::create_dir_all(target_path)
        .with_context(|| format!("failed to create {}", target_path.display()))?;
    for (rel, contents) in &def.files {
        let path = target_path.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&path, contents)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn resolve_source_kind(
    source: Option<SkillSourceArg>,
    repo_path: Option<&Path>,
) -> Result<SkillSourceKind> {
    Ok(match source {
        Some(kind) => map_cli_source(kind),
        None => auto_source_kind(repo_path),
    })
}

fn auto_source_kind(repo_path: Option<&Path>) -> SkillSourceKind {
    if repo_path.is_some() {
        SkillSourceKind::RepoCheckout
    } else {
        SkillSourceKind::Release
    }
}

fn map_cli_source(source: SkillSourceArg) -> SkillSourceKind {
    match source {
        SkillSourceArg::Repo => SkillSourceKind::RepoCheckout,
        SkillSourceArg::Release => SkillSourceKind::Release,
    }
}

fn map_target_role(role: SkillTargetRoleArg) -> SkillTargetRole {
    match role {
        SkillTargetRoleArg::Mainline => SkillTargetRole::Mainline,
        SkillTargetRoleArg::Development => SkillTargetRole::Development,
    }
}

fn select_skill_defs(
    defs: Vec<SkillDefinition>,
    skill: Option<&str>,
    all: bool,
) -> Result<Vec<SkillDefinition>> {
    if all {
        return Ok(defs);
    }
    let name = skill.ok_or_else(|| anyhow!("provide a skill name or --all"))?;
    let selected = defs
        .into_iter()
        .filter(|def| def.name == name)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        bail!("skill not found: {name}");
    }
    Ok(selected)
}

fn resolve_target(
    target: Option<&str>,
    config: &ForgeConfig,
    repo_path: Option<&Path>,
    target_role: Option<SkillTargetRole>,
) -> Result<ResolvedTarget> {
    match target {
        None | Some("user") => Ok(ResolvedTarget {
            kind: SkillTargetKind::User,
            role: target_role.unwrap_or(SkillTargetRole::Mainline),
            root: user_skills_dir()?,
        }),
        Some("forge_repo") => {
            let repo = repo_path.ok_or_else(|| {
                anyhow!("forge_repo target requires a configured Forge repo path")
            })?;
            let subpath = config
                .forge_repo_install_subpath
                .clone()
                .unwrap_or_else(|| DEFAULT_FORGE_REPO_INSTALL_SUBPATH.to_string());
            Ok(ResolvedTarget {
                kind: SkillTargetKind::ForgeRepo,
                role: target_role.unwrap_or(SkillTargetRole::Development),
                root: repo.join(subpath),
            })
        }
        Some(raw) if raw.starts_with("path:") => {
            let path = PathBuf::from(raw.trim_start_matches("path:"));
            if !path.is_absolute() {
                bail!("path target must be absolute: {}", path.display());
            }
            Ok(ResolvedTarget {
                kind: SkillTargetKind::Path,
                role: target_role.unwrap_or(SkillTargetRole::Development),
                root: path,
            })
        }
        Some(other) => bail!("invalid target: {other}"),
    }
}

fn user_skills_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".agents").join("skills"))
}

fn user_codex_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".codex"))
}

fn repo_skills_dir(repo_path: &Path) -> PathBuf {
    repo_path.join(REPO_SKILLS_SUBPATH)
}

fn repo_codex_user_dir(repo_path: &Path) -> PathBuf {
    repo_path.join(REPO_CODEX_USER_SUBPATH)
}

fn resolve_codex_target(target: Option<&str>) -> Result<ResolvedCodexTarget> {
    match target {
        None | Some("user") => Ok(ResolvedCodexTarget {
            kind: CodexTargetKind::User,
            root: user_codex_dir()?,
        }),
        Some(raw) if raw.starts_with("path:") => {
            let path = PathBuf::from(raw.trim_start_matches("path:"));
            if !path.is_absolute() {
                bail!("path target must be absolute: {}", path.display());
            }
            Ok(ResolvedCodexTarget {
                kind: CodexTargetKind::Path,
                root: path,
            })
        }
        Some(other) => bail!("invalid target: {other}"),
    }
}

fn parse_skill_frontmatter(body: &str) -> Result<SkillFrontmatter> {
    let rest = body
        .strip_prefix("---\n")
        .or_else(|| body.strip_prefix("---\r\n"))
        .ok_or_else(|| anyhow!("missing opening frontmatter delimiter"))?;
    let end = rest
        .find("\n---")
        .or_else(|| rest.find("\r\n---"))
        .ok_or_else(|| anyhow!("missing closing frontmatter delimiter"))?;
    let frontmatter = &rest[..end];
    let parsed: SkillFrontmatter =
        serde_yaml::from_str(frontmatter).context("failed to parse YAML frontmatter")?;
    Ok(parsed)
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: String,
    description: String,
}

fn hash_skill_files(files: &BTreeMap<String, Vec<u8>>) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for (path, bytes) in files {
        for byte in path
            .as_bytes()
            .iter()
            .chain([0u8].iter())
            .chain(bytes.iter())
        {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    format!("{hash:016x}")
}

fn build_diff_files(
    source: &BTreeMap<String, Vec<u8>>,
    target: &BTreeMap<String, Vec<u8>>,
) -> Vec<SkillDiffFile> {
    let mut names = BTreeSet::new();
    names.extend(source.keys().cloned());
    names.extend(target.keys().cloned());
    names
        .into_iter()
        .map(|path| {
            let source_hash = source.get(&path).map(hash_bytes);
            let target_hash = target.get(&path).map(hash_bytes);
            let status = match (source.get(&path), target.get(&path)) {
                (Some(lhs), Some(rhs)) if lhs == rhs => "same",
                (Some(_), Some(_)) => "changed",
                (Some(_), None) => "added",
                (None, Some(_)) => "removed",
                (None, None) => "same",
            };
            SkillDiffFile {
                path,
                status: status.to_string(),
                source_hash,
                target_hash,
            }
        })
        .collect()
}

fn hash_bytes(bytes: &Vec<u8>) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn upsert_managed_install(state: &mut ForgeState, entry: ManagedSkillInstall) {
    if let Some(existing) = state.managed_skill_installs.iter_mut().find(|current| {
        current.skill_name == entry.skill_name && current.target_path == entry.target_path
    }) {
        *existing = entry;
        return;
    }
    state.managed_skill_installs.push(entry);
    state.managed_skill_installs.sort_by(|a, b| {
        a.skill_name
            .cmp(&b.skill_name)
            .then(a.target_path.cmp(&b.target_path))
    });
}

fn matches_scope(role: &SkillTargetRole, scope: SkillsStatusScope) -> bool {
    match scope {
        SkillsStatusScope::All => true,
        SkillsStatusScope::Mainline => *role == SkillTargetRole::Mainline,
        SkillsStatusScope::Development => *role == SkillTargetRole::Development,
    }
}

fn skills_status(args: SkillsStatusArgs) -> Result<SkillsStatusResult> {
    let config = load_config()?;
    let state = load_state_or_default()?;
    let repo_path = discover_repo_path(args.repo_path, &config);
    let source_kind = auto_source_kind(repo_path.as_deref());
    let target_filter = match args.target {
        Some(raw) => {
            let resolved = resolve_target(
                Some(raw.as_str()),
                &config,
                repo_path.as_deref(),
                args.target_role.map(map_target_role),
            )?;
            Some(TargetFilter {
                kind: resolved.kind,
                role: args.target_role.map(map_target_role),
                root: resolved.root,
            })
        }
        None => None,
    };
    skills_status_with_source(
        &config,
        &state,
        source_kind,
        repo_path,
        args.scope,
        target_filter,
    )
}

fn skills_status_with_source(
    config: &ForgeConfig,
    state: &ForgeState,
    source_kind: SkillSourceKind,
    repo_path: Option<PathBuf>,
    scope: SkillsStatusScope,
    target_filter: Option<TargetFilter>,
) -> Result<SkillsStatusResult> {
    let defs = load_skills_for_source(&source_kind, repo_path.as_deref())?;
    let source_hashes = defs
        .iter()
        .map(|def| (def.name.clone(), hash_skill_files(&def.files)))
        .collect::<BTreeMap<_, _>>();

    let mut entries = Vec::new();
    for install in &state.managed_skill_installs {
        if !matches_scope(&install.target_role, scope) {
            continue;
        }
        if let Some(filter) = target_filter.as_ref() {
            if install.target_root != filter.root.display().to_string()
                || install.target_kind != filter.kind
                || filter
                    .role
                    .as_ref()
                    .is_some_and(|role| &install.target_role != role)
            {
                continue;
            }
        }
        let target_path = PathBuf::from(&install.target_path);
        if !target_path.exists() {
            entries.push(SkillStatusEntry {
                name: install.skill_name.clone(),
                target_kind: target_kind_name(&install.target_kind).to_string(),
                target_role: target_role_name(&install.target_role).to_string(),
                target_path: install.target_path.clone(),
                state: "missing".to_string(),
                source_kind: source_kind_name(&source_kind).to_string(),
                source_hash: source_hashes.get(&install.skill_name).cloned(),
                target_hash: None,
            });
            continue;
        }

        let target_files = load_skill_files_from_dir(&target_path)?;
        let target_hash = hash_skill_files(&target_files);
        let source_hash = source_hashes.get(&install.skill_name).cloned();
        let state_name = match source_hash.as_ref() {
            Some(hash) if hash == &target_hash => "up_to_date",
            Some(_) => "out_of_date",
            None => "diverged",
        };
        entries.push(SkillStatusEntry {
            name: install.skill_name.clone(),
            target_kind: target_kind_name(&install.target_kind).to_string(),
            target_role: target_role_name(&install.target_role).to_string(),
            target_path: install.target_path.clone(),
            state: state_name.to_string(),
            source_kind: source_kind_name(&source_kind).to_string(),
            source_hash,
            target_hash: Some(target_hash),
        });
    }

    for target in managed_target_roots(config, repo_path.as_deref())? {
        if !matches_scope(&target.role, scope) {
            continue;
        }
        if let Some(filter) = target_filter.as_ref() {
            if target.root != filter.root
                || target.kind != filter.kind
                || filter
                    .role
                    .as_ref()
                    .is_some_and(|role| &target.role != role)
            {
                continue;
            }
        }
        if !target.root.exists() {
            continue;
        }
        for def in &defs {
            let candidate = target.root.join(&def.name);
            if !candidate.exists() {
                continue;
            }
            let managed = state
                .managed_skill_installs
                .iter()
                .any(|entry| entry.target_path == candidate.display().to_string());
            if managed {
                continue;
            }
            let target_files = load_skill_files_from_dir(&candidate)?;
            entries.push(SkillStatusEntry {
                name: def.name.clone(),
                target_kind: target_kind_name(&target.kind).to_string(),
                target_role: target_role_name(&target.role).to_string(),
                target_path: candidate.display().to_string(),
                state: "unmanaged_collision".to_string(),
                source_kind: source_kind_name(&source_kind).to_string(),
                source_hash: source_hashes.get(&def.name).cloned(),
                target_hash: Some(hash_skill_files(&target_files)),
            });
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name).then(a.target_path.cmp(&b.target_path)));
    Ok(SkillsStatusResult {
        source_kind: source_kind_name(&source_kind).to_string(),
        scope: status_scope_name(scope).to_string(),
        entries,
    })
}

fn mainline_targets_for_reconcile(
    config: &ForgeConfig,
    state: &ForgeState,
    repo_path: Option<&Path>,
) -> Result<Vec<ResolvedTarget>> {
    let mut targets = managed_target_roots(config, repo_path)?
        .into_iter()
        .filter(|target| target.role == SkillTargetRole::Mainline)
        .collect::<Vec<_>>();

    for install in &state.managed_skill_installs {
        if install.target_role != SkillTargetRole::Mainline {
            continue;
        }
        let root = PathBuf::from(&install.target_root);
        let exists = targets.iter().any(|target| {
            target.kind == install.target_kind
                && target.root == root
                && target.role == install.target_role
        });
        if exists {
            continue;
        }
        targets.push(ResolvedTarget {
            kind: install.target_kind.clone(),
            role: install.target_role.clone(),
            root,
        });
    }

    targets.sort_by(|a, b| {
        target_kind_name(&a.kind)
            .cmp(target_kind_name(&b.kind))
            .then(a.root.cmp(&b.root))
    });
    targets.dedup_by(|a, b| a.kind == b.kind && a.role == b.role && a.root == b.root);
    Ok(targets)
}

fn managed_target_roots(
    config: &ForgeConfig,
    repo_path: Option<&Path>,
) -> Result<Vec<ResolvedTarget>> {
    let mut targets = vec![ResolvedTarget {
        kind: SkillTargetKind::User,
        role: SkillTargetRole::Mainline,
        root: user_skills_dir()?,
    }];
    if let Some(repo) = repo_path {
        targets.push(ResolvedTarget {
            kind: SkillTargetKind::ForgeRepo,
            role: SkillTargetRole::Development,
            root: repo.join(
                config
                    .forge_repo_install_subpath
                    .clone()
                    .unwrap_or_else(|| DEFAULT_FORGE_REPO_INSTALL_SUBPATH.to_string()),
            ),
        });
    }
    Ok(targets)
}

fn source_kind_name(kind: &SkillSourceKind) -> &'static str {
    match kind {
        SkillSourceKind::RepoCheckout => "repo",
        SkillSourceKind::Release => "release",
    }
}

fn target_kind_name(kind: &SkillTargetKind) -> &'static str {
    match kind {
        SkillTargetKind::User => "user",
        SkillTargetKind::ForgeRepo => "forge_repo",
        SkillTargetKind::Path => "path",
    }
}

fn target_role_name(role: &SkillTargetRole) -> &'static str {
    match role {
        SkillTargetRole::Mainline => "mainline",
        SkillTargetRole::Development => "development",
    }
}

fn codex_target_kind_name(kind: &CodexTargetKind) -> &'static str {
    match kind {
        CodexTargetKind::User => "user",
        CodexTargetKind::Path => "path",
    }
}

fn status_scope_name(scope: SkillsStatusScope) -> &'static str {
    match scope {
        SkillsStatusScope::Mainline => "mainline",
        SkillsStatusScope::Development => "development",
        SkillsStatusScope::All => "all",
    }
}

fn target_to_flag(kind: &SkillTargetKind, role: &SkillTargetRole, root: &Path) -> String {
    match kind {
        SkillTargetKind::User => format!("user@{}", target_role_name(role)),
        SkillTargetKind::ForgeRepo => format!("forge_repo@{}", target_role_name(role)),
        SkillTargetKind::Path => format!("path:{}@{}", root.display(), target_role_name(role)),
    }
}

#[derive(Debug, Clone, Copy)]
struct CodexAssetSpec {
    name: &'static str,
    relative_path: &'static str,
}

fn codex_asset_specs() -> &'static [CodexAssetSpec] {
    &[
        CodexAssetSpec {
            name: "agents",
            relative_path: CODEX_AGENTS_REL_PATH,
        },
        CodexAssetSpec {
            name: "rules",
            relative_path: CODEX_RULES_REL_PATH,
        },
    ]
}

fn codex_asset_name_from_arg(asset: &CodexAssetArg) -> &'static str {
    match asset {
        CodexAssetArg::Agents => "agents",
        CodexAssetArg::Rules => "rules",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        env::temp_dir().join(format!("forge-{label}-{nanos}"))
    }

    #[test]
    fn reconcile_targets_include_custom_mainline_paths() {
        let config = ForgeConfig::default();
        let custom_root = PathBuf::from("/tmp/forge-mainline-custom");
        let state = ForgeState {
            managed_skill_installs: vec![ManagedSkillInstall {
                skill_name: "linear-cli".to_string(),
                managed_by: "forge".to_string(),
                source_kind: SkillSourceKind::Release,
                source_repo_slug: FORGE_REPO_SLUG.to_string(),
                source_ref: "test".to_string(),
                source_hash: "abc".to_string(),
                source_repo_path: None,
                target_kind: SkillTargetKind::Path,
                target_role: SkillTargetRole::Mainline,
                target_root: custom_root.display().to_string(),
                target_path: custom_root.join("linear-cli").display().to_string(),
                installed_at_unix: 0,
            }],
            ..ForgeState::default()
        };

        let targets = mainline_targets_for_reconcile(&config, &state, None).expect("targets");

        assert!(targets.iter().any(|target| {
            target.kind == SkillTargetKind::User && target.role == SkillTargetRole::Mainline
        }));
        assert!(targets.iter().any(|target| {
            target.kind == SkillTargetKind::Path
                && target.role == SkillTargetRole::Mainline
                && target.root == custom_root
        }));
    }

    #[test]
    fn status_target_filter_matches_mainline_path_without_explicit_role() {
        let root = temp_path("status-mainline");
        let skill_root = root.join("linear-cli");
        let skill = load_release_skills()
            .into_iter()
            .find(|skill| skill.name == "linear-cli")
            .expect("linear-cli release skill");
        write_skill_definition(&skill_root, &skill).expect("write skill");

        let state = ForgeState {
            managed_skill_installs: vec![ManagedSkillInstall {
                skill_name: "linear-cli".to_string(),
                managed_by: "forge".to_string(),
                source_kind: SkillSourceKind::Release,
                source_repo_slug: FORGE_REPO_SLUG.to_string(),
                source_ref: "test".to_string(),
                source_hash: hash_skill_files(&skill.files),
                source_repo_path: None,
                target_kind: SkillTargetKind::Path,
                target_role: SkillTargetRole::Mainline,
                target_root: root.display().to_string(),
                target_path: skill_root.display().to_string(),
                installed_at_unix: 0,
            }],
            ..ForgeState::default()
        };
        let config = ForgeConfig::default();

        let result = skills_status_with_source(
            &config,
            &state,
            SkillSourceKind::Release,
            None,
            SkillsStatusScope::Mainline,
            Some(TargetFilter {
                kind: SkillTargetKind::Path,
                role: None,
                root: root.clone(),
            }),
        )
        .expect("status");

        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].target_role, "mainline");
        assert_eq!(
            result.entries[0].target_path,
            skill_root.display().to_string()
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn install_to_explicit_path_target_uses_mainline_role() {
        let install_root = temp_path("user-install");
        fs::create_dir_all(&install_root).expect("create install root");
        let config = ForgeConfig::default();
        let mut state = ForgeState::default();

        let result = skills_install_internal(
            &config,
            &mut state,
            InstallRequest {
                skill_names: Vec::new(),
                all: true,
                source_kind: Some(SkillSourceKind::Release),
                repo_path: None,
                target: Some(format!("path:{}", install_root.display())),
                target_role: Some(SkillTargetRole::Mainline),
                resolved_target: None,
                force: true,
                force_unmanaged: true,
                restrict_to_targets: None,
            },
        )
        .expect("install");

        assert_eq!(result.target_kind, "path");
        assert_eq!(result.target_role, "mainline");
        assert!(!result.installs.is_empty());
        assert!(
            state
                .managed_skill_installs
                .iter()
                .all(|entry| entry.target_role == SkillTargetRole::Mainline
                    && entry.target_root == install_root.display().to_string())
        );

        let _ = fs::remove_dir_all(install_root);
    }

    #[test]
    fn embedded_release_codex_assets_match_repo_sources() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate dir parent")
            .parent()
            .expect("repo root")
            .to_path_buf();
        let repo_assets = load_repo_codex_assets(&repo_root).expect("repo codex assets");
        let release_assets = load_release_codex_assets();

        let repo_map = repo_assets
            .into_iter()
            .map(|asset| (asset.name.clone(), asset))
            .collect::<BTreeMap<_, _>>();
        let release_map = release_assets
            .into_iter()
            .map(|asset| (asset.name.clone(), asset))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(
            repo_map.keys().collect::<Vec<_>>(),
            release_map.keys().collect::<Vec<_>>()
        );

        for (name, repo_asset) in repo_map {
            let release_asset = release_map.get(&name).expect("release codex asset exists");
            assert_eq!(repo_asset.relative_path, release_asset.relative_path);
            assert_eq!(
                repo_asset.contents, release_asset.contents,
                "embedded release payload drifted from repo codex asset for {name}"
            );
        }
    }

    #[test]
    fn codex_install_updates_only_selected_assets() {
        let target_root = temp_path("codex-install");
        let nested_dir = target_root.join("rules");
        fs::create_dir_all(&nested_dir).expect("create rules dir");
        let unrelated = target_root.join("notes.txt");
        fs::write(&unrelated, "keep me").expect("write unrelated file");

        let agents_asset = load_release_codex_assets()
            .into_iter()
            .find(|asset| asset.name == "agents")
            .expect("agents asset");
        let stale_agents = target_root.join(&agents_asset.relative_path);
        fs::write(&stale_agents, "stale").expect("write stale agents");

        let result = codex_install(CodexInstallArgs {
            asset: vec![CodexAssetArg::Agents],
            target: format!("path:{}", target_root.display()),
            source: Some(SkillSourceArg::Release),
            repo_path: None,
        })
        .expect("install codex agents");

        assert_eq!(result.assets.len(), 1);
        assert_eq!(result.assets[0].name, "agents");
        assert_eq!(result.assets[0].status, "updated");
        assert_eq!(
            fs::read(&stale_agents).expect("read installed agents"),
            agents_asset.contents
        );
        assert_eq!(
            fs::read_to_string(&unrelated).expect("read unrelated file"),
            "keep me"
        );
        assert!(!target_root.join(CODEX_RULES_REL_PATH).exists());

        let _ = fs::remove_dir_all(target_root);
    }

    #[test]
    fn codex_diff_reports_missing_and_changed_assets() {
        let target_root = temp_path("codex-diff");
        fs::create_dir_all(target_root.join("rules")).expect("create rules dir");

        let release_assets = load_release_codex_assets();
        let agents_asset = release_assets
            .iter()
            .find(|asset| asset.name == "agents")
            .expect("agents asset");
        let rules_asset = release_assets
            .iter()
            .find(|asset| asset.name == "rules")
            .expect("rules asset");

        fs::write(target_root.join(&agents_asset.relative_path), "stale")
            .expect("write stale agents");

        let result = codex_diff(CodexDiffArgs {
            asset: vec![CodexAssetArg::Agents, CodexAssetArg::Rules],
            target: format!("path:{}", target_root.display()),
            source: Some(SkillSourceArg::Release),
            repo_path: None,
        })
        .expect("diff codex assets");

        assert!(!result.identical);
        let by_name = result
            .assets
            .into_iter()
            .map(|entry| (entry.name.clone(), entry))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(by_name["agents"].status, "changed");
        assert_eq!(by_name["rules"].status, "missing");
        assert_eq!(
            by_name["rules"].source_hash,
            hash_bytes(&rules_asset.contents)
        );

        let _ = fs::remove_dir_all(target_root);
    }

    #[test]
    fn embedded_release_skills_match_repo_skill_directories() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("crate dir parent")
            .parent()
            .expect("repo root")
            .to_path_buf();
        let repo_skills = load_repo_skills(&repo_root).expect("repo skills");
        let release_skills = load_release_skills();

        let repo_map = repo_skills
            .into_iter()
            .map(|skill| (skill.name.clone(), skill))
            .collect::<BTreeMap<_, _>>();
        let release_map = release_skills
            .into_iter()
            .map(|skill| (skill.name.clone(), skill))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(
            repo_map.keys().collect::<Vec<_>>(),
            release_map.keys().collect::<Vec<_>>()
        );

        for (name, repo_skill) in repo_map {
            let release_skill = release_map.get(&name).expect("release skill exists");
            assert_eq!(
                repo_skill.files, release_skill.files,
                "embedded release payload drifted from repo skill for {name}"
            );
        }
    }

    #[test]
    fn doctor_summary_counts_warn_as_not_ready() {
        let checks = vec![
            DoctorCheck {
                id: "cargo".to_string(),
                category: "tool".to_string(),
                status: "pass".to_string(),
                summary: "cargo is available".to_string(),
                detail: None,
                remediation: Vec::new(),
                upgrades: Vec::new(),
            },
            DoctorCheck {
                id: "gh_auth".to_string(),
                category: "auth".to_string(),
                status: "warn".to_string(),
                summary: "GitHub CLI auth could not be confirmed in this non-interactive context"
                    .to_string(),
                detail: None,
                remediation: vec![
                    "Verify interactively in your terminal with `gh auth status`.".to_string(),
                    "If interactive `gh auth status` still fails, run `gh auth login`.".to_string(),
                ],
                upgrades: Vec::new(),
            },
        ];

        let passed = checks.iter().filter(|check| check.status == "pass").count();
        let warnings = checks.iter().filter(|check| check.status == "warn").count();
        let failures = checks.iter().filter(|check| check.status == "fail").count();
        let ready = failures == 0 && warnings == 0;
        let status = if failures > 0 {
            "fail"
        } else if warnings > 0 {
            "warn"
        } else {
            "pass"
        };

        assert_eq!(passed, 1);
        assert_eq!(warnings, 1);
        assert_eq!(failures, 0);
        assert!(!ready);
        assert_eq!(status, "warn");
    }

    #[test]
    fn doctor_windows_remediation_prefers_winget_for_gh_and_git() {
        assert_eq!(
            platform_tool_remediation("windows", "git"),
            vec!["winget install --id Git.Git"]
        );
        assert_eq!(
            platform_tool_remediation("windows", "gh"),
            vec!["winget install --id GitHub.cli"]
        );
    }

    #[test]
    fn doctor_windows_upgrades_prefer_winget_for_gh_and_git() {
        assert_eq!(
            tool_upgrade_commands_for("windows", "git"),
            vec!["winget upgrade --id Git.Git"]
        );
        assert_eq!(
            tool_upgrade_commands_for("windows", "gh"),
            vec!["winget upgrade --id GitHub.cli"]
        );
    }

    #[test]
    fn parse_calver_accepts_expected_shape() {
        assert_eq!(parse_calver("2026.411.2"), Some((20260411, 2)));
        assert_eq!(parse_calver("2026.1012.0"), Some((20261012, 0)));
        assert_eq!(parse_calver("20260412.0.0"), Some((20260412, 0)));
    }

    #[test]
    fn parse_calver_rejects_invalid_values() {
        assert_eq!(parse_calver("main"), None);
        assert_eq!(parse_calver("2026.411"), None);
        assert_eq!(parse_calver("2026.411.two"), None);
        assert_eq!(parse_calver("2026.411.2.extra"), None);
        assert_eq!(parse_calver("202604.0"), None);
        assert_eq!(parse_calver("20260412.0"), None);
    }

    #[test]
    fn latest_release_version_from_lines_uses_highest_calver_tag() {
        let lines = vec![
            "abc\trefs/tags/2026.411.1".to_string(),
            "def\trefs/tags/2026.411.1^{}".to_string(),
            "ghi\trefs/tags/2026.411.2".to_string(),
            "pqr\trefs/tags/20260412.0.0".to_string(),
            "jkl\trefs/tags/not-a-release".to_string(),
            "mno\trefs/tags/2026.411.9".to_string(),
        ];

        assert_eq!(
            latest_release_version_from_lines(&lines),
            Some("20260412.0.0".to_string())
        );
    }

    #[test]
    fn auth_sources_from_dir_reports_inline_and_token_file_sources() {
        let root = temp_path("doctor-auth-sources");
        fs::create_dir_all(&root).expect("create root");
        let configured_token = root.join("configured-token");
        fs::write(&configured_token, "secret\n").expect("write configured token");
        fs::write(
            root.join("config.toml"),
            format!(
                "token = \"abc\"\ntoken_file = \"{}\"\n",
                configured_token.display()
            ),
        )
        .expect("write config");
        fs::write(root.join("token"), "fallback\n").expect("write fallback token");

        let sources = auth_sources_from_dir(
            root.clone(),
            true,
            "LINEAR_API_KEY",
            parse_linear_doctor_config,
        );

        assert!(sources.iter().any(|item| item == "env:LINEAR_API_KEY"));
        assert!(sources.iter().any(|item| item == "config:inline_token"));
        assert!(
            sources
                .iter()
                .any(|item| item == &format!("config:token_file:{}", configured_token.display()))
        );
        assert!(
            sources
                .iter()
                .any(|item| item == &format!("file:{}", root.join("config.toml").display()))
        );
        assert!(
            sources
                .iter()
                .any(|item| item == &format!("file:{}", root.join("token").display()))
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn dedup_strings_removes_repeated_entries() {
        let values = vec![
            "env:LINEAR_API_KEY".to_string(),
            "env:LINEAR_API_KEY".to_string(),
            "file:/tmp/token".to_string(),
            "file:/tmp/token".to_string(),
        ];

        let deduped = dedup_strings(values);

        assert_eq!(deduped.len(), 2);
        assert!(deduped.iter().any(|item| item == "env:LINEAR_API_KEY"));
        assert!(deduped.iter().any(|item| item == "file:/tmp/token"));
    }

    #[test]
    fn top_level_help_documents_human_default_and_json_contract() {
        use clap::CommandFactory;

        let mut cmd = Cli::command();
        let help = cmd.render_long_help().to_string();

        assert!(help.contains("Default output is human-readable."));
        assert!(help.contains("Use --json for compact"));
    }

    #[test]
    fn human_error_output_is_not_json() {
        let rendered = format_error_human(&ErrorBody {
            code: "invalid_usage".to_string(),
            message: "provide a skill name or --all".to_string(),
        });

        assert!(rendered.starts_with("forge error [invalid_usage]"));
        assert!(rendered.contains("provide a skill name or --all"));
        assert!(!rendered.contains("{\"ok\":false"));
    }

    #[test]
    fn human_update_check_output_summarizes_drift() {
        let rendered = format_update_check_human(&UpdateCheckResult {
            source_kind: "repo".to_string(),
            repo_path: Some("/tmp/forge".to_string()),
            cached: false,
            local_head: Some("0123456789abcdef".to_string()),
            remote_head: Some("fedcba9876543210".to_string()),
            remote_default_branch: Some("main".to_string()),
            current_version: None,
            latest_version: None,
            update_available: true,
            checked_at_unix: 123,
            skills_out_of_date: true,
            codex_out_of_date: false,
            skills: vec![
                SkillStatusEntry {
                    name: "linear-cli".to_string(),
                    target_kind: "user".to_string(),
                    target_role: "mainline".to_string(),
                    target_path: "/tmp/skills/linear-cli".to_string(),
                    state: "out_of_date".to_string(),
                    source_kind: "repo".to_string(),
                    source_hash: Some("abc".to_string()),
                    target_hash: Some("def".to_string()),
                },
                SkillStatusEntry {
                    name: "forge-tools".to_string(),
                    target_kind: "user".to_string(),
                    target_role: "mainline".to_string(),
                    target_path: "/tmp/skills/forge-tools".to_string(),
                    state: "up_to_date".to_string(),
                    source_kind: "repo".to_string(),
                    source_hash: Some("abc".to_string()),
                    target_hash: Some("abc".to_string()),
                },
            ],
        });

        assert!(rendered.contains("forge self update-check: updates available"));
        assert!(rendered.contains("local head: 0123456789ab"));
        assert!(rendered.contains("skill details:"));
        assert!(rendered.contains("[OUT_OF_DATE] linear-cli user@mainline -> /tmp/skills/linear-cli"));
    }

    #[test]
    fn human_permissions_output_marks_missing_and_mismatch() {
        let rendered = format_permissions_human(
            "check",
            &PermissionsResult {
                items: vec![
                    PermissionItem {
                        path: "/tmp/config".to_string(),
                        kind: "file".to_string(),
                        exists: false,
                        expected_mode: "0600".to_string(),
                        actual_mode: None,
                        ok: false,
                        changed: false,
                    },
                    PermissionItem {
                        path: "/tmp/state".to_string(),
                        kind: "file".to_string(),
                        exists: true,
                        expected_mode: "0600".to_string(),
                        actual_mode: Some("0644".to_string()),
                        ok: false,
                        changed: false,
                    },
                ],
            },
        );

        assert!(rendered.contains("forge permissions check: 0 ok, 0 fixed, 1 mismatched, 1 missing"));
        assert!(rendered.contains("[MISSING] file expected 0600 /tmp/config"));
        assert!(rendered.contains("[MISMATCH] file expected 0600 actual 0644 /tmp/state"));
    }

    #[test]
    fn human_skills_install_output_summarizes_entries() {
        let rendered = format_skills_install_human(
            "install",
            &SkillsInstallResult {
                source_kind: "repo".to_string(),
                target_kind: "user".to_string(),
                target_role: "mainline".to_string(),
                target_root: "/tmp/skills".to_string(),
                installs: vec![
                    SkillInstallEntry {
                        name: "linear-cli".to_string(),
                        source_hash: "abc".to_string(),
                        target_path: "/tmp/skills/linear-cli".to_string(),
                        status: "installed".to_string(),
                    },
                    SkillInstallEntry {
                        name: "forge-tools".to_string(),
                        source_hash: "def".to_string(),
                        target_path: "/tmp/skills/forge-tools".to_string(),
                        status: "unchanged".to_string(),
                    },
                ],
            },
        );

        assert!(rendered.contains("forge skills install: 2 entries to user@mainline /tmp/skills from repo"));
        assert!(rendered.contains("summary: 1 installed, 1 unchanged"));
        assert!(rendered.contains("[INSTALLED] linear-cli -> /tmp/skills/linear-cli"));
        assert!(rendered.contains("[UNCHANGED] forge-tools -> /tmp/skills/forge-tools"));
    }

    #[test]
    fn human_codex_render_output_includes_rendered_contents() {
        let rendered = format_codex_render_human(&CodexRenderResult {
            source_kind: "repo".to_string(),
            target_kind: "user".to_string(),
            target_root: "/tmp/codex".to_string(),
            assets: vec![CodexRenderEntry {
                name: "agents".to_string(),
                relative_path: "AGENTS.md".to_string(),
                source_path: Some("/tmp/source/AGENTS.md".to_string()),
                target_path: "/tmp/codex/AGENTS.md".to_string(),
                source_hash: "abc123".to_string(),
                contents: "# AGENTS\nbody\n".to_string(),
            }],
        });

        assert!(rendered.contains("forge codex render: 1 assets from repo for user /tmp/codex"));
        assert!(rendered.contains("--- agents AGENTS.md -> /tmp/codex/AGENTS.md [abc123]"));
        assert!(rendered.contains("source: /tmp/source/AGENTS.md"));
        assert!(rendered.contains("# AGENTS\nbody"));
    }
}
