# Codex Threads CLI

This document defines the `codex-threads` CLI.

## Goal

Provide a fast, low-noise way to search and read old Codex sessions without handing raw session archives directly to every future agent run.

The source data lives in `~/.codex/sessions` and `~/.codex/session_index.jsonl`. This CLI builds a compact local index and exposes agent-friendly commands on top of it.

## Command Surface

```sh
codex-threads --json sync
codex-threads --json messages search "build a CLI" --limit 20
codex-threads --json threads resolve "tweet idea"
codex-threads --json threads read <session-id>
codex-threads --json events read <session-id> --limit 50
```

## Scope

The initial implementation should:

- read the local Codex session archive
- build a compact local index
- search extracted user and assistant messages
- resolve threads by name or message match
- read a normalized thread view
- read normalized events for a session

The initial implementation should not:

- depend on a remote service
- depend on a database server
- expose raw tool output by default unless asked through `events read`

## Auth And Data Location

No external auth is required.

Default source paths:

- `~/.codex/session_index.jsonl`
- `~/.codex/sessions/`

Default local index path:

- `~/.config/forge/codex-threads/index.json`

Overrides:

- `--codex-home <path>`
- `--index-path <path>`
- `CODEX_HOME`

## Install And Run

Run from source during development:

```sh
cargo run -p codex-threads -- --json sync
cargo run -p codex-threads -- --json messages search "build a CLI" --limit 5
cargo run -p codex-threads -- --codex-home ~/.codex --index-path /tmp/codex-index.json --json sync
```

Install locally:

```sh
cargo install --path crates/codex-threads
```

Then run:

```sh
codex-threads --json sync
codex-threads --json threads resolve "codex threads"
codex-threads --json events read <session-id> --limit 20
```

## Commands

### `sync`

```sh
codex-threads --json sync
```

Scans the local session archive and writes a compact searchable index.

Useful flags:

- `--codex-home <path>`
- `--index-path <path>`

### `messages search`

```sh
codex-threads --json messages search <query> [--limit <n>]
```

Searches extracted user and assistant messages from the local index.

Ranking notes:

- exact phrase matches rank above token-only matches
- more recent results break score ties

### `threads resolve`

```sh
codex-threads --json threads resolve <query> [--limit <n>]
```

Finds likely matching threads by thread name and extracted message text.

Ranking notes:

- thread-name matches are weighted more heavily than body matches
- message-body matches contribute a preview snippet for context

### `threads read`

```sh
codex-threads --json threads read <session-id>
```

Reads a normalized thread view including metadata and extracted messages.

### `events read`

```sh
codex-threads --json events read <session-id> [--limit <n>]
```

Reads normalized raw events from a session, primarily for deeper inspection and debugging.

## Output Model

Preferred top-level success shape:

```json
{
  "ok": true,
  "data": {}
}
```

Preferred error shape:

```json
{
  "ok": false,
  "error": {
    "code": "not_found",
    "message": "session not found"
  }
}
```

## Extraction Policy

The index should prefer signal over completeness.

Include:

- thread id
- thread name
- updated timestamp
- session path
- extracted user messages
- extracted assistant messages
- a few normalized metadata fields like cwd and model

Exclude from the main search index:

- raw tool outputs
- token count noise
- most event bookkeeping
- giant instruction payloads unless later proven useful

## Future Extensions

- incremental sync
- file-backed full-text index
- extraction of tool-call summaries
- skill-pattern mining commands
