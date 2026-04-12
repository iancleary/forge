use std::{
    env, fs,
    fmt::Write as _,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

pub const SLACK_API_BASE: &str = "https://slack.com/api";

#[derive(Debug, Default, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    token_file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SlackErrorResponse {
    error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

impl OutputMode {
    pub fn from_json_flag(json: bool) -> Self {
        if json { Self::Json } else { Self::Human }
    }
}

#[derive(Debug, Serialize)]
pub struct Envelope<T>
where
    T: Serialize,
{
    ok: bool,
    data: T,
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    ok: bool,
    error: ErrorBody,
}

#[derive(Debug, Serialize, Clone)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
}

pub fn read_token(
    env_var: &str,
    config_dir_env_var: &str,
    config_subdir: &str,
    use_forge_config_dir: bool,
    missing_message: &str,
) -> Result<String> {
    if let Ok(token) = env::var(env_var) {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    let config_path =
        config_file_path(config_dir_env_var, config_subdir, use_forge_config_dir)?;
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

    let token_path = config_dir_path(config_dir_env_var, config_subdir, use_forge_config_dir)?
        .join("token");
    if token_path.exists() {
        let token = fs::read_to_string(&token_path)
            .with_context(|| format!("failed to read token file at {}", token_path.display()))?;
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    bail!(missing_message.to_string())
}

pub fn config_dir_path(
    config_dir_env_var: &str,
    config_subdir: &str,
    use_forge_config_dir: bool,
) -> Result<PathBuf> {
    if let Ok(path) = env::var(config_dir_env_var) {
        return Ok(expand_path(&path));
    }
    if use_forge_config_dir {
        if let Ok(path) = env::var("FORGE_CONFIG_DIR") {
            return Ok(expand_path(&path).join(config_subdir));
        }
    }
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("forge").join(config_subdir));
    }

    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("forge")
        .join(config_subdir))
}

pub fn config_file_path(
    config_dir_env_var: &str,
    config_subdir: &str,
    use_forge_config_dir: bool,
) -> Result<PathBuf> {
    Ok(config_dir_path(config_dir_env_var, config_subdir, use_forge_config_dir)?.join("config.toml"))
}

pub fn prepare_config_dir(config_dir: &Path) -> Result<()> {
    fs::create_dir_all(config_dir)
        .with_context(|| format!("failed to create config dir at {}", config_dir.display()))?;
    ensure_owner_only_permissions(config_dir, true)?;
    Ok(())
}

pub fn write_token_file(config_dir: &Path, token: &str, force: bool) -> Result<PathBuf> {
    let token_file = config_dir.join("token");
    if token_file.exists() && !force {
        bail!("token file already exists; rerun with --force to overwrite");
    }
    fs::write(&token_file, format!("{token}\n"))
        .with_context(|| format!("failed to write token file at {}", token_file.display()))?;
    ensure_owner_only_permissions(&token_file, false)?;
    Ok(token_file)
}

pub fn prompt_for_token(prompt: &str) -> Result<String> {
    let mut stdout = io::stdout();
    write!(stdout, "{prompt}").context("failed to write auth prompt")?;
    stdout.flush().context("failed to flush auth prompt")?;

    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .context("failed to read Slack token from stdin")?;
    Ok(buffer)
}

pub fn normalize_token(token: String) -> Result<String> {
    let token = token.trim().to_string();
    if token.is_empty() {
        bail!("empty Slack token");
    }
    Ok(token)
}

pub fn ensure_owner_only_permissions(path: &Path, is_dir: bool) -> Result<()> {
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

pub fn expand_path(path: &str) -> PathBuf {
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

pub fn slack_client(token: &str, user_agent: &str) -> Result<Client> {
    Client::builder()
        .user_agent(user_agent)
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

pub fn print_json<T>(value: &T) -> Result<()>
where
    T: Serialize,
{
    println!(
        "{}",
        serde_json::to_string(value).context("failed to render JSON output")?
    );
    Ok(())
}

pub fn emit_output<T, F>(mode: OutputMode, data: T, human: F) -> Result<()>
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

pub fn print_human_text(text: &str) {
    if text.ends_with('\n') {
        print!("{text}");
    } else {
        println!("{text}");
    }
}

pub fn format_error_human(tool_name: &str, error: &ErrorBody) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{tool_name} error [{}]", error.code);
    for line in error.message.lines() {
        let _ = writeln!(out, "{line}");
    }
    out.trim_end().to_string()
}

pub fn print_error_json(error: &ErrorBody) {
    eprintln!(
        "{}",
        serde_json::to_string(&ErrorEnvelope {
            ok: false,
            error: error.clone(),
        })
        .unwrap_or_else(|_| {
            "{\"ok\":false,\"error\":{\"code\":\"internal_error\",\"message\":\"failed to serialize error\"}}".to_string()
        })
    );
}

pub async fn parse_slack_json_response<T>(response: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let body = response.text().await.context("failed to read Slack response body")?;

    if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        if let Some(retry_after) = retry_after {
            bail!(format!("ratelimited: retry after {retry_after} seconds"));
        }
        bail!("ratelimited");
    }

    if !status.is_success() {
        if let Ok(error) = serde_json::from_str::<SlackErrorResponse>(&body) {
            let message = error.error.unwrap_or_else(|| format!("HTTP {status}"));
            bail!(message);
        }
        bail!(format!("Slack API request failed with HTTP {status}"));
    }

    let payload = serde_json::from_str::<T>(&body).context("failed to decode Slack response body")?;
    Ok(payload)
}

pub async fn slack_get<T, Q>(client: &Client, method: &str, query: &Q) -> Result<T>
where
    T: DeserializeOwned,
    Q: Serialize + ?Sized,
{
    let response = client
        .get(format!("{SLACK_API_BASE}/{method}"))
        .query(query)
        .send()
        .await
        .with_context(|| format!("failed to call {method}"))?;
    parse_slack_api_response(response, method).await
}

pub async fn slack_post_form<T, F>(client: &Client, method: &str, form: &F) -> Result<T>
where
    T: DeserializeOwned,
    F: Serialize + ?Sized,
{
    let response = client
        .post(format!("{SLACK_API_BASE}/{method}"))
        .form(form)
        .send()
        .await
        .with_context(|| format!("failed to call {method}"))?;
    parse_slack_api_response(response, method).await
}

pub async fn slack_post_json<T, B>(client: &Client, method: &str, body: &B) -> Result<T>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let response = client
        .post(format!("{SLACK_API_BASE}/{method}"))
        .json(body)
        .send()
        .await
        .with_context(|| format!("failed to call {method}"))?;
    parse_slack_api_response(response, method).await
}

async fn parse_slack_api_response<T>(response: reqwest::Response, method: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = parse_slack_json_response::<Value>(response).await?;
    if value.get("ok").and_then(Value::as_bool) != Some(true) {
        let message = value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("slack_api_error")
            .to_string();
        bail!(message);
    }

    serde_json::from_value(value).with_context(|| format!("failed to decode {method} payload"))
}

pub fn classify_slack_error_code(message: &str) -> Option<&'static str> {
    match message {
        "invalid_auth" | "not_authed" | "token_revoked" | "token_expired" => Some("auth_error"),
        "missing_scope" | "no_permission" | "not_in_channel" => Some("access_denied"),
        "channel_not_found" | "file_not_found" | "user_not_found" => Some("not_found"),
        msg if msg.starts_with("ratelimited") => Some("rate_limited"),
        _ => None,
    }
}
