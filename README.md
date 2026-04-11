# forge

Agent-friendly CLIs built as Rust binaries.

## Layout

- `docs/` contains product and design specs for each CLI
- `crates/` contains Rust crates for each CLI implementation

## Current CLIs

- `forge` for shared config and self-management
- `slack-cli` for Slack research workflows
- `openclaw-slack` for stricter OpenClaw Slack workflows

## Token Strategy

- `slack-cli` uses a user token because it is intended to read and act on behalf of the user across the conversations the user can access
- `openclaw-slack` uses a bot token because it is intended to operate as a distinct assistant identity with tighter workflow-specific permissions

## Install And Run

Run from source during development:

```sh
cargo run -p forge -- self update-check --force
cargo run -p slack-cli -- --help
```

Install locally:

```sh
cargo install --path crates/forge
cargo install --path crates/slack-cli
```

Then run:

```sh
forge self update-check --force
slack-cli search "actual PDF" --limit 5
```
