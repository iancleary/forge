use std::{
    env, fs,
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

const API_BASE: &str = "https://slack.com/api";

#[derive(Parser, Debug)]
#[command(name = "slack-cli")]
#[command(about = "Slack utility CLI for deterministic research and retrieval workflows")]
struct Cli {
    #[arg(long, global = true, help = "Emit machine-readable JSON")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[command(subcommand)]
    Auth(AuthCommand),
    ResolvePermalink(ResolvePermalinkArgs),
    ReadThread(ReadThreadArgs),
    #[command(name = "channel-context")]
    ChannelContext(ContextArgs),
    ThreadContext(ThreadContextArgs),
    Search(SearchArgs),
}

#[derive(Subcommand, Debug)]
enum AuthCommand {
    Login(AuthLoginArgs),
}

#[derive(Args, Debug)]
struct AuthLoginArgs {
    #[arg(long, help = "Slack API token to save instead of prompting")]
    token: Option<String>,
    #[arg(long, help = "Overwrite an existing token file")]
    force: bool,
}

#[derive(Args, Debug)]
struct ResolvePermalinkArgs {
    #[arg(help = "Slack message permalink")]
    permalink: String,
}

#[derive(Args, Debug)]
struct ReadThreadArgs {
    #[arg(help = "Slack channel ID, such as C123")]
    channel_id: String,
    #[arg(help = "Thread root timestamp")]
    thread_ts: String,
    #[arg(long, default_value_t = 15, help = "Maximum number of thread messages to return")]
    limit: u32,
}

#[derive(Args, Debug)]
struct ContextArgs {
    #[arg(help = "Slack channel ID, such as C123")]
    channel_id: String,
    #[arg(help = "Target message timestamp in the parent channel timeline")]
    message_ts: String,
    #[arg(long, default_value_t = 5, help = "Number of earlier top-level channel messages")]
    before: u32,
    #[arg(long, default_value_t = 5, help = "Number of later top-level channel messages")]
    after: u32,
}

#[derive(Args, Debug)]
struct ThreadContextArgs {
    #[arg(help = "Slack channel ID, such as C123")]
    channel_id: String,
    #[arg(help = "Thread root timestamp")]
    thread_ts: String,
    #[arg(help = "Target message timestamp inside the thread")]
    message_ts: String,
    #[arg(long, default_value_t = 5, help = "Number of earlier thread messages")]
    before: u32,
    #[arg(long, default_value_t = 5, help = "Number of later thread messages")]
    after: u32,
}

#[derive(Args, Debug)]
struct SearchArgs {
    #[arg(help = "Slack search query")]
    query: String,
    #[arg(long, default_value_t = 20, help = "Maximum number of matches to request")]
    limit: u32,
    #[arg(long, default_value_t = 1, help = "Search results page number")]
    page: u32,
    #[arg(long, default_value = "timestamp", help = "Sort key, usually timestamp or score")]
    sort: String,
    #[arg(long, default_value = "desc", help = "Sort direction: asc or desc")]
    sort_dir: String,
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

#[derive(Debug, Serialize)]
struct AuthLoginResult {
    token_file: String,
    created: bool,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    token_file: Option<String>,
}

#[derive(Debug, Serialize)]
struct ResolvedPermalink {
    team_domain: String,
    channel_id: String,
    message_ts: String,
    thread_ts: String,
    is_thread_root: bool,
    reply_count: Option<u32>,
}

#[derive(Debug, Serialize)]
struct ThreadResult {
    channel_id: String,
    thread_ts: String,
    messages: Vec<Message>,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Serialize)]
struct ContextResult {
    channel_id: String,
    target: Message,
    before: Vec<Message>,
    after: Vec<Message>,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Serialize)]
struct ResponseMetadata {
    rate_limited: bool,
    next_cursor: Option<String>,
}

