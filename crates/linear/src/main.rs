use std::{env, fmt::Write as _, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use cli_core::{
    ErrorBody, OutputMode, emit_output, format_error_human, normalize_secret, prepare_config_dir,
    print_error_json, prompt_for_secret, resolve_config_dir, resolve_config_file, resolve_token,
    write_token_file,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const API_URL: &str = "https://api.linear.app/graphql";
const ISSUE_FIELDS: &str = r#"
            id
            identifier
            title
            description
            url
            state { id name }
            assignee { id name }
"#;
const PROJECT_MILESTONE_FIELDS: &str = r#"
              id
              name
              description
              targetDate
              sortOrder
              project { id name }
"#;

#[derive(Parser, Debug)]
#[command(name = "linear")]
#[command(about = "Linear GraphQL wrapper for agent-friendly issue workflows")]
#[command(version)]
#[command(
    after_help = "Output:\n  - Default output is human-readable.\n  - Use --json for compact machine-readable JSON.\n  - Errors follow the same rule: human-readable by default, compact JSON with --json."
)]
struct Cli {
    #[arg(long, global = true, help = "Emit compact machine-readable JSON")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Viewer,
    #[command(subcommand)]
    Auth(AuthCommand),
    Config(ConfigArgs),
    Completions(CompletionsArgs),
    #[command(alias = "teams", subcommand)]
    Team(TeamsCommand),
    #[command(subcommand)]
    Project(ProjectCommand),
    #[command(alias = "issues", subcommand)]
    Issue(IssuesCommand),
    #[command(alias = "m", subcommand)]
    Milestone(MilestoneCommand),
}

#[derive(Subcommand, Debug)]
enum TeamsCommand {
    List,
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    Login(AuthLoginArgs),
}

#[derive(Args, Debug)]
struct AuthLoginArgs {
    #[arg(long, help = "Linear API key to save instead of prompting")]
    api_key: Option<String>,
    #[arg(long, help = "Overwrite an existing token file")]
    force: bool,
}

#[derive(Args, Debug)]
struct ConfigArgs {
    #[arg(long, help = "Overwrite an existing config.toml")]
    force: bool,
}

#[derive(Args, Debug)]
struct CompletionsArgs {
    #[arg(value_enum)]
    shell: Shell,
}

#[derive(Subcommand, Debug)]
enum ProjectCommand {
    List(ProjectListArgs),
    View(ProjectViewArgs),
}

#[derive(Subcommand, Debug)]
enum IssuesCommand {
    List(IssuesListArgs),
    Read(IssuesReadArgs),
    Create(IssuesCreateArgs),
    Update(IssuesUpdateArgs),
}

#[derive(Subcommand, Debug)]
enum MilestoneCommand {
    List(MilestoneListArgs),
    View(MilestoneViewArgs),
    Create(MilestoneCreateArgs),
    Update(MilestoneUpdateArgs),
    Delete(MilestoneDeleteArgs),
}

#[derive(Args, Debug)]
struct ProjectListArgs {
    #[arg(
        long,
        default_value_t = 20,
        help = "Maximum number of projects to return"
    )]
    limit: usize,
}

#[derive(Args, Debug)]
struct ProjectViewArgs {
    #[arg(help = "Linear project UUID")]
    project_id: String,
}

#[derive(Args, Debug)]
struct IssuesListArgs {
    #[arg(long, help = "Linear team UUID; falls back to config team_id")]
    team_id: Option<String>,
    #[arg(long, help = "Filter issues assigned to the authenticated user")]
    assigned_to_me: bool,
    #[arg(
        long,
        default_value_t = 20,
        help = "Maximum number of issues to return"
    )]
    limit: usize,
}

#[derive(Args, Debug)]
struct IssuesReadArgs {
    #[arg(help = "Linear issue identifier such as ENG-123")]
    issue_id: String,
}

#[derive(Args, Debug)]
struct IssuesCreateArgs {
    #[arg(long, help = "Linear team UUID; falls back to config team_id")]
    team_id: Option<String>,
    #[arg(long)]
    title: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    description_file: Option<PathBuf>,
    #[arg(long)]
    state_id: Option<String>,
}

#[derive(Args, Debug)]
struct IssuesUpdateArgs {
    #[arg(help = "Linear issue identifier such as ENG-123")]
    issue_id: String,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    description_file: Option<PathBuf>,
    #[arg(long)]
    state_id: Option<String>,
}

#[derive(Args, Debug)]
struct MilestoneListArgs {
    #[arg(long, help = "Linear project UUID")]
    project: String,
    #[arg(
        long,
        default_value_t = 50,
        help = "Maximum number of milestones to return"
    )]
    limit: usize,
}

