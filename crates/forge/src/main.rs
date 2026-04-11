use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command as ProcessCommand,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(name = "forge")]
#[command(about = "Forge manager CLI")]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(name = "self", subcommand)]
    Self_(SelfCommand),
}

#[derive(Subcommand, Debug)]
enum SelfCommand {
    #[command(name = "update-check")]
    UpdateCheck(UpdateCheckArgs),
    Update(UpdateArgs),
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
    #[arg(long, help = "Remote branch to update from; defaults to the remote default branch when known")]
    branch: Option<String>,
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
}

#[derive(Debug, Serialize)]
struct UpdateCheckResult {
    repo_path: String,
    cached: bool,
    local_head: Option<String>,
    remote_head: Option<String>,
    remote_default_branch: Option<String>,
    update_available: bool,
    checked_at_unix: u64,
}

#[derive(Debug, Serialize)]
struct UpdateResult {
    repo_path: String,
    branch: String,
    before_head: String,
    after_head: String,
    changed: bool,
}

fn main() {
    let cli = Cli::parse();
    let result = run(cli);

    if let Err(err) = result {
        let error = ErrorEnvelope {
            ok: false,
            error: classify_error(&err),
        };
        eprintln!(
            "{}",
            serde_json::to_string_pretty(&error).unwrap_or_else(|_| {
                "{\"ok\":false,\"error\":{\"code\":\"internal_error\",\"message\":\"failed to serialize error\"}}".to_string()
            })
        );
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Self_(SelfCommand::UpdateCheck(args)) => {
            let data = update_check(args)?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Self_(SelfCommand::Update(args)) => {
            let data = update(args)?;
            print_json(&Envelope { ok: true, data })?;
        }
    }

    Ok(())
}

fn update_check(args: UpdateCheckArgs) -> Result<UpdateCheckResult> {
    let config = load_config()?;
    let _auto_check_updates = config.auto_check_updates.unwrap_or(true);
    let _auto_update = config.auto_update.unwrap_or(false);
    let repo_path = resolve_repo_path(args.repo_path, &config)?;
    let checked_at_unix = now_unix()?;
    let state_path = state_file_path()?;
    let ttl_seconds = config.update_check_ttl_minutes.unwrap_or(1_440) * 60;

    if !args.force && state_path.exists() {
        if let Ok(state) = load_state(&state_path) {
            if let Some(last_checked) = state.last_checked_unix {
                if checked_at_unix.saturating_sub(last_checked) < ttl_seconds {
                    if let Some(update_available) = state.update_available {
                        return Ok(UpdateCheckResult {
                            repo_path: repo_path.display().to_string(),
                            cached: true,
                            local_head: state.local_head,
                            remote_head: state.remote_head,
                            remote_default_branch: state.remote_default_branch,
                            update_available,
                            checked_at_unix: last_checked,
                        });
                    }
                }
            }
        }
    }

    ensure_git_repo(&repo_path)?;
    let local_head = git_stdout(&repo_path, &["rev-parse", "HEAD"])?;
    let (remote_default_branch, remote_head) = remote_default_branch_and_head(&repo_path)?;
    let update_available = match remote_head.as_ref() {
        Some(remote_head) => remote_head != &local_head,
        None => false,
    };

    let state = ForgeState {
        last_checked_unix: Some(checked_at_unix),
        repo_path: Some(repo_path.display().to_string()),
        local_head: Some(local_head.clone()),
        remote_head: remote_head.clone(),
        remote_default_branch: remote_default_branch.clone(),
        update_available: Some(update_available),
    };
    let _ = save_state(&state_path, &state);

    Ok(UpdateCheckResult {
        repo_path: repo_path.display().to_string(),
        cached: false,
        local_head: Some(local_head),
        remote_head,
        remote_default_branch,
        update_available,
        checked_at_unix,
    })
}

fn update(args: UpdateArgs) -> Result<UpdateResult> {
    let config = load_config()?;
    let repo_path = resolve_repo_path(args.repo_path, &config)?;
    ensure_git_repo(&repo_path)?;

    let before_head = git_stdout(&repo_path, &["rev-parse", "HEAD"])?;
    let branch = match args.branch {
        Some(branch) => branch,
        None => remote_default_branch_and_head(&repo_path)?
            .0
            .unwrap_or_else(|| "main".to_string()),
    };

    run_git(&repo_path, &["pull", "--rebase", "origin", &branch])?;
    let after_head = git_stdout(&repo_path, &["rev-parse", "HEAD"])?;

    Ok(UpdateResult {
        repo_path: repo_path.display().to_string(),
        branch,
        before_head: before_head.clone(),
        after_head: after_head.clone(),
        changed: before_head != after_head,
    })
}

fn print_json<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("failed to render JSON output")?
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
    toml::from_str(&body).with_context(|| format!("failed to parse config file at {}", path.display()))
}

fn load_state(path: &Path) -> Result<ForgeState> {
    let body = fs::read_to_string(path)
        .with_context(|| format!("failed to read state file at {}", path.display()))?;
    toml::from_str(&body).with_context(|| format!("failed to parse state file at {}", path.display()))
}

fn save_state(path: &Path, state: &ForgeState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let body = toml::to_string_pretty(state).context("failed to serialize state file")?;
    fs::write(path, body).with_context(|| format!("failed to write {}", path.display()))
}

fn resolve_repo_path(cli_repo_path: Option<PathBuf>, config: &ForgeConfig) -> Result<PathBuf> {
    if let Some(path) = cli_repo_path {
        return Ok(path);
    }
    if let Some(path) = config.repo_path.as_ref() {
        return Ok(expand_path(path));
    }
    env::current_dir().context("failed to resolve current working directory")
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

fn classify_error(error: &anyhow::Error) -> ErrorBody {
    let message = error.to_string();
    let code = match message.as_str() {
        msg if msg.contains("not a git repository") => "not_git_repo",
        msg if msg.contains("failed to run git") => "git_unavailable",
        msg if msg.contains("git pull --rebase") => "update_failed",
        msg if msg.contains("failed to read config file") || msg.contains("failed to parse config file") => {
            "config_error"
        }
        _ => "internal_error",
    };

    ErrorBody {
        code: code.to_string(),
        message,
    }
}