#[derive(Debug, Serialize)]
struct SearchResult {
    query: String,
    messages: Vec<SearchMatch>,
    response_metadata: ResponseMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchMatch {
    #[serde(default)]
    iid: Option<String>,
    #[serde(default)]
    team: Option<String>,
    #[serde(default)]
    channel: Option<SearchChannel>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    username: Option<String>,
    #[serde(default)]
    text: String,
    ts: String,
    #[serde(default)]
    permalink: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchChannel {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    user: Option<String>,
    #[serde(default)]
    text: String,
    ts: String,
    #[serde(default)]
    thread_ts: Option<String>,
    #[serde(default)]
    reply_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SlackListResponse<T> {
    ok: bool,
    error: Option<String>,
    messages: Option<Vec<T>>,
    response_metadata: Option<SlackCursor>,
}

#[derive(Debug, Deserialize)]
struct SlackSearchResponse {
    ok: bool,
    error: Option<String>,
    messages: Option<SlackSearchMessages>,
}

#[derive(Debug, Deserialize)]
struct SlackSearchMessages {
    matches: Vec<SearchMatch>,
    #[serde(default)]
    pagination: Option<SlackPagination>,
}

#[derive(Debug, Deserialize)]
struct SlackPagination {
    #[serde(default)]
    page: Option<u32>,
    #[serde(default)]
    page_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct SlackCursor {
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackErrorResponse {
    error: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = run(cli).await;
    if let Err(err) = result {
        let cli_error = classify_error(&err);
        eprintln!(
            "{}",
            serde_json::to_string(&ErrorEnvelope {
                ok: false,
                error: cli_error,
            })
            .unwrap_or_else(|_| "{\"ok\":false,\"error\":{\"code\":\"internal_error\",\"message\":\"failed to serialize error\"}}".to_string())
        );
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Auth(AuthCommand::Login(args)) => {
            let data = login(args)?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::ResolvePermalink(args) => {
            let data = resolve_permalink(&args.permalink)?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::ReadThread(args) => {
            let token = read_token()?;
            let client = slack_client(&token)?;
            let data = read_thread(&client, &args.channel_id, &args.thread_ts, args.limit).await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::ChannelContext(args) => {
            let token = read_token()?;
            let client = slack_client(&token)?;
            let data = read_channel_context(
                &client,
                &args.channel_id,
                &args.message_ts,
                args.before,
                args.after,
            )
            .await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::ThreadContext(args) => {
            let token = read_token()?;
            let client = slack_client(&token)?;
            let data = read_thread_context(
                &client,
                &args.channel_id,
                &args.thread_ts,
                &args.message_ts,
                args.before,
                args.after,
            )
            .await?;
            print_json(&Envelope { ok: true, data })?;
        }
        Command::Search(args) => {
            let token = read_token()?;
            let client = slack_client(&token)?;
            let data = search_messages(
                &client,
                &args.query,
                args.limit,
                args.page,
                &args.sort,
                &args.sort_dir,
            )
            .await?;
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
        serde_json::to_string(value).context("failed to render JSON output")?
    );
    Ok(())
}

fn read_token() -> Result<String> {
    if let Ok(token) = env::var("SLACK_API_TOKEN") {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    let config_path = config_file_path()?;
    if config_path.exists() {
        let contents = fs::read_to_string(&config_path).with_context(|| {
            format!("failed to read config file at {}", config_path.display())
        })?;
        let config: ConfigFile = toml::from_str(&contents).with_context(|| {
            format!("failed to parse config file at {}", config_path.display())
        })?;

        if let Some(token) = config.token {
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Ok(token);
            }
        }

        if let Some(token_file) = config.token_file {
            let token_path = expand_path(&token_file);
            let token = fs::read_to_string(&token_path).with_context(|| {
                format!("failed to read token file at {}", token_path.display())
            })?;
            let token = token.trim().to_string();
            if !token.is_empty() {
                return Ok(token);
            }
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

    bail!(
        "missing Slack auth; set SLACK_API_TOKEN or create ~/.config/forge/slack-cli/config.toml or ~/.config/forge/slack-cli/token"
    )
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

    let token = match args.token {
        Some(token) => token,
        None => prompt_for_token()?,
    };
    let token = token.trim().to_string();
    if token.is_empty() {
        bail!("empty Slack token");
    }

    fs::write(&token_file, format!("{token}\n"))
        .with_context(|| format!("failed to write token file at {}", token_file.display()))?;
    ensure_owner_only_permissions(&token_file, false)?;

    Ok(AuthLoginResult {
        token_file: token_file.display().to_string(),
        created: true,
    })
}

fn prompt_for_token() -> Result<String> {
    let mut stdout = io::stdout();
    write!(stdout, "Paste Slack API token: ").context("failed to write auth prompt")?;
    stdout.flush().context("failed to flush auth prompt")?;

    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .context("failed to read Slack token from stdin")?;
    Ok(buffer)
}

fn config_dir_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("FORGE_SLACK_CLI_CONFIG_DIR") {
        let path = expand_path(&path);
        return Ok(path);
    }

    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("forge").join("slack-cli"));
    }

    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("forge")
        .join("slack-cli"))
}

fn config_file_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join("config.toml"))
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

fn slack_client(token: &str) -> Result<Client> {
    Client::builder()
        .user_agent("forge/slack-cli")
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {token}")
                    .parse()
                    .context("invalid token for Authorization header")?,
            );
            headers.insert(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=utf-8"
                    .parse()
                    .expect("static header"),
            );
            headers
        })
        .build()
        .context("failed to build HTTP client")
}

fn resolve_permalink(permalink: &str) -> Result<ResolvedPermalink> {
    let url = Url::parse(permalink).context("invalid Slack permalink")?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("Slack permalink is missing a host"))?;
    let mut segments = url
        .path_segments()
        .ok_or_else(|| anyhow!("Slack permalink path is invalid"))?;

    let archives = segments.next().ok_or_else(|| anyhow!("missing archives segment"))?;
    if archives != "archives" {
        bail!("unsupported Slack permalink path");
    }

    let channel_id = segments
        .next()
        .ok_or_else(|| anyhow!("missing channel id in permalink"))?
        .to_string();
    let raw_message = segments
        .next()
        .ok_or_else(|| anyhow!("missing message id in permalink"))?;
    let message_ts = parse_permalink_ts(raw_message)?;
    let thread_ts = url
        .query_pairs()
        .find(|(key, _)| key == "thread_ts")
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| message_ts.clone());
    let is_thread_root = thread_ts == message_ts;

    Ok(ResolvedPermalink {
        team_domain: host.to_string(),
        channel_id,
        message_ts,
        thread_ts,
        is_thread_root,
        reply_count: None,
    })
}

fn parse_permalink_ts(raw: &str) -> Result<String> {
    let trimmed = raw
        .strip_prefix('p')
        .ok_or_else(|| anyhow!("message segment must start with 'p'"))?;
    if trimmed.len() < 7 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
        bail!("message segment must be numeric after 'p'");
    }
    let split_at = trimmed.len() - 6;
    Ok(format!("{}.{}", &trimmed[..split_at], &trimmed[split_at..]))
}

async fn read_thread(
    client: &Client,
    channel_id: &str,
    thread_ts: &str,
    limit: u32,
) -> Result<ThreadResult> {
    let response = client
        .get(format!("{API_BASE}/conversations.replies"))
        .query(&[
            ("channel", channel_id),
            ("ts", thread_ts),
            ("limit", &limit.to_string()),
            ("inclusive", "true"),
        ])
        .send()
        .await
        .context("failed to call conversations.replies")?;

    let rate_limited = response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS;
    let payload = parse_list_response(response).await?;
    let messages = payload.messages.unwrap_or_default();

    Ok(ThreadResult {
        channel_id: channel_id.to_string(),
        thread_ts: thread_ts.to_string(),
        messages,
        response_metadata: ResponseMetadata {
            rate_limited,
            next_cursor: payload.response_metadata.and_then(|m| m.next_cursor),
        },
    })
}

async fn read_channel_context(
    client: &Client,
    channel_id: &str,
    message_ts: &str,
    before: u32,
    after: u32,
) -> Result<ContextResult> {
    let target_window = fetch_history(
        client,
        channel_id,
        Some(message_ts),
        Some(message_ts),
        true,
        1,
    )
    .await?;
    let before_window = if before > 0 {
        fetch_history(
            client,
            channel_id,
            None,
            Some(message_ts),
            false,
            before,
        )
        .await?
    } else {
        HistoryWindow::default()
    };
    let after_window = if after > 0 {
        fetch_history(
            client,
            channel_id,
            Some(message_ts),
            None,
            false,
            after,
        )
        .await?
    } else {
        HistoryWindow::default()
    };

    let target = target_window
        .messages
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("failed to isolate target message"))?;
    let mut before_messages = before_window.messages;
    before_messages.sort_by(|left, right| left.ts.cmp(&right.ts));
    let mut after_messages = after_window.messages;
    after_messages.sort_by(|left, right| left.ts.cmp(&right.ts));