#[derive(Args, Debug)]
struct MilestoneViewArgs {
    #[arg(help = "Linear project milestone UUID")]
    milestone_id: String,
}

#[derive(Args, Debug)]
struct MilestoneCreateArgs {
    #[arg(long, help = "Linear project UUID")]
    project: String,
    #[arg(long)]
    name: String,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    description_file: Option<PathBuf>,
    #[arg(long, help = "Target date in YYYY-MM-DD format")]
    target_date: Option<String>,
}

#[derive(Args, Debug)]
struct MilestoneUpdateArgs {
    #[arg(help = "Linear project milestone UUID")]
    milestone_id: String,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    description: Option<String>,
    #[arg(long)]
    description_file: Option<PathBuf>,
    #[arg(long, help = "Target date in YYYY-MM-DD format")]
    target_date: Option<String>,
}

#[derive(Args, Debug)]
struct MilestoneDeleteArgs {
    #[arg(help = "Linear project milestone UUID")]
    milestone_id: String,
    #[arg(long, help = "Delete without confirmation")]
    force: bool,
}

#[derive(Debug, Default, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    token_file: Option<String>,
    #[serde(default)]
    team_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Viewer {
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Team {
    id: String,
    key: Option<String>,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "targetDate")]
    target_date: Option<String>,
    #[serde(default, rename = "startDate")]
    start_date: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectMilestone {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "targetDate")]
    target_date: Option<String>,
    #[serde(default, rename = "sortOrder")]
    sort_order: Option<f64>,
    #[serde(default)]
    project: Option<ProjectRef>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectRef {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    id: String,
    identifier: String,
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    state: Option<IssueState>,
    #[serde(default)]
    assignee: Option<IssueAssignee>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueState {
    id: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IssueAssignee {
    id: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct IssuesListResult {
    team_id: String,
    assigned_to_me: bool,
    issues: Vec<Issue>,
}

#[derive(Debug, Serialize)]
struct ProjectsListResult {
    projects: Vec<Project>,
}

#[derive(Debug, Serialize)]
struct ProjectMilestonesListResult {
    project_id: String,
    milestones: Vec<ProjectMilestone>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MutationResult {
    success: bool,
    issue: Option<Issue>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectMilestoneMutationResult {
    success: bool,
    #[serde(default, rename = "projectMilestone")]
    project_milestone: Option<ProjectMilestone>,
}

#[tokio::main]
async fn main() {
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
                    print_error_json(&ErrorBody {
                        code: "invalid_usage".to_string(),
                        message: err.to_string(),
                    });
                }
                _ => {
                    let _ = err.print();
                }
            }
            std::process::exit(exit_code);
        }
    };
    let output = OutputMode::from_json_flag(cli.json);
    let result = run(cli).await;
    if let Err(err) = result {
        let cli_error = classify_error(&err);
        match output {
            OutputMode::Json => print_error_json(&cli_error),
            OutputMode::Human => eprintln!("{}", format_error_human("linear", &cli_error)),
        }
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let output = OutputMode::from_json_flag(cli.json);
    match cli.command {
        Command::Auth(AuthCommand::Login(args)) => {
            let data = login(args)?;
            emit_output(output, data, format_auth_login_human)?;
            return Ok(());
        }
        Command::Config(args) => {
            let data = write_config_template(args.force)?;
            emit_output(output, data, format_config_init_human)?;
            return Ok(());
        }
        Command::Completions(args) => {
            let data = render_completions(args.shell)?;
            emit_output(output, data, format_completions_human)?;
            return Ok(());
        }
        _ => {}
    }

    let config = load_config()?;
    let token = read_token(&config)?;
    let client = linear_client(&token)?;

    match cli.command {
        Command::Viewer => {
            let data = fetch_viewer(&client).await?;
            emit_output(output, data, format_viewer_human)?;
        }
        Command::Auth(_) | Command::Config(_) | Command::Completions(_) => {
            unreachable!("handled above")
        }
        Command::Team(TeamsCommand::List) => {
            let data = fetch_teams(&client).await?;
            emit_output(output, data, |data| format_teams_human(data))?;
        }
        Command::Project(ProjectCommand::List(args)) => {
            let data = list_projects(&client, args.limit).await?;
            emit_output(output, data, format_projects_list_human)?;
        }
        Command::Project(ProjectCommand::View(args)) => {
            let data = read_project(&client, &args.project_id).await?;
            emit_output(output, data, format_project_human)?;
        }
        Command::Issue(IssuesCommand::List(args)) => {
            let team_id = resolve_team_id(args.team_id, &config)?;
            let data = list_issues(&client, &team_id, args.assigned_to_me, args.limit).await?;
            emit_output(output, data, format_issues_list_human)?;
        }
        Command::Issue(IssuesCommand::Read(args)) => {
            let data = read_issue(&client, &args.issue_id).await?;
            emit_output(output, data, |data| format_issue_human("read", data))?;
        }
        Command::Issue(IssuesCommand::Create(args)) => {
            let team_id = resolve_team_id(args.team_id, &config)?;
            let description = resolve_description(args.description, args.description_file)?;
            let data = create_issue(
                &client,
                &team_id,
                &args.title,
                description.as_deref(),
                args.state_id.as_deref(),
            )
            .await?;
            emit_output(output, data, |data| {
                format_issue_mutation_human("create", data)
            })?;
        }
        Command::Issue(IssuesCommand::Update(args)) => {
            let description = resolve_description(args.description, args.description_file)?;
            let data = update_issue(
                &client,
                &args.issue_id,
                args.title.as_deref(),
                description.as_deref(),
                args.state_id.as_deref(),
            )
            .await?;
            emit_output(output, data, |data| {
                format_issue_mutation_human("update", data)
            })?;
        }
        Command::Milestone(MilestoneCommand::List(args)) => {
            let data = list_project_milestones(&client, &args.project, args.limit).await?;
            emit_output(output, data, format_milestones_list_human)?;
        }
        Command::Milestone(MilestoneCommand::View(args)) => {
            let data = read_project_milestone(&client, &args.milestone_id).await?;
            emit_output(output, data, |data| format_milestone_human("view", data))?;
        }
        Command::Milestone(MilestoneCommand::Create(args)) => {
            let description = resolve_description(args.description, args.description_file)?;
            let data = create_project_milestone(
                &client,
                &args.project,
                &args.name,
                description.as_deref(),
                args.target_date.as_deref(),
            )
            .await?;
            emit_output(output, data, |data| {
                format_milestone_mutation_human("create", data)
            })?;
        }
        Command::Milestone(MilestoneCommand::Update(args)) => {
            let description = resolve_description(args.description, args.description_file)?;
            let data = update_project_milestone(
                &client,
                &args.milestone_id,
                args.name.as_deref(),
                description.as_deref(),
                args.target_date.as_deref(),
            )
            .await?;
            emit_output(output, data, |data| {
                format_milestone_mutation_human("update", data)
            })?;
        }
        Command::Milestone(MilestoneCommand::Delete(args)) => {
            let data = delete_project_milestone(&client, &args.milestone_id, args.force).await?;
            emit_output(output, data, format_milestone_delete_human)?;
        }
    }

    Ok(())
}

fn render_completions(shell: Shell) -> Result<CompletionsResult> {
    let mut command = Cli::command();
    let mut buffer = Vec::new();
    generate(shell, &mut command, "linear", &mut buffer);
    let script = String::from_utf8(buffer).context("failed to render completions script")?;
    Ok(CompletionsResult {
        shell: shell.to_string(),
        script,
    })
}

fn format_auth_login_human(result: &AuthLoginResult) -> String {
    format!(
        "linear auth login: wrote token file\npath: {}",
        result.token_file
    )
}

fn format_config_init_human(result: &ConfigInitResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear config: {}",
        if result.created {
            "wrote config template"
        } else {
            "left existing config in place"
        }
    );
    let _ = writeln!(out, "config dir: {}", result.config_dir);
    let _ = writeln!(out, "config file: {}", result.config_file);
    let _ = writeln!(out, "token file: {}", result.token_file);
    out.trim_end().to_string()
}

fn format_completions_human(result: &CompletionsResult) -> String {
    result.script.clone()
}

fn format_viewer_human(viewer: &Viewer) -> String {
    format!(
        "linear viewer: {} <{}>\nid: {}",
        viewer.name, viewer.email, viewer.id
    )
}

fn format_teams_human(teams: &[Team]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "linear team list: {} teams", teams.len());
    for (idx, team) in teams.iter().enumerate() {
        let _ = writeln!(
            out,
            "{}. {} {} [{}]",
            idx + 1,
            team.key.as_deref().unwrap_or("-"),
            team.name,
            team.id
        );
    }
    out.trim_end().to_string()
}

