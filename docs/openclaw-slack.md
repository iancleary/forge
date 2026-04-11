# OpenClaw Slack CLI

This document defines the stricter Slack CLI used by OpenClaw-specific workflows.

## Goal

Provide a smaller, workflow-oriented command surface for OpenClaw so agent runs are faster, less ambiguous, and less likely to fail due to tool misuse, timeouts, or oversized outputs.

This is not the shared Slack utility layer. It sits on top of that layer or parallel to it with tighter workflow contracts.

## Why Separate It

`slack-cli` is for reusable Slack primitives across many Codex sessions.

`openclaw-slack` is for one agent and one working style:

- inbox triage
- thread summarization
- reply drafting
- explicit send actions
- bounded, policy-heavy workflows

Separating them keeps the general CLI stable while letting OpenClaw use stricter commands and smaller outputs.

## Setup

Use Slack App Settings under `OAuth & Permissions` to configure the app.

Do not store private app-settings URLs with raw workspace or app IDs in the repo. If you need an example route, sanitize it:

```text
https://app.slack.com/app-settings/<workspace-id>/<app-id>/oauth
```

## Safety Model

Suggested action tiers:

- Tier 1: read and summarize
- Tier 2: draft only
- Tier 3: explicit send
- Tier 4: destructive or administrative actions

Default behavior:

- reads are allowed
- drafts are allowed when explicitly requested
- sends require an explicit send command
- destructive/admin actions should be excluded unless there is a strong proven need

## Permission Scopes

Recommended token type:

- bot token

Decision:

- `openclaw-slack` uses a bot token
- reason: OpenClaw should operate as a distinct assistant identity rather than impersonating the user by default
- this gives it a narrower, more intentional operational boundary than the shared `slack-cli`

Recommended baseline scopes for `openclaw-slack`:

- `channels:history`
- `groups:history`
- `im:history`
- `mpim:history`

Recommended only if OpenClaw must send messages:

- `chat:write`

Potentially useful, but do not add unless a command requires them:

- `channels:read`
- `pins:write`
- `bookmarks:write`
- `files:write`

Avoid by default:

- broad channel management scopes
- reminder scopes
- unrelated write scopes

## Candidate Commands

Examples:

```sh
openclaw-slack inbox next --json
openclaw-slack thread summarize <channel-id> <thread-ts> --json
openclaw-slack reply draft <channel-id> <thread-ts> --instructions "short and direct" --json
openclaw-slack reply send <channel-id> <thread-ts> --body-file reply.md --json
openclaw-slack mentions triage --limit 10 --json
```

Design requirements:

- bounded output size
- explicit defaults
- no hidden sending behavior
- draft and send are separate verbs
- retryable structured errors
- small result schemas tailored to OpenClaw

## Relationship To `slack-cli`

`slack-cli` exposes primitives such as:

- permalink parsing
- thread reads
- context reads
- search

`openclaw-slack` should expose workflows such as:

- summarize a thread
- draft a reply
- send a reviewed reply
- triage mentions or inbox items

Token boundary:

- `slack-cli` uses a user token for general user-aligned Slack access
- `openclaw-slack` uses a bot token for assistant-specific workflows
