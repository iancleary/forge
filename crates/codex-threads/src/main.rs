use std::{
    cmp::Reverse,
    collections::HashMap,
    env,
    fmt::Write as _,
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow};
use clap::{Args, Parser, Subcommand};
use cli_core::{ErrorBody, OutputMode, emit_output, format_error_human, print_error_json};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(name = "codex-threads")]
#[command(about = "Search and read local Codex thread archives")]
#[command(
    after_help = "Output:\n  - Default output is human-readable.\n  - Use --json for compact machine-readable JSON.\n  - Errors follow the same rule: human-readable by default, compact JSON with --json."
)]
struct Cli {
    #[arg(long, global = true, help = "Emit compact machine-readable JSON")]
    json: bool,
    #[arg(
        long,
        global = true,
        help = "Override the Codex home directory; defaults to ~/.codex or CODEX_HOME"
    )]
    codex_home: Option<PathBuf>,
    #[arg(
        long,
        global = true,
        help = "Override the local index path; defaults to ~/.config/forge/codex-threads/index.json"
    )]
    index_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Sync,
    #[command(subcommand)]
    Messages(MessagesCommand),
    #[command(subcommand)]
    Threads(ThreadsCommand),
    #[command(subcommand)]
    Events(EventsCommand),
}

#[derive(Subcommand, Debug)]
enum MessagesCommand {
    Search(MessagesSearchArgs),
}

#[derive(Subcommand, Debug)]
enum ThreadsCommand {
    Resolve(ThreadsResolveArgs),
    Read(ThreadsReadArgs),
}

#[derive(Subcommand, Debug)]
enum EventsCommand {
    Read(EventsReadArgs),
}

#[derive(Args, Debug)]
struct MessagesSearchArgs {
    query: String,
    #[arg(long, default_value_t = 20)]
    limit: usize,
}

#[derive(Args, Debug)]
struct ThreadsResolveArgs {
    query: String,
    #[arg(long, default_value_t = 10)]
    limit: usize,
}

#[derive(Args, Debug)]
struct ThreadsReadArgs {
    session_id: String,
}