fn format_projects_list_human(result: &ProjectsListResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear project list: {} projects",
        result.projects.len()
    );
    for (idx, project) in result.projects.iter().enumerate() {
        let _ = writeln!(out, "{}. {} [{}]", idx + 1, project.name, project.id);
        let _ = writeln!(
            out,
            "   start: {}  target: {}",
            project.start_date.as_deref().unwrap_or("-"),
            project.target_date.as_deref().unwrap_or("-")
        );
        if let Some(url) = project.url.as_deref() {
            let _ = writeln!(out, "   url: {url}");
        }
        if let Some(description) = project.description.as_deref() {
            let _ = writeln!(out, "   {}", preview_human(description, 160));
        }
    }
    out.trim_end().to_string()
}

fn format_project_human(project: &Project) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear project view: {} [{}]",
        project.name, project.id
    );
    let _ = writeln!(
        out,
        "start: {}  target: {}",
        project.start_date.as_deref().unwrap_or("-"),
        project.target_date.as_deref().unwrap_or("-")
    );
    if let Some(url) = project.url.as_deref() {
        let _ = writeln!(out, "url: {url}");
    }
    if let Some(description) = project.description.as_deref() {
        out.push('\n');
        for line in description.lines() {
            let _ = writeln!(out, "{line}");
        }
    }
    out.trim_end().to_string()
}

