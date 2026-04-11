# Slack CLI

This document defines the reusable Slack utility CLI for this repo.

## Goal

Provide a stable Slack CLI that Codex can use across sessions for deterministic Slack operations.

The initial implementation is read-heavy and optimized for workspace research. It is not the conversational assistant layer.

## Scope

Use the official Slack `slack` CLI for:

- login and authorization
- app and project setup
- app deployment and manifests
- local development against Slack apps

Use this repo's `slack-cli` for:

- resolving permalinks into Slack identifiers
- reading a thread
- pulling nearby channel context around a message
- pulling nearby thread context around a message
- searching messages
- later, a small set of explicit write actions if they are broadly useful outside OpenClaw

## Setup

Create and install a Slack app, then configure OAuth scopes in Slack App Settings under `OAuth & Permissions`.

Do not commit raw app-settings URLs with workspace IDs or app IDs to this repo. If you need to document the route, sanitize it like this:

```text
https://app.slack.com/app-settings/<workspace-id>/<app-id>/oauth
```

That is the Slack App Settings page where you can install the app, review `OAuth & Permissions`, and copy the generated user token after installation.

For public repo docs, prefer naming the UI location instead:

- Slack App Settings
- OAuth & Permissions
- User Token Scopes

## Safety Rules

- v1 is read-only
- no commands for posting, editing, deleting, reacting, joining, or mutating channels
- preferred auth is a local config directory under `~/.config/forge/slack-cli/`
- `SLACK_API_TOKEN` remains supported as an override for ad hoc use and CI
- the expected setup is: create a Slack app, grant read scopes, install it to the workspace, then store the token locally
- official Slack CLI authentication can be revisited later, but it is not required for the initial implementation

Future broad write actions in `slack-cli` should stay explicit and narrow, for example:

- `post-message`
- `reply`
- `bookmark add`

OpenClaw-specific workflows belong in `openclaw-slack`, not here.

## Permission Scopes

Recommended token type:

- user token

Decision:

- `slack-cli` uses a user token
- reason: this CLI is the shared primitive layer for Codex sessions and should reflect what the user can read and explicitly do in Slack
- this also aligns with `search:read`, which is needed for message search

Recommended scopes for the reusable `slack-cli`:

- `channels:history`
- `groups:history`
- `im:history`
- `mpim:history`
- `search:read`

Optional:

- `channels:read`

Why:

- `conversations.history` and `conversations.replies` need the relevant `*:history` scopes based on conversation type
- `search.messages` needs `search:read`
- `channels:read` is useful if we later add channel lookup or validation

Scopes to avoid in this CLI unless a broadly reusable command requires them:

- write scopes such as `channels:write`, `files:write`, `pins:write`, `reminders:write`
- unrelated read scopes such as `bookmarks:read`, `links:read`, `files:read`

## Auth Configuration

Preferred auth layout:

```text
~/.config/forge/slack-cli/
  config.toml
  token
```

Place the copied user token here:

```text
~/.config/forge/slack-cli/token
```

Supported lookup order:

1. `SLACK_API_TOKEN`
2. `~/.config/forge/slack-cli/config.toml`
3. `~/.config/forge/slack-cli/token`

Optional override:

- `FORGE_SLACK_CLI_CONFIG_DIR`

Recommended `config.toml`:

```toml
token_file = "~/.config/forge/slack-cli/token"
```

Alternative `config.toml`:

```toml
token = "xoxp-..."
```

The token file should contain only the raw token string.

Recommended local permissions:

- config directory: `0700`
- token file: `0600`

The env var remains useful for:

- CI
- one-off local testing
- temporary profile switching

## Command List

### `resolve-permalink`

```sh
slack-cli resolve-permalink <permalink> [--json]
```

Resolves a Slack permalink into identifiers that later commands can reuse.

Expected fields:

```json
{
  "ok": true,
  "data": {
    "team_id": "T123",
    "channel_id": "C123",
    "message_ts": "1712785154.123456",
    "thread_ts": "1712785154.123456",
    "is_thread_root": true,
    "reply_count": 8
  }
}
```

### `read-thread`

```sh
slack-cli read-thread <channel-id> <thread-ts> [--limit <n>] [--json]
```

