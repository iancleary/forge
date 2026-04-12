# Forge CLI

This document defines the top-level `forge` manager CLI.

## Goal

Provide shared configuration and self-management commands for the local Forge toolchain.

This is where update-check and update behavior should live, rather than embedding self-update logic independently inside every tool.

Forge also owns the lifecycle of Forge-managed consumer skills. The detailed skill management contract lives in `docs/forge-skills.md`.

When Forge is being used as the first-party source of truth for Codex behavior, the higher-level routing and ownership model lives in `docs/codex.md`.

Forge also owns the explicit deployment path for the narrow set of user-scoped Codex files it manages in v1. Those commands are top-level `forge codex` subcommands rather than part of the skills lifecycle.

## Commands

Default output contract for every `forge` command:

- human-readable text by default
- compact JSON envelope with `--json`
- no pretty-printed JSON on the agent path

### `forge doctor`

```sh
forge doctor [--json]
```

Checks whether the local Forge environment is ready for agent workflows.

Current checks:

- required tools: `cargo`, `git`, `gh`, `rg`, `jq`
- GitHub CLI auth readiness via `gh auth status`
- Linear token-source presence via `LINEAR_API_KEY`, `~/.config/forge/linear/config.toml`, and `~/.config/forge/linear/token`
- Slack token-source presence via `SLACK_API_TOKEN`, `~/.config/forge/slack-cli/config.toml`, and `~/.config/forge/slack-cli/token`
- Forge config directory presence at `~/.config/forge/` or `FORGE_CONFIG_DIR`

Behavior:

- default output is optimized for fast, effective visual scanning by a human reader
- `--json` emits deterministic machine-readable output with minimal tokens for agent handoff and chaining
- `--json` should stay compact rather than pretty-printed because token efficiency matters more than readability on the agent path
- errors follow the same contract: human-readable by default, compact JSON with `--json`
- agent skills should prefer `--json` when chaining Forge output into later reasoning or commands
- remediation is included for failing or warning checks
- upgrade commands may also be included for installed tools
- auth checks are advisory and should not block Codex from continuing
- `gh` auth should warn gracefully when it cannot be confirmed from a non-interactive subprocess
- when that happens, the primary remediation is to ask the user to run `gh auth status` in an interactive terminal
- file-based auth tools such as `linear` and `slack-cli` should report configured token sources, not claim a verified logged-in session
- for those file-based tools, doctor should surface the exact CLI commands and docs to use next, such as `linear config`, `linear auth login`, and `slack-cli auth login`
- Windows install hints should prefer `winget` for `git` and `gh`
- Windows upgrade hints should prefer `winget upgrade --id ...` for `git` and `gh`
- on macOS and Linux, use `cargo` for tools that support it such as `rg` and `jq`

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

Updates the active Forge source of truth and reconciles Forge-managed surfaces.

In repo-checkout mode, Forge updates the local repo using:

```sh
git pull --rebase origin <branch>
```

Default branch behavior:

- prefer the remote default branch when it can be resolved
- otherwise fall back to `main`

In release mode, Forge uses the installed release payload as the canonical source and updates `mainline` Forge-managed skills without requiring a local checkout.

Important boundary:

- in repo-checkout mode, `forge self update` pulls the configured repo and then reconciles managed skills
- in release mode, `forge self update-check` compares the running Forge version to the newest release tag from the Forge repo
- in release mode, `forge self update` installs the newest tagged release with Cargo when needed
- after source update, Forge reconciles managed skills and reapplies the managed Codex baseline

### `forge codex render`

```sh
forge codex render [--asset agents|rules]... [--target user|path:<abs-path>] [--source repo|release] [--repo-path <path>] [--json]
```

Renders the Forge-managed Codex user assets for the selected target root without writing them.

V1 managed assets:

- `AGENTS.md`
- `rules/user-policy.rules`

Behavior:

- defaults to all managed assets
- defaults to `user`, which maps to `~/.codex`
- supports `path:<abs-path>` targets for testing and explicit non-default installs
- uses repo source when running from a Forge checkout unless `--source release` is selected explicitly
- default output renders human-readable file sections
- `--json` emits deterministic compact JSON that includes rendered content and the resolved target paths

### `forge codex diff`

```sh
forge codex diff [--asset agents|rules]... [--target user|path:<abs-path>] [--source repo|release] [--repo-path <path>] [--json]
```

Compares the rendered Forge-managed Codex user assets against the selected live target root.

Behavior:

- reports `same`, `changed`, or `missing` per managed asset
- does not touch unrelated files under `~/.codex`
- is intended to be the cheap preview step before install

### `forge codex install`

```sh
forge codex install [--asset agents|rules]... [--target user|path:<abs-path>] [--source repo|release] [--repo-path <path>] [--json]
```

Writes the selected Forge-managed Codex user assets into the selected target root.

Behavior:

- writes only the selected managed files
- creates parent directories when needed
- leaves unrelated files in the target tree untouched
- returns `installed`, `updated`, or `unchanged` per managed asset
- keeps `~/.codex/config.toml`, auth state, session history, and plugin caches out of scope

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

Codex user-config lifecycle and ownership boundaries are documented in `docs/codex.md`.

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

- format: `YYYYMMDD.0.N`
- first release on April 10, 2026: `20260410.0.0`
- second release the same day: `20260410.0.1`
- release dates use the `America/Phoenix` calendar day rather than UTC

This keeps crate versions valid for Cargo while allowing multiple releases per day.