fn format_issues_list_human(result: &IssuesListResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear issue list: {} issues for team {}{}",
        result.issues.len(),
        result.team_id,
        if result.assigned_to_me {
            " (assigned to me)"
        } else {
            ""
        }
    );
    for (idx, issue) in result.issues.iter().enumerate() {
        let _ = writeln!(
            out,
            "{}. {} {} [{}] {}",
            idx + 1,
            issue.identifier,
            issue.title,
            issue
                .state
                .as_ref()
                .map(|state| state.name.as_str())
                .unwrap_or("-"),
            issue
                .assignee
                .as_ref()
                .map(|assignee| assignee.name.as_str())
                .unwrap_or("-")
        );
        if let Some(url) = issue.url.as_deref() {
            let _ = writeln!(out, "   url: {url}");
        }
    }
    out.trim_end().to_string()
}

fn format_issue_human(command: &str, issue: &Issue) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear issue {command}: {} {}",
        issue.identifier, issue.title
    );
    let _ = writeln!(out, "id: {}", issue.id);
    let _ = writeln!(
        out,
        "state: {}",
        issue
            .state
            .as_ref()
            .map(|state| state.name.as_str())
            .unwrap_or("-")
    );
    let _ = writeln!(
        out,
        "assignee: {}",
        issue
            .assignee
            .as_ref()
            .map(|assignee| assignee.name.as_str())
            .unwrap_or("-")
    );
    if let Some(url) = issue.url.as_deref() {
        let _ = writeln!(out, "url: {url}");
    }
    if let Some(description) = issue.description.as_deref() {
        out.push('\n');
        for line in description.lines() {
            let _ = writeln!(out, "{line}");
        }
    }
    out.trim_end().to_string()
}

fn format_issue_mutation_human(command: &str, result: &MutationResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear issue {command}: {}",
        if result.success { "success" } else { "failed" }
    );
    if let Some(issue) = result.issue.as_ref() {
        out.push('\n');
        out.push_str(&format_issue_human(command, issue));
    }
    out.trim_end().to_string()
}

fn format_milestones_list_human(result: &ProjectMilestonesListResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear milestone list: {} milestones for project {}",
        result.milestones.len(),
        result.project_id
    );
    for (idx, milestone) in result.milestones.iter().enumerate() {
        let _ = writeln!(out, "{}. {} [{}]", idx + 1, milestone.name, milestone.id);
        let _ = writeln!(
            out,
            "   target: {}  sort: {}",
            milestone.target_date.as_deref().unwrap_or("-"),
            milestone
                .sort_order
                .map(|value| value.to_string())
                .as_deref()
                .unwrap_or("-")
        );
        if let Some(project) = milestone.project.as_ref() {
            let _ = writeln!(out, "   project: {} [{}]", project.name, project.id);
        }
    }
    out.trim_end().to_string()
}

fn format_milestone_human(command: &str, milestone: &ProjectMilestone) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear milestone {command}: {} [{}]",
        milestone.name, milestone.id
    );
    let _ = writeln!(
        out,
        "target: {}  sort: {}",
        milestone.target_date.as_deref().unwrap_or("-"),
        milestone
            .sort_order
            .map(|value| value.to_string())
            .as_deref()
            .unwrap_or("-")
    );
    if let Some(project) = milestone.project.as_ref() {
        let _ = writeln!(out, "project: {} [{}]", project.name, project.id);
    }
    if let Some(description) = milestone.description.as_deref() {
        out.push('\n');
        for line in description.lines() {
            let _ = writeln!(out, "{line}");
        }
    }
    out.trim_end().to_string()
}

fn format_milestone_mutation_human(
    command: &str,
    result: &ProjectMilestoneMutationResult,
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "linear milestone {command}: {}",
        if result.success { "success" } else { "failed" }
    );
    if let Some(milestone) = result.project_milestone.as_ref() {
        out.push('\n');
        out.push_str(&format_milestone_human(command, milestone));
    }
    out.trim_end().to_string()
}

