# forge

Agent-friendly CLIs built as Rust binaries.

## Layout

- `docs/` contains product and design specs for each CLI
- `crates/` contains Rust crates for each CLI implementation

## Current CLIs

- `slack-cli` for Slack research workflows
- `openclaw-slack` for stricter OpenClaw Slack workflows

## Token Strategy

- `slack-cli` uses a user token because it is intended to read and act on behalf of the user across the conversations the user can access
- `openclaw-slack` uses a bot token because it is intended to operate as a distinct assistant identity with tighter workflow-specific permissions
