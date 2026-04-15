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
- Slack agent token-source presence via `SLACK_AGENT_API_TOKEN`, `~/.config/forge/slack-agent/config.toml`, and `~/.config/forge/slack-agent/token`
- Slack query token-source presence via `SLACK_QUERY_API_TOKEN`, `~/.config/forge/slack-query/config.toml`, and `~/.config/forge/slack-query/token`
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
- file-based auth tools such as `linear`, `slack-query`, and `slack-agent` should report configured token sources, not claim a verified logged-in session
- for those file-based tools, doctor should surface the exact CLI commands and docs to use next, such as `linear config`, `linear auth login`, `slack-query auth login`, and `slack-agent auth login`
- Windows install hints should prefer `winget` for `git` and `gh`
- Windows upgrade hints should prefer `winget upgrade --id ...` for `git` and `gh`
- on macOS and Linux, use `cargo` for tools that support it such as `rg` and `jq`

### `forge permissions check`

```sh
forge permissions check [--json]
```

Audits known Forge-managed config directories and secret files, including:

- `~/.config/forge/`
- `~/.config/forge/slack-agent/`
- `~/.config/forge/slack-query/`
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
forge self update-check [--json]
```

Checks whether the local Forge install is out of date.

Behavior:

- always checks the latest Forge release live
- checks Forge release drift and Forge-managed skill drift
- checks `mainline` Forge-managed skill drift by default
- uses the installed Forge release as the canonical source for managed skills

### `forge self update`

```sh
forge self update [--build-from-source] [--json]
```

Updates to the newest published Forge release and reconciles Forge-managed surfaces.

Important boundary:

- `forge self update-check` compares the running Forge version to the newest release tag from the Forge repo
- `forge self update` resolves the target tag's binary list from that tag's release installer
- `forge self update` resolves the target tag's tool contract from `config/release-tools.toml`
- `forge self update` resolves the target tag's skill contract from `config/release-skills.toml`
- `forge self update` prefers a verified platform release artifact when one is published for the current platform
- `forge self update` verifies artifact SHA-256 before install
- `forge self update` falls back to a tagged source build with `--locked` when verified artifacts are unavailable or when `--build-from-source` is passed
- checksum mismatch is a hard failure; Forge does not silently weaken the trust model after verification fails
- in human-readable mode, `forge self update` shows a spinner while long-running steps are in progress
- in interactive human mode, `forge self update` prompts for each unmanaged skill collision to overwrite or skip
- in JSON or other non-interactive mode, unmanaged skill collisions still fail explicitly
- after install, `forge self update` migrates declared legacy tool config dirs, removes declared legacy binaries when their replacements exist, and removes declared obsolete root files under `~/.config/forge`
- after install, `forge self update` migrates declared legacy Forge-managed skill installs and updates their recorded names in local state
- when a release update installs a new Forge binary, the newly installed binary performs release-sourced skill and Codex reconciliation so embedded payloads match the target tag
- after source update, Forge reconciles managed skills and reapplies the managed Codex baseline

### `forge version`

```sh
forge version [--json] [--update]
```

Shows release/version metadata for the running Forge binary, including:

- `release_version`
- `latest_version`
- `update_available`
- `git_hash`
- binary path
- platform

Behavior:

- in human mode, when `update_available` is true, Forge prompts before running `forge self update`
- in non-interactive contexts (`--json`, no tty), `forge version` reports availability but does not prompt
- `--update` runs `forge self update` immediately when an update is available (useful for automation)

Examples:

```sh
# Informational output
forge version

# Machine-readable output
forge version --json

# Directly trigger an update check/apply when newer release is available
forge version --update
```

If `forge self update` reports an unmanaged collision, take ownership once (example):

```sh
forge skills install learning-systems --source release --force-unmanaged
forge self update
```

### `forge dev install`

```sh
forge dev install [--repo-path <path>] [--no-force] [--json]
```

Installs Forge binaries from a local checkout for explicit development workflows.

Behavior:

- defaults `--repo-path` to the current Git worktree root when available, otherwise the current directory
- reads the managed binary list from `scripts/install-forge-release.sh` in that checkout
- builds the managed binaries in one workspace release build and installs them into `~/.cargo/bin`
- `--no-force` means do not overwrite an existing installed binary
- does not affect `forge self update` source-of-truth behavior

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
- `--source repo` requires `--repo-path <path>`
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

Root Forge config is intentionally narrow.

`forge self update-check` and `forge self update` do not use it.

Current supported root setting:

```text
~/.config/forge/config.toml
```

Example:

```toml
forge_repo_install_subpath = ".agents/skills-installed"
```

Skill lifecycle configuration is documented in `docs/forge-skills.md`.

Codex user-config lifecycle and ownership boundaries are documented in `docs/codex.md`.

Optional override:

- `FORGE_CONFIG_DIR`

State file:

```text
~/.config/forge/state.toml
```

## Notes

- `forge self update-check` is safe to run frequently as a live network check
- `forge self update` is explicit on purpose
- Forge-managed skills are deployed artifacts, not peer sources of truth
- local-checkout workflows should use `forge dev install` and explicit `--repo-path` flags rather than root-config inference

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