#[derive(Args, Debug)]
struct EventsReadArgs {
    session_id: String,
    #[arg(long, default_value_t = 50)]
    limit: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThreadIndex {
    generated_at: String,
    threads: Vec<IndexedThread>,
    messages: Vec<IndexedMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct IndexedThread {
    id: String,
    thread_name: String,
    updated_at: Option<String>,
    session_path: String,
    cwd: Option<String>,
    model: Option<String>,
    cli_version: Option<String>,
    message_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct IndexedMessage {
    session_id: String,
    thread_name: String,
    timestamp: Option<String>,
    role: String,
    text: String,
    preview: String,
    source_type: String,
}

#[derive(Debug, Serialize)]
struct SyncResult {
    session_count: usize,
    message_count: usize,
    index_path: String,
}

#[derive(Debug, Serialize)]
struct MessageSearchResult {
    query: String,
    matches: Vec<MessageMatch>,
}

#[derive(Debug, Serialize)]
struct MessageMatch {
    session_id: String,
    thread_name: String,
    timestamp: Option<String>,
    role: String,
    preview: String,
    text: String,
}

#[derive(Debug, Serialize)]
struct ThreadsResolveResult {
    query: String,
    matches: Vec<ThreadMatch>,
}

#[derive(Debug, Serialize)]
struct ThreadMatch {
    id: String,
    thread_name: String,
    updated_at: Option<String>,
    cwd: Option<String>,
    model: Option<String>,
    message_count: usize,
    matched_preview: Option<String>,
}

#[derive(Debug, Serialize)]
struct ThreadReadResult {
    thread: IndexedThread,
    messages: Vec<IndexedMessage>,
}

#[derive(Debug, Serialize)]
struct EventsReadResult {
    session_id: String,
    events: Vec<NormalizedEvent>,
}

#[derive(Debug, Serialize)]
struct NormalizedEvent {
    timestamp: Option<String>,
    event_type: String,
    role: Option<String>,
    text: Option<String>,
    raw_type: String,
}

#[derive(Debug, Deserialize, Clone)]
struct SessionIndexEntry {
    id: String,
    thread_name: Option<String>,
    updated_at: Option<String>,
}

#[derive(Debug, Default)]
struct SessionBuild {
    id: String,
    thread_name: String,
    updated_at: Option<String>,
    session_path: String,
    cwd: Option<String>,
    model: Option<String>,
    cli_version: Option<String>,
    messages: Vec<IndexedMessage>,
    seen_messages: HashMap<String, (usize, String)>,
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
    let result = run(cli);
    if let Err(err) = result {
        let cli_error = classify_error(&err);
        match output {
            OutputMode::Json => print_error_json(&cli_error),
            OutputMode::Human => eprintln!("{}", format_error_human("codex-threads", &cli_error)),
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let paths = Paths::from_cli(&cli)?;
    let output = OutputMode::from_json_flag(cli.json);
    match cli.command {
        Command::Sync => {
            let data = sync_index(&paths)?;
            emit_output(output, data, format_sync_human)?;
        }
        Command::Messages(MessagesCommand::Search(args)) => {
            let index = load_index(&paths)?;
            let data = search_messages(&index, &args.query, args.limit);
            emit_output(output, data, format_message_search_human)?;
        }
        Command::Threads(ThreadsCommand::Resolve(args)) => {
            let index = load_index(&paths)?;
            let data = resolve_threads(&index, &args.query, args.limit);
            emit_output(output, data, format_threads_resolve_human)?;
        }
        Command::Threads(ThreadsCommand::Read(args)) => {
            let index = load_index(&paths)?;
            let data = read_thread(&index, &args.session_id)?;
            emit_output(output, data, format_thread_read_human)?;
        }
        Command::Events(EventsCommand::Read(args)) => {
            let data = read_events(&paths, &args.session_id, args.limit)?;
            emit_output(output, data, format_events_read_human)?;
        }
    }
    Ok(())
}

fn format_sync_human(result: &SyncResult) -> String {
    format!(
        "codex-threads sync: indexed {} sessions and {} messages\nindex: {}",
        result.session_count, result.message_count, result.index_path
    )
}

fn format_message_search_human(result: &MessageSearchResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "codex-threads messages search: {} matches for {:?}",
        result.matches.len(),
        result.query
    );
    for (idx, message) in result.matches.iter().enumerate() {
        let _ = writeln!(
            out,
            "{}. {} [{}] {} {}",
            idx + 1,
            message.thread_name,
            message.session_id,
            message.timestamp.as_deref().unwrap_or("-"),
            message.role
        );
        let _ = writeln!(out, "   {}", message.preview);
    }
    out.trim_end().to_string()
}

fn format_threads_resolve_human(result: &ThreadsResolveResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "codex-threads threads resolve: {} matches for {:?}",
        result.matches.len(),
        result.query
    );
    for (idx, thread) in result.matches.iter().enumerate() {
        let _ = writeln!(out, "{}. {} [{}]", idx + 1, thread.thread_name, thread.id);
        let _ = writeln!(
            out,
            "   updated: {}  model: {}  messages: {}",
            thread.updated_at.as_deref().unwrap_or("-"),
            thread.model.as_deref().unwrap_or("-"),
            thread.message_count
        );
        if let Some(cwd) = thread.cwd.as_deref() {
            let _ = writeln!(out, "   cwd: {cwd}");
        }
        if let Some(preview) = thread.matched_preview.as_deref() {
            let _ = writeln!(out, "   match: {preview}");
        }
    }
    out.trim_end().to_string()
}

fn format_thread_read_human(result: &ThreadReadResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "codex-threads threads read: {} [{}]",
        result.thread.thread_name, result.thread.id
    );
    let _ = writeln!(
        out,
        "updated: {}  model: {}  messages: {}",
        result.thread.updated_at.as_deref().unwrap_or("-"),
        result.thread.model.as_deref().unwrap_or("-"),
        result.thread.message_count
    );
    if let Some(cwd) = result.thread.cwd.as_deref() {
        let _ = writeln!(out, "cwd: {cwd}");
    }
    if let Some(path) = Some(result.thread.session_path.as_str()) {
        let _ = writeln!(out, "session: {path}");
    }
    out.push('\n');
    for message in &result.messages {
        let _ = writeln!(
            out,
            "[{}] {} {}",
            message.timestamp.as_deref().unwrap_or("-"),
            message.role,
            message.source_type
        );
        for line in message.text.lines() {
            let _ = writeln!(out, "  {line}");
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn format_events_read_human(result: &EventsReadResult) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "codex-threads events read: {} events for [{}]",
        result.events.len(),
        result.session_id
    );
    for event in &result.events {
        let _ = writeln!(
            out,
            "[{}] {} raw={} role={}",
            event.timestamp.as_deref().unwrap_or("-"),
            event.event_type,
            event.raw_type,
            event.role.as_deref().unwrap_or("-")
        );
        if let Some(text) = event.text.as_deref() {
            for line in text.lines() {
                let _ = writeln!(out, "  {line}");
            }
        }
    }
    out.trim_end().to_string()
}

#[derive(Debug, Clone)]
struct Paths {
    codex_home: PathBuf,
    index_path: PathBuf,
}

impl Paths {
    fn from_cli(cli: &Cli) -> Result<Self> {
        let codex_home = match cli.codex_home.as_ref() {
            Some(path) => path.clone(),
            None => codex_root()?,
        };
        let index_path = match cli.index_path.as_ref() {
            Some(path) => path.clone(),
            None => default_local_index_path()?,
        };
        Ok(Self {
            codex_home,
            index_path,
        })
    }

    fn sessions_dir(&self) -> PathBuf {
        self.codex_home.join("sessions")
    }

    fn session_index_path(&self) -> PathBuf {
        self.codex_home.join("session_index.jsonl")
    }
}

fn sync_index(paths: &Paths) -> Result<SyncResult> {
    let sessions_root = paths.sessions_dir();
    let session_index_path = paths.session_index_path();
    let session_index = read_session_index(&session_index_path)?
        .into_iter()
        .map(|entry| (entry.id.clone(), entry))
        .collect::<HashMap<_, _>>();
    let session_files = list_session_files(&sessions_root)?;

    let mut threads = Vec::new();
    let mut messages = Vec::new();

    for session_path in session_files {
        let session_id = session_id_from_path(&session_path).ok_or_else(|| {
            anyhow!(
                "failed to derive session id from {}",
                session_path.display()
            )
        })?;
        let entry = session_index.get(&session_id).cloned();
        let build = parse_session_file(&session_path, &session_id, entry)?;
        let thread = IndexedThread {
            id: build.id.clone(),
            thread_name: build.thread_name.clone(),
            updated_at: build.updated_at.clone(),
            session_path: build.session_path.clone(),
            cwd: build.cwd.clone(),
            model: build.model.clone(),
            cli_version: build.cli_version.clone(),
            message_count: build.messages.len(),
        };
        messages.extend(build.messages);
        threads.push(thread);
    }

    threads.sort_by_key(|thread| Reverse(thread.updated_at.clone()));
    let index = ThreadIndex {
        generated_at: now_rfc3339_fallback(),
        threads,
        messages,
    };

    let index_path = paths.index_path.clone();
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(
        &index_path,
        serde_json::to_string_pretty(&index).context("failed to serialize index")?,
    )
    .with_context(|| format!("failed to write {}", index_path.display()))?;

    Ok(SyncResult {
        session_count: index.threads.len(),
        message_count: index.messages.len(),
        index_path: index_path.display().to_string(),
    })
}

fn load_index(paths: &Paths) -> Result<ThreadIndex> {
    let index_path = paths.index_path.clone();
    let body = fs::read_to_string(&index_path)
        .with_context(|| format!("failed to read {}", index_path.display()))?;
    serde_json::from_str(&body).with_context(|| format!("failed to parse {}", index_path.display()))
}

fn search_messages(index: &ThreadIndex, query: &str, limit: usize) -> MessageSearchResult {
    let needle = query.to_lowercase();
    let tokens = query_tokens(query);
    let mut matches = index
        .messages
        .iter()
        .filter_map(|message| {
            let score = score_text(&message.text, &needle, &tokens);
            if score == 0 {
                return None;
            }
            Some((
                score,
                MessageMatch {
                    session_id: message.session_id.clone(),
                    thread_name: message.thread_name.clone(),
                    timestamp: message.timestamp.clone(),
                    role: message.role.clone(),
                    preview: message.preview.clone(),
                    text: message.text.clone(),
                },
            ))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| right.timestamp.cmp(&left.timestamp))
    });

    MessageSearchResult {
        query: query.to_string(),
        matches: matches
            .into_iter()
            .take(limit)
            .map(|(_, message)| message)
            .collect(),
    }
}

fn resolve_threads(index: &ThreadIndex, query: &str, limit: usize) -> ThreadsResolveResult {
    let needle = query.to_lowercase();
    let tokens = query_tokens(query);
    let mut scores: HashMap<String, (usize, Option<String>)> = HashMap::new();

    for thread in &index.threads {
        let mut score = score_text(&thread.thread_name, &needle, &tokens) * 10;
        if let Some(cwd) = thread.cwd.as_ref() {
            score += score_text(cwd, &needle, &tokens) * 2;
        }

        let mut best_preview = None;
        let mut best_message_score = 0;
        for message in index
            .messages
            .iter()
            .filter(|message| message.session_id == thread.id)
        {
            let message_score = score_text(&message.text, &needle, &tokens);
            if message_score > best_message_score {
                best_message_score = message_score;
                best_preview = Some(message.preview.clone());
            }
        }
        score += best_message_score * 3;

        if score > 0 {
            scores.insert(thread.id.clone(), (score, best_preview));
        }
    }

    let mut matches = index
        .threads
        .iter()
        .filter_map(|thread| {
            scores.get(&thread.id).map(|(score, preview)| {
                (
                    *score,
                    ThreadMatch {
                        id: thread.id.clone(),
                        thread_name: thread.thread_name.clone(),
                        updated_at: thread.updated_at.clone(),
                        cwd: thread.cwd.clone(),
                        model: thread.model.clone(),
                        message_count: thread.message_count,
                        matched_preview: preview.clone(),
                    },
                )
            })
        })
        .collect::<Vec<_>>();

    matches.sort_by(|(left_score, left), (right_score, right)| {
        right_score
            .cmp(left_score)
            .then_with(|| right.updated_at.cmp(&left.updated_at))
    });

    ThreadsResolveResult {
        query: query.to_string(),
        matches: matches
            .into_iter()
            .take(limit)
            .map(|(_, thread)| thread)
            .collect(),
    }
}

fn read_thread(index: &ThreadIndex, session_id: &str) -> Result<ThreadReadResult> {
    let thread = index
        .threads
        .iter()
        .find(|thread| thread.id == session_id)
        .cloned()
        .ok_or_else(|| anyhow!("session not found"))?;
    let mut messages = index
        .messages
        .iter()
        .filter(|message| message.session_id == session_id)
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| message.timestamp.clone());

    Ok(ThreadReadResult { thread, messages })
}

fn read_events(paths: &Paths, session_id: &str, limit: usize) -> Result<EventsReadResult> {
    let session_path = find_session_file(&paths.sessions_dir(), session_id)?
        .ok_or_else(|| anyhow!("session not found"))?;
    let file = fs::File::open(&session_path)
        .with_context(|| format!("failed to open {}", session_path.display()))?;
    let reader = BufReader::new(file);
    let mut values = Vec::new();

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed reading {}", session_path.display()))?;
        let value: Value = serde_json::from_str(&line).context("failed to parse session event")?;
        values.push(value);
    }

