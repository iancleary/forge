# Forge CLI

This document defines the top-level `forge` manager CLI.

## Goal

Provide shared configuration and self-management commands for the local Forge toolchain.

This is where update-check and update behavior should live, rather than embedding self-update logic independently inside every tool.

## Commands

### `forge self update-check`

```sh
forge self update-check [--force] [--repo-path <path>] [--json]
```

Checks whether the local Forge repo is behind the remote default branch.

Behavior:

- reads `~/.config/forge/config.toml` when present
- caches the last check result in `~/.config/forge/state.toml`
- uses a TTL to avoid hitting the network on every run
- `--force` bypasses the cache

### `forge self update`

```sh
forge self update [--repo-path <path>] [--branch <name>] [--json]
```

Updates the local Forge repo using:

```sh
git pull --rebase origin <branch>
```

Default branch behavior:

- prefer the remote default branch when it can be resolved
- otherwise fall back to `main`

## Config

Preferred config file:

```text
~/.config/forge/config.toml
```

Example:

```toml
auto_check_updates = true
auto_update = false
update_check_ttl_minutes = 1440
repo_path = "~/Development/forge"
```

Optional override:

- `FORGE_CONFIG_DIR`

State cache:

```text
~/.config/forge/state.toml
```

## Notes

- `auto_check_updates` is the intended default pattern
- `auto_update` should stay off by default until the toolchain is more mature
- `forge self update-check` is safe to run frequently with cache enabled
- `forge self update` is explicit on purpose

## Versioning Policy

Forge uses semver-compatible calendar versioning for crates:

- format: `YYYY.MMDD.N`
- first release on April 10, 2026: `2026.410.0`
- second release the same day: `2026.410.1`

This keeps crate versions valid for Cargo while allowing multiple releases per day.