fn format_milestone_delete_human(result: &MilestoneDeleteResult) -> String {
    format!(
        "linear milestone delete: {} [{}]",
        if result.success { "deleted" } else { "failed" },
        result.milestone_id
    )
}

fn preview_human(text: &str, max_chars: usize) -> String {
    let single_line = text.replace('\n', " ").trim().to_string();
    if single_line.chars().count() <= max_chars {
        single_line
    } else {
        let preview = single_line.chars().take(max_chars).collect::<String>();
        format!("{preview}...")
    }
}

fn load_config() -> Result<ConfigFile> {
    let path = config_file_path()?;
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let body = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file at {}", path.display()))?;
    toml::from_str(&body)
        .with_context(|| format!("failed to parse config file at {}", path.display()))
}

#[derive(Debug, Serialize)]
struct ConfigInitResult {
    config_dir: String,
    config_file: String,
    token_file: String,
    created: bool,
}

#[derive(Debug, Serialize)]
struct AuthLoginResult {
    token_file: String,
    created: bool,
}

#[derive(Debug, Serialize)]
struct CompletionsResult {
    shell: String,
    script: String,
}

#[derive(Debug, Serialize)]
struct MilestoneDeleteResult {
    milestone_id: String,
    success: bool,
}

fn write_config_template(force: bool) -> Result<ConfigInitResult> {
    let config_dir = config_dir_path()?;
    let config_file = config_file_path()?;
    let token_file = config_dir.join("token");

    prepare_config_dir(&config_dir)?;

    if config_file.exists() && !force {
        return Ok(ConfigInitResult {
            config_dir: config_dir.display().to_string(),
            config_file: config_file.display().to_string(),
            token_file: token_file.display().to_string(),
            created: false,
        });
    }

    let body = format!(
        "token_file = \"{}\"\n# team_id = \"TEAM_UUID\"\n",
        token_file.display()
    );
    fs::write(&config_file, body)
        .with_context(|| format!("failed to write config file at {}", config_file.display()))?;
    cli_core::ensure_owner_only_permissions(&config_file, false)?;

    Ok(ConfigInitResult {
        config_dir: config_dir.display().to_string(),
        config_file: config_file.display().to_string(),
        token_file: token_file.display().to_string(),
        created: true,
    })
}

fn login(args: AuthLoginArgs) -> Result<AuthLoginResult> {
    let config_dir = config_dir_path()?;
    prepare_config_dir(&config_dir)?;

    let api_key = match args.api_key {
        Some(key) => key,
        None => prompt_for_api_key()?,
    };
    let api_key = normalize_secret(api_key, "empty API key")?;
    let token_file = write_token_file(&config_dir, &api_key, args.force)?;

    Ok(AuthLoginResult {
        token_file: token_file.display().to_string(),
        created: true,
    })
}

fn prompt_for_api_key() -> Result<String> {
    prompt_for_secret(
        "Paste Linear API key: ",
        "failed to read API key from stdin",
    )
}

fn read_token(config: &ConfigFile) -> Result<String> {
    resolve_token(
        "LINEAR_API_KEY",
        "FORGE_LINEAR_CLI_CONFIG_DIR",
        "linear",
        false,
        config.token.as_deref(),
        config.token_file.as_deref(),
        "missing Linear auth; set LINEAR_API_KEY or create ~/.config/forge/linear/config.toml or ~/.config/forge/linear/token",
    )
}

fn resolve_team_id(team_id: Option<String>, config: &ConfigFile) -> Result<String> {
    match team_id.or_else(|| config.team_id.clone()) {
        Some(id) if !id.trim().is_empty() => Ok(id),
        _ => bail!(
            "missing team_id; pass --team-id or configure team_id in ~/.config/forge/linear/config.toml"
        ),
    }
}

fn resolve_description(
    description: Option<String>,
    description_file: Option<PathBuf>,
) -> Result<Option<String>> {
    match (description, description_file) {
        (Some(_), Some(_)) => bail!("pass either --description or --description-file, not both"),
        (Some(description), None) => Ok(Some(description)),
        (None, Some(path)) => {
            let body = fs::read_to_string(&path).with_context(|| {
                format!("failed to read description file at {}", path.display())
            })?;
            Ok(Some(body))
        }
        (None, None) => Ok(None),
    }
}

fn linear_client(token: &str) -> Result<Client> {
    Client::builder()
        .user_agent("forge/linear")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                token
                    .parse()
                    .context("invalid Linear API key for Authorization header")?,
            );
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/json".parse().expect("static header"),
            );
            headers
        })
        .build()
        .context("failed to build HTTP client")
}

async fn fetch_viewer(client: &Client) -> Result<Viewer> {
    let query = r#"query Viewer { viewer { id name email } }"#;
    let data = graphql(client, query, None).await?;
    serde_json::from_value(data["viewer"].clone()).context("failed to decode viewer")
}