    let start = values.len().saturating_sub(limit);
    let mut events = Vec::new();
    for value in values.into_iter().skip(start) {
        let raw_type = value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let timestamp = value
            .get("timestamp")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let (event_type, role, text) = normalize_event_shape(&value);

        events.push(NormalizedEvent {
            timestamp,
            event_type,
            role,
            text,
            raw_type,
        });
    }

    Ok(EventsReadResult {
        session_id: session_id.to_string(),
        events,
    })
}

fn parse_session_file(
    session_path: &Path,
    session_id: &str,
    entry: Option<SessionIndexEntry>,
) -> Result<SessionBuild> {
    let file = fs::File::open(session_path)
        .with_context(|| format!("failed to open {}", session_path.display()))?;
    let reader = BufReader::new(file);
    let (thread_name, updated_at) = match entry {
        Some(entry) => (
            entry.thread_name.unwrap_or_else(|| "untitled".to_string()),
            entry.updated_at,
        ),
        None => ("untitled".to_string(), None),
    };
    let mut build = SessionBuild {
        id: session_id.to_string(),
        thread_name,
        updated_at,
        session_path: session_path.display().to_string(),
        ..Default::default()
    };

    for line in reader.lines() {
        let line = line.with_context(|| format!("failed reading {}", session_path.display()))?;
        let value: Value = serde_json::from_str(&line).context("failed to parse session event")?;
        extract_session_data(&mut build, &value)?;
    }

    if build.thread_name == "untitled" {
        if let Some(first_prompt) = build
            .messages
            .iter()
            .find(|message| message.role == "user" && is_meaningful_title_candidate(&message.text))
        {
            build.thread_name = first_meaningful_line(&first_prompt.text, 60);
        } else if let Some(first_user) =
            build.messages.iter().find(|message| message.role == "user")
        {
            build.thread_name = first_meaningful_line(&first_user.text, 60);
        }
    }

    for message in &mut build.messages {
        message.thread_name = build.thread_name.clone();
    }

    Ok(build)
}

