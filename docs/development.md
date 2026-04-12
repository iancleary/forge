# Forge Development

This document is for working on Forge from a local checkout.

If you want to use Forge as an installed tool on a machine, start in `README.md` instead.

## Run From Source

```sh
cargo run -p forge -- self update-check --force
cargo run -p slack-cli -- --help
cargo run -p codex-threads -- --json sync
cargo run -p linear -- auth login
cargo run -p linear -- --json viewer
```

## Install Locally From Checkout

```sh
cargo install --path crates/forge
cargo install --path crates/slack-cli
cargo install --path crates/codex-threads
cargo install --path crates/linear
```

Then:

```sh
forge doctor
forge self update-check --force
```

## Use Repo-Sourced Skills While Developing

When developing, install skills from the repo source:

```sh
forge skills install --all --source repo --target user
```

If you want to test non-user targets explicitly:

```sh
forge skills install --all --source repo --target path:/opt/forge-skills --target-role mainline
forge skills status --target path:/opt/forge-skills
```

## Development Notes

- Prefer `docs/*.md` as the command/spec source of truth; keep implementations aligned with the docs.
- Prefer JSON-first output for agent consumption.
