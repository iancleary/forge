use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const API_URL: &str = "https://api.linear.app/graphql";

#[derive(Parser, Debug)]
#[command(name = "linear")]
#[command(about = "Linear GraphQL wrapper for agent-friendly issue workflows")]
#[command(version)]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON")]
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
    #[arg(long, default_value_t = 20, help = "Maximum number of projects to return")]
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
    #[arg(long, default_value_t = 20, help = "Maximum number of issues to return")]
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
    #[arg(long, default_value_t = 50, help = "Maximum number of milestones to return")]
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
    let cli = Cli::parse();
    let result = run(cli).await;
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

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Auth(AuthCommand::Login(args)) => {
            let data = login(args)?;
            print_json(&Envelope { ok: true, data })?;
            return Ok(());
        }
        Command::Config(args) => {
            let data = write_config_template(args.force)?;
            print_json(&Envelope { ok: true, data })?;
            return Ok(());
        }
        Command::Completions(args) => {
            print_completions(args.shell);
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
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Auth(_) | Command::Config(_) | Command::Completions(_) => {
            unreachable!("handled above")
        }
        Command::Team(TeamsCommand::List) => {
            let data = fetch_teams(&client).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Project(ProjectCommand::List(args)) => {
            let data = list_projects(&client, args.limit).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Project(ProjectCommand::View(args)) => {
            let data = read_project(&client, &args.project_id).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Issue(IssuesCommand::List(args)) => {
            let team_id = resolve_team_id(args.team_id, &config)?;
            let data = list_issues(&client, &team_id, args.assigned_to_me, args.limit).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Issue(IssuesCommand::Read(args)) => {
            let data = read_issue(&client, &args.issue_id).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Issue(IssuesCommand::Create(args)) => {
            let team_id = resolve_team_id(args.team_id, &config)?;
            let description = resolve_description(args.description, args.description_file)?;
            let data = create_issue(&client, &team_id, &args.title, description.as_deref(), args.state_id.as_deref()).await?;
            print_json(&Envelope { ok: true, data })?;
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
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Milestone(MilestoneCommand::List(args)) => {
            let data = list_project_milestones(&client, &args.project, args.limit).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Milestone(MilestoneCommand::View(args)) => {
            let data = read_project_milestone(&client, &args.milestone_id).await?;
            print_json(&Envelope { ok: true, data })?;
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
            print_json(&Envelope { ok: true, data })?;
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
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Milestone(MilestoneCommand::Delete(args)) => {
            let data = delete_project_milestone(&client, &args.milestone_id, args.force).await?;
            print_json(&Envelope { ok: true, data })?;
        }
    }

    Ok(())
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

fn print_completions(shell: Shell) {
    let mut command = Cli::command();
    generate(shell, &mut command, "linear", &mut std::io::stdout());
}

fn load_config() -> Result<ConfigFile> {
    let path = config_file_path()?;
    if !path.exists() {
        let legacy_path = legacy_config_file_path()?;
        if legacy_path.exists() {
            let body = fs::read_to_string(&legacy_path)
                .with_context(|| format!("failed to read config file at {}", legacy_path.display()))?;
            return toml::from_str(&body)
                .with_context(|| format!("failed to parse config file at {}", legacy_path.display()));
        }
        return Ok(ConfigFile::default());
    }
    let body = fs::read_to_string(&path)
        .with_context(|| format!("failed to read config file at {}", path.display()))?;
    toml::from_str(&body).with_context(|| format!("failed to parse config file at {}", path.display()))
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

fn write_config_template(force: bool) -> Result<ConfigInitResult> {
    let config_dir = config_dir_path()?;
    let config_file = config_file_path()?;
    let token_file = config_dir.join("token");

    fs::create_dir_all(&config_dir)
        .with_context(|| format!("failed to create config dir at {}", config_dir.display()))?;
    ensure_owner_only_permissions(&config_dir, true)?;

    if config_file.exists() && !force {
        return Ok(ConfigInitResult {
            config_dir: config_dir.display().to_string(),
            config_file: config_file.display().to_string(),
            token_file: token_file.display().to_string(),
            created: false,
        });
    }

    let body = format!(
        "token_file = \"{}\"\n# team_id = \"YOUR_TEAM_ID\"\n",
        token_file.display()
    );
    fs::write(&config_file, body)
        .with_context(|| format!("failed to write config file at {}", config_file.display()))?;
    ensure_owner_only_permissions(&config_file, false)?;

    Ok(ConfigInitResult {
        config_dir: config_dir.display().to_string(),
        config_file: config_file.display().to_string(),
        token_file: token_file.display().to_string(),
        created: true,
    })
}

fn login(args: AuthLoginArgs) -> Result<AuthLoginResult> {
    let config_dir = config_dir_path()?;
    let token_file = config_dir.join("token");

    fs::create_dir_all(&config_dir)
        .with_context(|| format!("failed to create config dir at {}", config_dir.display()))?;
    ensure_owner_only_permissions(&config_dir, true)?;

    if token_file.exists() && !args.force {
        bail!("token file already exists; rerun with --force to overwrite");
    }

    let api_key = match args.api_key {
        Some(key) => key,
        None => prompt_for_api_key()?,
    };
    let api_key = api_key.trim().to_string();
    if api_key.is_empty() {
        bail!("empty API key");
    }

    fs::write(&token_file, format!("{api_key}\n"))
        .with_context(|| format!("failed to write token file at {}", token_file.display()))?;
    ensure_owner_only_permissions(&token_file, false)?;

    Ok(AuthLoginResult {
        token_file: token_file.display().to_string(),
        created: true,
    })
}

fn prompt_for_api_key() -> Result<String> {
    let mut stdout = io::stdout();
    write!(stdout, "Paste Linear API key: ").context("failed to write auth prompt")?;
    stdout.flush().context("failed to flush auth prompt")?;

    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .context("failed to read API key from stdin")?;
    Ok(buffer)
}

fn read_token(config: &ConfigFile) -> Result<String> {
    if let Ok(token) = env::var("LINEAR_API_KEY") {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    if let Some(token) = config.token.as_ref() {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    if let Some(token_file) = config.token_file.as_ref() {
        let path = expand_path(token_file);
        let token = fs::read_to_string(&path)
            .with_context(|| format!("failed to read token file at {}", path.display()))?;
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    let token_path = config_dir_path()?.join("token");
    if token_path.exists() {
        let token = fs::read_to_string(&token_path)
            .with_context(|| format!("failed to read token file at {}", token_path.display()))?;
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    let legacy_token_path = legacy_config_dir_path()?.join("token");
    if legacy_token_path.exists() {
        let token = fs::read_to_string(&legacy_token_path)
            .with_context(|| format!("failed to read token file at {}", legacy_token_path.display()))?;
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }
    bail!("missing Linear auth; set LINEAR_API_KEY or create ~/.config/forge/linear/config.toml or ~/.config/forge/linear/token")
}

fn resolve_team_id(team_id: Option<String>, config: &ConfigFile) -> Result<String> {
    match team_id.or_else(|| config.team_id.clone()) {
        Some(id) if !id.trim().is_empty() => Ok(id),
        _ => bail!("missing team_id; pass --team-id or configure team_id in ~/.config/forge/linear/config.toml"),
    }
}

fn resolve_description(description: Option<String>, description_file: Option<PathBuf>) -> Result<Option<String>> {
    match (description, description_file) {
        (Some(_), Some(_)) => bail!("pass either --description or --description-file, not both"),
        (Some(description), None) => Ok(Some(description)),
        (None, Some(path)) => {
            let body = fs::read_to_string(&path)
                .with_context(|| format!("failed to read description file at {}", path.display()))?;
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
                token.parse().context("invalid Linear API key for Authorization header")?,
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

async fn list_issues(client: &Client, team_id: &str, assigned_to_me: bool, limit: usize) -> Result<IssuesListResult> {
    let query = r#"
        query TeamIssues($teamId: String!, $limit: Int!) {
          team(id: $teamId) {
            issues(first: $limit) {
              nodes {
                id
                identifier
                title
                description
                url
                state { id name }
                assignee { id name }
              }
            }
          }
        }
    "#;
    let fetch_limit = if assigned_to_me {
        (limit.saturating_mul(5)).max(limit)
    } else {
        limit
    };
    let variables = json!({
        "teamId": team_id,
        "limit": fetch_limit as i64,
    });
    let data = graphql(client, query, Some(variables)).await?;
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
            id
            identifier
            title
            description
            url
            state { id name }
            assignee { id name }
          }
        }
    "#;
    let variables = json!({ "id": issue_id });
    let data = graphql(client, query, Some(variables)).await?;
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
              id
              identifier
              title
              description
              url
              state { id name }
              assignee { id name }
            }
          }
        }
    "#;
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
    let data = graphql(client, query, Some(json!({ "input": input }))).await?;
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
        bail!("issues update requires at least one of --title, --description/--description-file, or --state-id");
    }
    let query = r#"
        mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
          issueUpdate(id: $id, input: $input) {
            success
            issue {
              id
              identifier
              title
              description
              url
              state { id name }
              assignee { id name }
            }
          }
        }
    "#;
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
    let data = graphql(client, query, Some(json!({ "id": issue_id, "input": input }))).await?;
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
                id
                name
                description
                targetDate
                sortOrder
                project { id name }
              }
            }
          }
        }
    "#;
    let variables = json!({
        "projectId": project_id,
        "limit": limit as i64,
    });
    let data = graphql(client, query, Some(variables)).await?;
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
            id
            name
            description
            targetDate
            sortOrder
            project { id name }
          }
        }
    "#;
    let variables = json!({ "id": milestone_id });
    let data = graphql(client, query, Some(variables)).await?;
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
              id
              name
              description
              targetDate
              sortOrder
              project { id name }
            }
          }
        }
    "#;
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
    let data = graphql(client, query, Some(json!({ "input": input }))).await?;
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
              id
              name
              description
              targetDate
              sortOrder
              project { id name }
            }
          }
        }
    "#;
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
        query,
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
) -> Result<Value> {
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
    Ok(data["projectMilestoneDelete"].clone())
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
    let body: Value = response.json().await.context("failed to decode Linear response body")?;

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
    if let Ok(path) = env::var("FORGE_LINEAR_CLI_CONFIG_DIR") {
        return Ok(expand_path(&path));
    }
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("forge").join("linear"));
    }
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("forge")
        .join("linear"))
}

fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join("config.toml"))
}

fn legacy_config_dir_path() -> Result<PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("forge").join("linear-cli"));
    }
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("forge")
        .join("linear-cli"))
}

fn legacy_config_file_path() -> Result<PathBuf> {
    Ok(legacy_config_dir_path()?.join("config.toml"))
}

fn ensure_owner_only_permissions(path: &PathBuf, is_dir: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = if is_dir { 0o700 } else { 0o600 };
        fs::set_permissions(path, PermissionsExt::from_mode(mode))
            .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        let _ = is_dir;
    }

    Ok(())
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

fn classify_error(error: &anyhow::Error) -> ErrorBody {
    let message = error.to_string();
    let code = match message.as_str() {
        msg if msg.contains("missing Linear auth") => "auth_missing",
        "Invalid auth token" | "invalid_authentication" => "auth_error",
        msg if msg.contains("missing team_id") => "validation_error",
        msg if msg.contains("requires at least one") => "validation_error",
        msg if msg.contains("milestone delete requires --force") => "validation_error",
        msg if msg.contains("pass either --description or --description-file") => "validation_error",
        msg if msg.contains("token file already exists") => "validation_error",
        msg if msg.contains("empty API key") => "validation_error",
        _ => "internal_error",
    };
    ErrorBody {
        code: code.to_string(),
        message,
    }
}
