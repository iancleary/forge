# Forge CLI

This document defines the top-level `forge` manager CLI.

## Goal

Provide shared configuration and self-management commands for the local Forge toolchain.

This is where update-check and update behavior should live, rather than embedding self-update logic independently inside every tool.

Forge also owns the lifecycle of Forge-managed consumer skills. The detailed skill management contract lives in `docs/forge-skills.md`.

## Commands

### `forge permissions check`

```sh
forge permissions check [--json]
```

Audits known Forge-managed config directories and secret files, including:

- `~/.config/forge/`
- `~/.config/forge/slack-cli/`
- `~/.config/forge/linear/`

It reports whether directories and token files match the expected owner-only modes.

### `forge permissions fix`

```sh
forge permissions fix [--json]
```

Applies the expected local permissions:

- directories: `0700`
- token and config files: `0600`

### `forge self update-check`

```sh
forge self update-check [--force] [--repo-path <path>] [--json]
```

Checks whether the local Forge install is out of date.

Behavior:

- reads `~/.config/forge/config.toml` when present
- caches the last check result in `~/.config/forge/state.toml`
- uses a TTL to avoid hitting the network on every run
- `--force` bypasses the cache
- checks both Forge binary/repo drift and Forge-managed skill drift
- checks `mainline` Forge-managed skill drift by default
- uses the configured Forge repo when running in repo-checkout mode
- otherwise uses the installed Forge release as the canonical source for managed skills

### `forge self update`

```sh
forge self update [--repo-path <path>] [--branch <name>] [--json]
```

Updates the local Forge installation and reconciles Forge-managed skills.

In repo-checkout mode, Forge updates the local repo using:

```sh
git pull --rebase origin <branch>
```

Default branch behavior:

- prefer the remote default branch when it can be resolved
- otherwise fall back to `main`

In release mode, Forge uses the installed release payload as the canonical source and updates `mainline` Forge-managed skills without requiring a local checkout.

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

Skill lifecycle configuration is documented in `docs/forge-skills.md`.

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
- Forge-managed skills are deployed artifacts, not peer sources of truth

## Forge CLI-First Guidance

Forge should be the primary interface for Forge-managed domains when the task needs stable, reusable output.

- prefer Forge commands when the result should be low-token, deterministic, and ready for a follow-up command
- use `jq` only for one-off local projection after a Forge command already returned the right records
- use `rg` for local repo exploration and unstructured file search, not as the main interface to an external system Forge already models

When deciding whether to expand Forge instead of relying on shell shaping, look for repeated pain:

- the same `jq` cleanup appears across multiple sessions
- the same noisy fields keep being dropped before reasoning can begin
- the same IDs, summaries, or normalized fields are needed every time
- the desired shaped output can be described as a stable example in the docs

When those signals are present, fold the pain into Forge as the smallest possible stable primitive: a narrow flag, output mode, or subcommand rather than a broad generic query feature.

## Versioning Policy

Forge uses semver-compatible calendar versioning for crates:

- format: `YYYY.MMDD.N`
- first release on April 10, 2026: `2026.410.0`
- second release the same day: `2026.410.1`

This keeps crate versions valid for Cargo while allowing multiple releases per day.