    Ok(ContextResult {
        channel_id: channel_id.to_string(),
        target,
        before: before_messages,
        after: after_messages,
        response_metadata: ResponseMetadata {
            rate_limited: target_window.rate_limited
                || before_window.rate_limited
                || after_window.rate_limited,
            next_cursor: after_window
                .next_cursor
                .or(before_window.next_cursor)
                .or(target_window.next_cursor),
        },
    })
}

async fn read_thread_context(
    client: &Client,
    channel_id: &str,
    thread_ts: &str,
    message_ts: &str,
    before: u32,
    after: u32,
) -> Result<ContextResult> {
    let thread = read_thread(client, channel_id, thread_ts, before + after + 50).await?;
    let target_index = thread
        .messages
        .iter()
        .position(|message| message.ts == message_ts)
        .ok_or_else(|| anyhow!("target message was not returned in thread"))?;

    let start = target_index.saturating_sub(before as usize);
    let end = (target_index + after as usize + 1).min(thread.messages.len());

    let target = thread
        .messages
        .get(target_index)
        .cloned()
        .ok_or_else(|| anyhow!("failed to isolate target thread message"))?;
    let before_messages = thread.messages[start..target_index].to_vec();
    let after_messages = thread.messages[target_index + 1..end].to_vec();

    Ok(ContextResult {
        channel_id: channel_id.to_string(),
        target,
        before: before_messages,
        after: after_messages,
        response_metadata: thread.response_metadata,
    })
}