Reads the root message and replies for a thread.

Slack API mapping: `conversations.replies`

Expected fields:

```json
{
  "ok": true,
  "data": {
    "channel_id": "C123",
    "thread_ts": "1712785154.123456",
    "messages": [
      {
        "ts": "1712785154.123456",
        "user": "U123",
        "text": "Root message"
      }
    ]
  }
}
```

### `search`

```sh
slack-cli search <query> [--limit <n>] [--page <n>] [--json]
```

Searches Slack messages the user token can access.

Slack API mapping: `search.messages`

Expected fields:

```json
{
  "ok": true,
  "data": {
    "query": "actual PDF",
    "messages": [
      {
        "channel": {
          "id": "C123",
          "name": "general"
        },
        "user": "U123",
        "username": "assistant",
        "text": "Match text",
        "ts": "1712785154.123456",
        "permalink": "https://workspace.slack.com/archives/..."
      }
    ]
  }
}
```

### `channel-context`

```sh
slack-cli channel-context <channel-id> <message-ts> [--before <n>] [--after <n>] [--json]
```

Reads nearby top-level channel messages around a target message.

Slack API mapping: `conversations.history`

Expected fields:

```json
{
  "ok": true,
  "data": {
    "channel_id": "C123",
    "target": {
      "ts": "1712785154.123456",
      "user": "U123",
      "text": "Target message"
    },
    "before": [],
    "after": []
  }
}
```

This command returns parent-channel timeline context, not thread replies.

### `thread-context`

```sh
slack-cli thread-context <channel-id> <thread-ts> <message-ts> [--before <n>] [--after <n>] [--json]
```

Reads nearby messages within a thread around a target thread message.

Slack API mapping: `conversations.replies`

Expected fields:

```json
{
  "ok": true,
  "data": {
    "channel_id": "C123",
    "target": {
      "ts": "1712785154.123456",
      "user": "U123",
      "text": "Target reply"
    },
    "before": [],
    "after": []
  }
}
```

This command returns thread-local context, not surrounding top-level channel messages.

## Verified Examples

These commands have been exercised successfully against a real Slack workspace using the current implementation:

```sh
slack-cli resolve-permalink "https://workspace.slack.com/archives/C123/p1712785154123456"
slack-cli read-thread C123 1712785154.123456 --limit 20
slack-cli channel-context C123 1712785154.123456 --before 2 --after 2
slack-cli thread-context C123 1712785000.000001 1712785154.123456 --before 2 --after 2
slack-cli search "actual PDF" --limit 5
```

## Planned Next Commands

Broadly useful future write commands, if needed:

```sh
slack-cli post-message <channel-id> --body-file message.md [--json]
slack-cli reply <channel-id> <thread-ts> --body-file reply.md [--json]
```

## API Mapping

Initial implementation should call Slack Web API endpoints directly using a locally supplied token.

Mappings:

- `resolve-permalink`
  - parse permalink locally
  - enrich with thread metadata later if needed
- `read-thread`
  - `conversations.replies`
- `search`
  - `search.messages`
- `channel-context`
  - `conversations.history`
- `thread-context`
  - `conversations.replies`

Relevant Slack constraints from the official docs:

- `conversations.replies` on public and private channel threads requires a user token with the relevant history scopes
- `search.messages` requires a user token with `search:read`
- for newer non-Marketplace external apps, `conversations.history` and `conversations.replies` are subject to tighter rate limits

## Relationship To OpenClaw

`slack-cli` is the shared primitive layer.

It should expose deterministic commands with stable JSON and minimal policy. OpenClaw-specific triage, drafting, batching, and approval-heavy behavior should live in a separate `openclaw-slack` CLI.

Token boundary:

- `slack-cli` uses a user token
- `openclaw-slack` uses a bot token
- this keeps the shared utility CLI aligned with user access while allowing OpenClaw to operate as a separate assistant identity

Public references:

- `conversations.history`: `https://api.slack.com/methods/conversations.history`
- `conversations.replies`: `https://api.slack.com/methods/conversations.replies`
- `search.messages`: `https://api.slack.com/methods/search.messages`