async fn fetch_teams(client: &Client) -> Result<Vec<Team>> {
    let query = r#"query Teams { teams { nodes { id key name } } }"#;
    let data = graphql(client, query, None).await?;
    serde_json::from_value(data["teams"]["nodes"].clone()).context("failed to decode teams")
}

async fn list_projects(client: &Client, limit: usize) -> Result<ProjectsListResult> {
    let query = r#"
        query Projects($limit: Int!) {
          projects(first: $limit) {
            nodes {
              id
              name
              description
              targetDate
              startDate
              url
            }
          }
        }
    "#;
    let variables = json!({ "limit": limit as i64 });
    let data = graphql(client, query, Some(variables)).await?;
    let projects = serde_json::from_value(data["projects"]["nodes"].clone())
        .context("failed to decode projects list")?;
    Ok(ProjectsListResult { projects })
}

async fn read_project(client: &Client, project_id: &str) -> Result<Project> {
    let query = r#"
        query Project($id: String!) {
          project(id: $id) {
            id
            name
            description
            targetDate
            startDate
            url
          }
        }
    "#;
    let variables = json!({ "id": project_id });
    let data = graphql(client, query, Some(variables)).await?;
    serde_json::from_value(data["project"].clone()).context("failed to decode project")
}

async fn list_issues(
    client: &Client,
    team_id: &str,
    assigned_to_me: bool,
    limit: usize,
) -> Result<IssuesListResult> {
    let query = r#"
        query TeamIssues($teamId: String!, $limit: Int!) {
          team(id: $teamId) {
            issues(first: $limit) {
              nodes {
__ISSUE_FIELDS__
              }
            }
          }
        }
    "#
    .replace("__ISSUE_FIELDS__", ISSUE_FIELDS);
    let fetch_limit = if assigned_to_me {
        (limit.saturating_mul(5)).max(limit)
    } else {
        limit
    };
    let variables = json!({
        "teamId": team_id,
        "limit": fetch_limit as i64,
    });
    let data = graphql(client, &query, Some(variables)).await?;
    let mut issues: Vec<Issue> = serde_json::from_value(data["team"]["issues"]["nodes"].clone())
        .context("failed to decode issues list")?;
    if assigned_to_me {
        let viewer = fetch_viewer(client).await?;
        issues.retain(|issue| {
            issue
                .assignee
                .as_ref()
                .map(|assignee| assignee.id == viewer.id)
                .unwrap_or(false)
        });
        issues.truncate(limit);
    }
    Ok(IssuesListResult {
        team_id: team_id.to_string(),
        assigned_to_me,
        issues,
    })
}

async fn read_issue(client: &Client, issue_id: &str) -> Result<Issue> {
    let query = r#"
        query Issue($id: String!) {
          issue(id: $id) {
__ISSUE_FIELDS__
          }
        }
    "#
    .replace("__ISSUE_FIELDS__", ISSUE_FIELDS);
    let variables = json!({ "id": issue_id });
    let data = graphql(client, &query, Some(variables)).await?;
    serde_json::from_value(data["issue"].clone()).context("failed to decode issue")
}

async fn create_issue(
    client: &Client,
    team_id: &str,
    title: &str,
    description: Option<&str>,
    state_id: Option<&str>,
) -> Result<MutationResult> {
    let query = r#"
        mutation IssueCreate($input: IssueCreateInput!) {
          issueCreate(input: $input) {
            success
            issue {
__ISSUE_FIELDS__
            }
          }
        }
    "#
    .replace("__ISSUE_FIELDS__", ISSUE_FIELDS);
    let mut input = json!({
        "teamId": team_id,
        "title": title,
    });
    if let Some(description) = description {
        input["description"] = Value::String(description.to_string());
    }
    if let Some(state_id) = state_id {
        input["stateId"] = Value::String(state_id.to_string());
    }
    let data = graphql(client, &query, Some(json!({ "input": input }))).await?;
    serde_json::from_value(data["issueCreate"].clone()).context("failed to decode issueCreate")
}

async fn update_issue(
    client: &Client,
    issue_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    state_id: Option<&str>,
) -> Result<MutationResult> {
    if title.is_none() && description.is_none() && state_id.is_none() {
        bail!(
            "issues update requires at least one of --title, --description/--description-file, or --state-id"
        );
    }
    let query = r#"
        mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
          issueUpdate(id: $id, input: $input) {
            success
            issue {
__ISSUE_FIELDS__
            }
          }
        }
    "#
    .replace("__ISSUE_FIELDS__", ISSUE_FIELDS);
    let mut input = json!({});
    if let Some(title) = title {
        input["title"] = Value::String(title.to_string());
    }
    if let Some(description) = description {
        input["description"] = Value::String(description.to_string());
    }
    if let Some(state_id) = state_id {
        input["stateId"] = Value::String(state_id.to_string());
    }
    let data = graphql(
        client,
        &query,
        Some(json!({ "id": issue_id, "input": input })),
    )
    .await?;
    serde_json::from_value(data["issueUpdate"].clone()).context("failed to decode issueUpdate")
}