#[derive(Default)]
struct HistoryWindow {
    messages: Vec<Message>,
    rate_limited: bool,
    next_cursor: Option<String>,
}

async fn fetch_history(
    client: &Client,
    channel_id: &str,
    oldest: Option<&str>,
    latest: Option<&str>,
    inclusive: bool,
    limit: u32,
) -> Result<HistoryWindow> {
    let mut query = vec![
        ("channel".to_string(), channel_id.to_string()),
        ("inclusive".to_string(), inclusive.to_string()),
        ("limit".to_string(), limit.to_string()),
    ];
    if let Some(oldest) = oldest {
        query.push(("oldest".to_string(), oldest.to_string()));
    }
    if let Some(latest) = latest {
        query.push(("latest".to_string(), latest.to_string()));
    }

    let response = client
        .get(format!("{API_BASE}/conversations.history"))
        .query(&query)
        .send()
        .await
        .context("failed to call conversations.history")?;

    let rate_limited = response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS;
    let payload = parse_list_response(response).await?;

    Ok(HistoryWindow {
        messages: payload.messages.unwrap_or_default(),
        rate_limited,
        next_cursor: payload.response_metadata.and_then(|m| m.next_cursor),
    })
}

async fn parse_list_response(response: reqwest::Response) -> Result<SlackListResponse<Message>> {
    let status = response.status();
    let body = response.text().await.context("failed to read Slack response body")?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_str::<SlackErrorResponse>(&body) {
            let message = error.error.unwrap_or_else(|| format!("HTTP {status}"));
            bail!(message);
        }
        bail!("Slack API request failed with HTTP {status}");
    }

    let payload = serde_json::from_str::<SlackListResponse<Message>>(&body)
        .context("failed to decode Slack response body")?;
    if !payload.ok {
        bail!(payload.error.unwrap_or_else(|| "slack_api_error".to_string()));
    }

    Ok(payload)
}

async fn search_messages(
    client: &Client,
    query: &str,
    limit: u32,
    page: u32,
    sort: &str,
    sort_dir: &str,
) -> Result<SearchResult> {
    let response = client
        .get(format!("{API_BASE}/search.messages"))
        .query(&[
            ("query", query),
            ("count", &limit.to_string()),
            ("page", &page.to_string()),
            ("sort", sort),
            ("sort_dir", sort_dir),
        ])
        .send()
        .await
        .context("failed to call search.messages")?;

    let rate_limited = response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS;
    let status = response.status();
    let body = response.text().await.context("failed to read Slack response body")?;

    if !status.is_success() {
        if let Ok(error) = serde_json::from_str::<SlackErrorResponse>(&body) {
            let message = error.error.unwrap_or_else(|| format!("HTTP {status}"));
            bail!(message);
        }
        bail!("Slack API request failed with HTTP {status}");
    }

    let payload = serde_json::from_str::<SlackSearchResponse>(&body)
        .context("failed to decode Slack search response body")?;
    if !payload.ok {
        bail!(payload.error.unwrap_or_else(|| "slack_api_error".to_string()));
    }

    let messages = payload.messages.unwrap_or(SlackSearchMessages {
        matches: Vec::new(),
        pagination: None,
    });
    let next_cursor = messages.pagination.and_then(|pagination| {
        match (pagination.page, pagination.page_count) {
            (Some(current), Some(total)) if current < total => Some((current + 1).to_string()),
            _ => None,
        }
    });

    Ok(SearchResult {
        query: query.to_string(),
        messages: messages.matches,
        response_metadata: ResponseMetadata {
            rate_limited,
            next_cursor,
        },
    })
}

fn classify_error(error: &anyhow::Error) -> ErrorBody {
    let message = error.to_string();
    let code = match message.as_str() {
        msg if msg.contains("missing Slack auth") => "auth_missing",
        "invalid_auth" | "not_authed" | "token_revoked" | "token_expired" => "auth_error",
        "missing_scope" | "no_permission" | "not_in_channel" => "access_denied",
        "channel_not_found" => "not_found",
        "ratelimited" => "rate_limited",
        "invalid_query" => "validation_error",
        msg if msg.contains("token file already exists") => "validation_error",
        msg if msg.contains("empty Slack token") => "validation_error",
        _ => "internal_error",
    };

    ErrorBody {
        code: code.to_string(),
        message,
    }
}