fn extract_session_data(build: &mut SessionBuild, value: &Value) -> Result<()> {
    match value.get("type").and_then(Value::as_str) {
        Some("session_meta") => {
            if let Some(payload) = value.get("payload") {
                if build.cwd.is_none() {
                    build.cwd = payload
                        .get("cwd")
                        .and_then(Value::as_str)
                        .map(ToString::to_string);
                }
                if build.cli_version.is_none() {
                    build.cli_version = payload
                        .get("cli_version")
                        .and_then(Value::as_str)
                        .map(ToString::to_string);
                }
                if build.model.is_none() {
                    build.model = payload
                        .get("model")
                        .and_then(Value::as_str)
                        .or_else(|| payload.get("model_slug").and_then(Value::as_str))
                        .map(ToString::to_string);
                }
            }
        }
        Some("event_msg") => {
            if let Some(payload) = value.get("payload") {
                match payload.get("type").and_then(Value::as_str) {
                    Some("user_message") => {
                        if let Some(text) = payload.get("message").and_then(Value::as_str) {
                            push_message(build, value, "user", text, "event_msg");
                        }
                    }
                    Some("agent_message") => {
                        if let Some(text) = payload.get("message").and_then(Value::as_str) {
                            push_message(build, value, "assistant", text, "event_msg");
                        }
                    }
                    _ => {}
                }
            }
        }
        Some("response_item") => {
            if let Some(payload) = value.get("payload") {
                if payload.get("type").and_then(Value::as_str) == Some("message") {
                    let role = payload
                        .get("role")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    if let Some(content) = payload.get("content").and_then(Value::as_array) {
                        let joined = content
                            .iter()
                            .filter_map(extract_content_text)
                            .collect::<Vec<_>>()
                            .join("\n");
                        if !joined.trim().is_empty() && matches!(role, "user" | "assistant") {
                            push_message(build, value, role, &joined, "response_item");
                        }
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn push_message(
    build: &mut SessionBuild,
    value: &Value,
    role: &str,
    text: &str,
    source_type: &str,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    let timestamp = value
        .get("timestamp")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let dedupe_key = format!("{}|{}", role, text);
    if let Some((index, existing_source_type)) = build.seen_messages.get(&dedupe_key).cloned() {
        if existing_source_type == "event_msg" && source_type == "response_item" {
            if let Some(existing) = build.messages.get_mut(index) {
                existing.source_type = source_type.to_string();
                if existing.timestamp.is_none() {
                    existing.timestamp = timestamp.clone();
                }
            }
        }
        return;
    }
    build
        .seen_messages
        .insert(dedupe_key, (build.messages.len(), source_type.to_string()));
    let preview = preview_text(text, 180);
    build.messages.push(IndexedMessage {
        session_id: build.id.clone(),
        thread_name: build.thread_name.clone(),
        timestamp,
        role: role.to_string(),
        text: text.to_string(),
        preview,
        source_type: source_type.to_string(),
    });
}

fn extract_content_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    if let Some(text) = value.get("input_text").and_then(Value::as_str) {
        return Some(text.to_string());
    }
    None
}

fn normalize_event_shape(value: &Value) -> (String, Option<String>, Option<String>) {
    match value.get("type").and_then(Value::as_str) {
        Some("session_meta") => ("session_meta".to_string(), None, None),
        Some("event_msg") => {
            let payload_type = value
                .get("payload")
                .and_then(|payload| payload.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("event_msg")
                .to_string();
            let role = match payload_type.as_str() {
                "user_message" => Some("user".to_string()),
                "agent_message" => Some("assistant".to_string()),
                _ => None,
            };
            let text = value
                .get("payload")
                .and_then(|payload| payload.get("message"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .or_else(|| {
                    value
                        .get("payload")
                        .and_then(|payload| payload.get("text"))
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                });
            (payload_type, role, text)
        }
        Some("response_item") => {
            let payload_type = value
                .get("payload")
                .and_then(|payload| payload.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("response_item")
                .to_string();
            let role = value
                .get("payload")
                .and_then(|payload| payload.get("role"))
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let text = value
                .get("payload")
                .and_then(|payload| payload.get("content"))
                .and_then(Value::as_array)
                .map(|content| {
                    content
                        .iter()
                        .filter_map(extract_content_text)
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .filter(|joined| !joined.trim().is_empty());
            (payload_type, role, text)
        }
        Some(other) => (other.to_string(), None, None),
        None => ("unknown".to_string(), None, None),
    }
}

fn read_session_index(path: &Path) -> Result<Vec<SessionIndexEntry>> {
    let file =
        fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("failed reading {}", path.display()))?;
        let entry: SessionIndexEntry =
            serde_json::from_str(&line).context("failed to parse session_index entry")?;
        entries.push(entry);
    }
    Ok(entries)
}

fn find_session_file(sessions_root: &Path, session_id: &str) -> Result<Option<PathBuf>> {
    for file in list_session_files(sessions_root)? {
        if session_id_from_path(&file).as_deref() == Some(session_id) {
            return Ok(Some(file));
        }
    }
    Ok(None)
}

fn list_session_files(sessions_root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for year in fs::read_dir(sessions_root)
        .with_context(|| format!("failed to read {}", sessions_root.display()))?
    {
        let year = year?;
        if !year.file_type()?.is_dir() {
            continue;
        }
        for month in fs::read_dir(year.path())? {
            let month = month?;
            if !month.file_type()?.is_dir() {
                continue;
            }
            for day in fs::read_dir(month.path())? {
                let day = day?;
                if !day.file_type()?.is_dir() {
                    continue;
                }
                for file in fs::read_dir(day.path())? {
                    let file = file?;
                    if file.file_type()?.is_file()
                        && file.path().extension().and_then(|ext| ext.to_str()) == Some("jsonl")
                    {
                        files.push(file.path());
                    }
                }
            }
        }
    }
    files.sort();
    Ok(files)
}

fn session_id_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    stem.rsplit('-').next().map(ToString::to_string)
}

fn default_local_index_path() -> Result<PathBuf> {
    Ok(config_dir_path()?.join("codex-threads").join("index.json"))
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

fn codex_root() -> Result<PathBuf> {
    if let Ok(path) = env::var("CODEX_HOME") {
        return Ok(expand_path(&path));
    }
    let home = env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".codex"))
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

fn preview_text(text: &str, max_chars: usize) -> String {
    let trimmed = text.replace('\n', " ");
    if trimmed.chars().count() <= max_chars {
        return trimmed;
    }
    let preview = trimmed.chars().take(max_chars).collect::<String>();
    format!("{preview}...")
}

fn first_meaningful_line(text: &str, max_chars: usize) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or(text);
    preview_text(line, max_chars)
}

fn is_meaningful_title_candidate(text: &str) -> bool {
    let trimmed = text.trim();
    !(trimmed.starts_with("# AGENTS.md")
        || trimmed.starts_with("<environment_context>")
        || trimmed.starts_with("<INSTRUCTIONS>")
        || trimmed.starts_with("You are Codex")
        || trimmed.contains("# AGENTS.md instructions for "))
}

fn query_tokens(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|token| token.trim().to_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn score_text(text: &str, needle: &str, tokens: &[String]) -> usize {
    let haystack = text.to_lowercase();
    let mut score = 0;
    if haystack.contains(needle) {
        score += 100;
    }
    for token in tokens {
        if haystack.contains(token) {
            score += 10;
        }
    }
    score
}

fn now_rfc3339_fallback() -> String {
    "generated".to_string()
}

fn classify_error(error: &anyhow::Error) -> ErrorBody {
    let message = error.to_string();
    let code = match message.as_str() {
        msg if msg.starts_with("error: ") => "invalid_usage",
        msg if msg.contains("session not found") => "not_found",
        msg if msg.contains("failed to read") || msg.contains("failed to open") => "io_error",
        msg if msg.contains("failed to parse") => "parse_error",
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
    use clap::CommandFactory;

    #[test]
    fn top_level_help_documents_output_contract() {
        let mut cmd = Cli::command();
        let help = cmd.render_long_help().to_string();

        assert!(help.contains("Default output is human-readable."));
        assert!(help.contains("Use --json for compact machine-readable JSON."));
    }

    #[test]
    fn sync_human_output_is_readable() {
        let rendered = format_sync_human(&SyncResult {
            session_count: 3,
            message_count: 42,
            index_path: "/tmp/index.json".to_string(),
        });

        assert!(rendered.contains("indexed 3 sessions and 42 messages"));
        assert!(rendered.contains("index: /tmp/index.json"));
    }

    #[test]
    fn message_search_human_output_lists_matches() {
        let rendered = format_message_search_human(&MessageSearchResult {
            query: "forge".to_string(),
            matches: vec![MessageMatch {
                session_id: "abc123".to_string(),
                thread_name: "Build Forge CLI".to_string(),
                timestamp: Some("2026-04-11T10:00:00Z".to_string()),
                role: "user".to_string(),
                preview: "Implement the forge output contract".to_string(),
                text: "Implement the forge output contract".to_string(),
            }],
        });

        assert!(rendered.contains("codex-threads messages search: 1 matches for \"forge\""));
        assert!(rendered.contains("1. Build Forge CLI [abc123] 2026-04-11T10:00:00Z user"));
        assert!(rendered.contains("Implement the forge output contract"));
    }

    #[test]
    fn thread_read_human_output_renders_message_blocks() {
        let rendered = format_thread_read_human(&ThreadReadResult {
            thread: IndexedThread {
                id: "abc123".to_string(),
                thread_name: "Investigate output".to_string(),
                updated_at: Some("2026-04-11T11:00:00Z".to_string()),
                session_path: "/tmp/session.jsonl".to_string(),
                cwd: Some("/work".to_string()),
                model: Some("gpt-5.4".to_string()),
                cli_version: None,
                message_count: 2,
            },
            messages: vec![IndexedMessage {
                session_id: "abc123".to_string(),
                thread_name: "Investigate output".to_string(),
                timestamp: Some("2026-04-11T11:00:00Z".to_string()),
                role: "user".to_string(),
                text: "Find the bug".to_string(),
                preview: "Find the bug".to_string(),
                source_type: "event_msg".to_string(),
            }],
        });

        assert!(rendered.contains("codex-threads threads read: Investigate output [abc123]"));
        assert!(rendered.contains("session: /tmp/session.jsonl"));
        assert!(rendered.contains("[2026-04-11T11:00:00Z] user event_msg"));
        assert!(rendered.contains("  Find the bug"));
    }

    #[test]
    fn human_error_output_is_not_json() {
        let rendered = format_error_human(
            "codex-threads",
            &ErrorBody {
                code: "not_found".to_string(),
                message: "session not found".to_string(),
            },
        );

        assert!(rendered.starts_with("codex-threads error [not_found]"));
        assert!(rendered.contains("session not found"));
        assert!(!rendered.contains("{\"ok\":false"));
    }
}