async fn list_project_milestones(
    client: &Client,
    project_id: &str,
    limit: usize,
) -> Result<ProjectMilestonesListResult> {
    let query = r#"
        query ProjectMilestones($projectId: String!, $limit: Int!) {
          project(id: $projectId) {
            projectMilestones(first: $limit) {
              nodes {
__PROJECT_MILESTONE_FIELDS__
              }
            }
          }
        }
    "#
    .replace("__PROJECT_MILESTONE_FIELDS__", PROJECT_MILESTONE_FIELDS);
    let variables = json!({
        "projectId": project_id,
        "limit": limit as i64,
    });
    let data = graphql(client, &query, Some(variables)).await?;
    let milestones = serde_json::from_value(data["project"]["projectMilestones"]["nodes"].clone())
        .context("failed to decode project milestones list")?;
    Ok(ProjectMilestonesListResult {
        project_id: project_id.to_string(),
        milestones,
    })
}

async fn read_project_milestone(client: &Client, milestone_id: &str) -> Result<ProjectMilestone> {
    let query = r#"
        query ProjectMilestone($id: String!) {
          projectMilestone(id: $id) {
__PROJECT_MILESTONE_FIELDS__
          }
        }
    "#
    .replace("__PROJECT_MILESTONE_FIELDS__", PROJECT_MILESTONE_FIELDS);
    let variables = json!({ "id": milestone_id });
    let data = graphql(client, &query, Some(variables)).await?;
    serde_json::from_value(data["projectMilestone"].clone())
        .context("failed to decode project milestone")
}

async fn create_project_milestone(
    client: &Client,
    project_id: &str,
    name: &str,
    description: Option<&str>,
    target_date: Option<&str>,
) -> Result<ProjectMilestoneMutationResult> {
    let query = r#"
        mutation ProjectMilestoneCreate($input: ProjectMilestoneCreateInput!) {
          projectMilestoneCreate(input: $input) {
            success
            projectMilestone {
__PROJECT_MILESTONE_FIELDS__
            }
          }
        }
    "#
    .replace("__PROJECT_MILESTONE_FIELDS__", PROJECT_MILESTONE_FIELDS);
    let mut input = json!({
        "projectId": project_id,
        "name": name,
    });
    if let Some(description) = description {
        input["description"] = Value::String(description.to_string());
    }
    if let Some(target_date) = target_date {
        input["targetDate"] = Value::String(target_date.to_string());
    }
    let data = graphql(client, &query, Some(json!({ "input": input }))).await?;
    serde_json::from_value(data["projectMilestoneCreate"].clone())
        .context("failed to decode projectMilestoneCreate")
}

async fn update_project_milestone(
    client: &Client,
    milestone_id: &str,
    name: Option<&str>,
    description: Option<&str>,
    target_date: Option<&str>,
) -> Result<ProjectMilestoneMutationResult> {
    if name.is_none() && description.is_none() && target_date.is_none() {
        bail!(
            "milestone update requires at least one of --name, --description/--description-file, or --target-date"
        );
    }
    let query = r#"
        mutation ProjectMilestoneUpdate($id: String!, $input: ProjectMilestoneUpdateInput!) {
          projectMilestoneUpdate(id: $id, input: $input) {
            success
            projectMilestone {
__PROJECT_MILESTONE_FIELDS__
            }
          }
        }
    "#
    .replace("__PROJECT_MILESTONE_FIELDS__", PROJECT_MILESTONE_FIELDS);
    let mut input = json!({});
    if let Some(name) = name {
        input["name"] = Value::String(name.to_string());
    }
    if let Some(description) = description {
        input["description"] = Value::String(description.to_string());
    }
    if let Some(target_date) = target_date {
        input["targetDate"] = Value::String(target_date.to_string());
    }
    let data = graphql(
        client,
        &query,
        Some(json!({ "id": milestone_id, "input": input })),
    )
    .await?;
    serde_json::from_value(data["projectMilestoneUpdate"].clone())
        .context("failed to decode projectMilestoneUpdate")
}

