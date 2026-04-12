# Forge Development

This document is for working on Forge from a local checkout.

If you want to use Forge as an installed tool on a machine, start in `README.md` instead.

## Run From Source

```sh
cargo run -p forge -- self update-check --force
cargo run -p forge -- dev install --repo-path "$(pwd)"
cargo run -p slack-agent -- --help
cargo run -p slack-query -- --help
cargo run -p codex-threads -- --json sync
cargo run -p linear -- auth login
cargo run -p linear -- --json viewer
```

## Install Locally From Checkout

```sh
cargo install --path crates/forge
cargo install --path crates/slack-query
cargo install --path crates/slack-agent
cargo install --path crates/codex-threads
cargo install --path crates/linear
```

Then:

```sh
forge doctor
forge dev install --repo-path "$(pwd)"
forge self update-check --force
```

## Use Repo-Sourced Skills While Developing

When developing, install skills from the repo source:

```sh
forge skills install --all --source repo --repo-path "$(pwd)"
```

If you want to test non-user targets explicitly:

```sh
forge skills install --all --source repo --repo-path "$(pwd)" --target path:/opt/forge-skills --target-role mainline
forge skills status --target path:/opt/forge-skills
```

## Development Notes

- Prefer `docs/*.md` as the command/spec source of truth; keep implementations aligned with the docs.
- Prefer human-readable default output with compact `--json` for agent consumption.
