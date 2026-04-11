# forge

Agent-friendly CLIs built as Rust binaries.

## Layout

- `docs/` contains product and design specs for each CLI
- `crates/` contains Rust crates for each CLI implementation

## Current CLIs

- `forge` for shared config and self-management
- `slack-cli` for Slack research workflows
- `linear` for Linear issue, project, and milestone workflows
- `openclaw-slack` for stricter OpenClaw Slack workflows

## Token Strategy

- `slack-cli` uses a user token because it is intended to read and act on behalf of the user across the conversations the user can access
- `openclaw-slack` uses a bot token because it is intended to operate as a distinct assistant identity with tighter workflow-specific permissions

## Versioning

Forge uses semver-compatible calendar versioning:

- format: `YYYY.MMDD.N`
- example: `2026.410.0`
- `YYYY` is the calendar year
- `MMDD` is month and day without separators
- `N` is the release counter for that day

This sorts better for Cargo than build metadata forms such as `2026.4.10+0`.

## Install And Run

Run from source during development:

```sh
cargo run -p forge -- self update-check --force
cargo run -p slack-cli -- --help
cargo run -p codex-threads -- --json sync
cargo run -p linear -- auth login
cargo run -p linear -- --json viewer
```

Install locally:

```sh
cargo install --path crates/forge
cargo install --path crates/slack-cli
cargo install --path crates/codex-threads
cargo install --path crates/linear
```

Then run:

```sh
forge self update-check --force
slack-cli search "actual PDF" --limit 5
codex-threads --json messages search "build a CLI" --limit 5
linear auth login
linear --json project list --limit 5
```