async fn delete_project_milestone(
    client: &Client,
    milestone_id: &str,
    force: bool,
) -> Result<MilestoneDeleteResult> {
    if !force {
        bail!("milestone delete requires --force");
    }
    let query = r#"
        mutation ProjectMilestoneDelete($id: String!) {
          projectMilestoneDelete(id: $id) {
            success
          }
        }
    "#;
    let data = graphql(client, query, Some(json!({ "id": milestone_id }))).await?;
    Ok(MilestoneDeleteResult {
        milestone_id: milestone_id.to_string(),
        success: data["projectMilestoneDelete"]["success"]
            .as_bool()
            .unwrap_or(false),
    })
}

async fn graphql(client: &Client, query: &str, variables: Option<Value>) -> Result<Value> {
    let mut body = json!({ "query": query });
    if let Some(variables) = variables {
        body["variables"] = variables;
    }
    let response = client
        .post(API_URL)
        .json(&body)
        .send()
        .await
        .context("failed to call Linear GraphQL API")?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .context("failed to decode Linear response body")?;

    if !status.is_success() {
        bail!("Linear API request failed with HTTP {}", status);
    }
    if let Some(errors) = body.get("errors") {
        let message = errors
            .as_array()
            .and_then(|errors| errors.first())
            .and_then(|error| error.get("message"))
            .and_then(Value::as_str)
            .unwrap_or("linear_graphql_error");
        bail!(message.to_string());
    }
    body.get("data")
        .cloned()
        .ok_or_else(|| anyhow!("Linear response missing data"))
}

fn config_dir_path() -> Result<PathBuf> {
    resolve_config_dir("FORGE_LINEAR_CLI_CONFIG_DIR", "linear", false)
}

fn config_file_path() -> Result<PathBuf> {
    resolve_config_file("FORGE_LINEAR_CLI_CONFIG_DIR", "linear", false)
}

fn classify_error(error: &anyhow::Error) -> ErrorBody {
    let message = error.to_string();
    let code = match message.as_str() {
        msg if msg.contains("missing Linear auth") => "auth_missing",
        "Invalid auth token" | "invalid_authentication" => "auth_error",
        msg if msg.contains("missing team_id") => "validation_error",
        msg if msg.contains("requires at least one") => "validation_error",
        msg if msg.contains("milestone delete requires --force") => "validation_error",
        msg if msg.contains("pass either --description or --description-file") => {
            "validation_error"
        }
        msg if msg.contains("token file already exists") => "validation_error",
        msg if msg.contains("empty API key") => "validation_error",
        _ => "internal_error",
    };
    ErrorBody {
        code: code.to_string(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_help_documents_output_contract() {
        let mut cmd = Cli::command();
        let help = cmd.render_long_help().to_string();

        assert!(help.contains("Default output is human-readable."));
        assert!(help.contains("Use --json for compact machine-readable JSON."));
    }

    #[test]
    fn viewer_human_output_is_readable() {
        let rendered = format_viewer_human(&Viewer {
            id: "usr_123".to_string(),
            name: "Ian Cleary".to_string(),
            email: "ian@example.com".to_string(),
        });

        assert!(rendered.contains("linear viewer: Ian Cleary <ian@example.com>"));
        assert!(rendered.contains("id: usr_123"));
    }

    #[test]
    fn issues_list_human_output_lists_issues() {
        let rendered = format_issues_list_human(&IssuesListResult {
            team_id: "team_123".to_string(),
            assigned_to_me: true,
            issues: vec![Issue {
                id: "iss_123".to_string(),
                identifier: "ENG-123".to_string(),
                title: "Ship output contract".to_string(),
                description: None,
                url: Some("https://linear.app/issue/ENG-123".to_string()),
                state: Some(IssueState {
                    id: "state_1".to_string(),
                    name: "In Progress".to_string(),
                }),
                assignee: Some(IssueAssignee {
                    id: "usr_123".to_string(),
                    name: "Ian".to_string(),
                }),
            }],
        });

        assert!(
            rendered.contains("linear issue list: 1 issues for team team_123 (assigned to me)")
        );
        assert!(rendered.contains("1. ENG-123 Ship output contract [In Progress] Ian"));
        assert!(rendered.contains("url: https://linear.app/issue/ENG-123"));
    }

    #[test]
    fn milestone_delete_human_output_is_readable() {
        let rendered = format_milestone_delete_human(&MilestoneDeleteResult {
            milestone_id: "mil_123".to_string(),
            success: true,
        });

        assert_eq!(rendered, "linear milestone delete: deleted [mil_123]");
    }

    #[test]
    fn human_error_output_is_not_json() {
        let rendered = format_error_human(
            "linear",
            &ErrorBody {
                code: "validation_error".to_string(),
                message: "missing team_id".to_string(),
            },
        );

        assert!(rendered.starts_with("linear error [validation_error]"));
        assert!(rendered.contains("missing team_id"));
        assert!(!rendered.contains("{\"ok\":false"));
    }
}
