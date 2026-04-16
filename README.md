# forge

Agent-friendly CLIs built as Rust binaries.

## Install (User-First)

On supported macOS and Linux targets, the fast release install path uses attested release artifacts. That fast path uses local attestation verification through GitHub CLI.

Rust and Cargo are still required when:

- you pass `--build-from-source`
- no attested release artifact is available for your platform
- `gh` attestation verification support is unavailable locally, which causes an explicit tagged source-build fallback (artifact path disabled; source build only)
- you are developing Forge from source

This section is the primary path: using Forge as an installed tool on a machine.

Recommended patterns:

1. simplest bootstrap on a new machine
2. deterministic bootstrap pinned to a specific release when needed
3. steady-state updates through `forge self update-check` and `forge self update`

### 1. Simplest Bootstrap

Use `curl` as the bootstrap path on a new machine:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.sh | sh
```

That installer:

- resolves the latest published Forge release tag by default
- re-executes the installer script from the exact release tag it is about to install
- uses attested release artifacts for supported platforms when local attestation verification is available
- verifies artifact SHA-256 against the published release checksums
- verifies the GitHub release attestation before installing an artifact
- falls back to a tagged source build with `--locked` when attestation verification cannot run or no attested artifact is available (artifact path disabled, source build only)
- installs Forge-managed skills into `~/.agents/skills` by default
- installs the managed Codex baseline into `~/.codex/` by default

### 2. Deterministic Bootstrap

If you want a deterministic install pinned to a specific release:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/20260412.0.7/scripts/install-forge-release.sh | sh -s -- --tag 20260412.0.7
```

If you want the binaries but not the Codex baseline:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.sh | sh -s -- --skip-codex
```

After installation, a good first verification step is:

```sh
forge doctor
forge skills status
forge codex diff
```

If you want to verify a downloaded release archive against the published GitHub provenance attestation:

```sh
gh attestation verify ./forge-20260412.0.7-aarch64-apple-darwin.tar.gz \
  --repo iancleary/forge \
  --source-ref refs/tags/20260412.0.7 \
  --signer-workflow iancleary/forge/.github/workflows/release-artifacts.yml \
  --predicate-type https://slsa.dev/provenance/v1
```

That command is the strictest and explicit check used for release artifact trust.

### 3. Steady-State Updates

After bootstrap, prefer the Forge-managed update path:

```sh
forge self update-check
forge self update
```

In release mode, that path now:

- checks GitHub release tags by querying the Forge repo tags
- uses attested release artifacts for supported platforms when local attestation verification is available
- verifies artifacts against the published release manifest, checksums, and GitHub release attestation
- falls back to a tagged source build with `--locked` when attestation verification cannot run or an attested artifact is unavailable (artifact path disabled, source build only)
- reconciles Forge-managed skills
- reapplies the managed Codex baseline

The `curl` installer remains useful for first install and recovery, but it is no longer the steady-state update path.

## What Gets Installed

Forge installs two managed surfaces by default:

- skills under `~/.agents/skills` (`forge skills install --all`)
- Codex user baseline under `~/.codex/` (`forge codex install`)

These are intentionally narrower than taking ownership of all local Codex state.

## Common Commands

The most common commands and their intent:

- `forge doctor`  
  checks your local Forge health and state files.
- `forge self update-check`  
  compares the installed Forge version to the latest GitHub release.
- `forge self update [--build-from-source]`
  updates Forge and managed assets when behind.
- `forge version [--json] [--update]`  
  reports the current hash/version plus release state; `--update` runs `forge self update` directly when an update is available.
- `forge skills install --all`  
  reconciles all managed skills from the release catalog.
- `forge skills status`  
  shows managed/unmanaged and health status for skills.
- `forge codex install`  
  installs the managed Forge baseline under `~/.codex`.
- `forge codex diff`  
  compares managed `~/.codex` files with installed versions.
- `forge codex render`  
  applies templated policy artifacts from managed files into local state.

Quick reference:

```sh
forge doctor
forge self update-check
forge self update
forge version
forge skills status
forge codex diff
```

Common maintenance commands:

```sh
forge skills list --source all
forge skills validate --all
forge skills diff design-algorithm --target user
forge skills diff linear-cli --target user
forge skills status --scope all
forge codex render
forge skills install --all --source repo
forge skills revert --all --target user
```

## Codex Notes

If you want Codex to use these CLIs and the Forge-managed consumer skills outside local development, install the binaries and then install the skills into the Codex `USER` skill directory at `~/.agents/skills`.

If Forge is your first-party Codex source of truth, treat the repo docs plus the Forge-managed skills as the canonical portable policy surface. Repo-local `AGENTS.md` guidance can reinforce that behavior, but it should not be the only place where cross-repo routing rules live.

From a local checkout:

```sh
just install-dev-local
forge skills install --all --source repo
```

When adding or removing CLIs, update the embedded list in `scripts/install-forge-release.sh` and ensure the repo check passes:

```sh
just install-list-check
```

From an installed Forge release or after pointing an agent at this repo:

```sh
forge skills install --all
forge skills status
forge codex diff
forge codex install
```

Forge-managed Codex user config is intentionally narrower than the skills surface. In v1, `forge codex render`, `forge codex diff`, and `forge codex install` manage only:

- `~/.codex/AGENTS.md`
- `~/.codex/rules/user-policy.rules`

They do not take ownership of live `~/.codex/config.toml`, session history, auth state, or plugin caches.

The important mental model:

- `forge` owns the portable, user-scoped Codex behavior you want available everywhere
- skill `description` frontmatter is part of the routing contract, not incidental prose
- router skills such as `forge-tools` tell Codex which narrower skill to use next
- machine-local or private details should stay out of the Forge-managed surface

If Forge reports that `~/.config/forge/state.toml` cannot be parsed after a local schema change during development, remove that file and reinstall the managed skills you want Forge to track.

If you use a non-user target as part of your primary managed install set, mark it explicitly:

```sh
forge skills install --all --source repo --target path:/opt/forge-skills --target-role mainline
forge skills status --target path:/opt/forge-skills
```

If you temporarily install skills from a local checkout and want to switch back to the standard release-backed install:

```sh
forge skills revert --all --target user
```

## Development

Local development lifecycle is documented in `docs/development.md`.

## Reference

Repo layout:

- `docs/` contains product and design specs for each CLI
- `crates/` contains Rust crates for each CLI implementation

Current CLIs:

- `forge` for shared config and self-management
- `codex-threads` for local Codex session archive search and retrieval
- `slack-agent` for stricter assistant Slack workflows
- `slack-query` for Slack research workflows
- `linear` for Linear issue, project, and milestone workflows

Internal support crates:

- `slack-core` as a shared library crate for Slack auth/config/client code used by `slack-query` and `slack-agent`

Key design notes:

- `docs/agent-friendly-clis.md` for the cross-repo CLI contract
- `docs/algorithm.md` for the Forge design sequence: question, delete, simplify, accelerate, automate
- `docs/codex.md` for Forge as the first-party source of truth for Codex skills, routing, and portable policy

Versioning:

- format: `YYYYMMDD.0.N` (America/Phoenix calendar day)

## Recommended Codex Companions

Forge covers the Forge-authored CLIs and their managed skills. For adjacent work, a minimal high-value Codex setup should also include:

- `openai-docs` for OpenAI product, model, and API questions that need current official documentation
- GitHub plugin skills plus the `gh` CLI for issues, PR review threads, CI triage, and release-adjacent repository workflows

These are recommended companions, not part of Forge's managed skill lifecycle. Keep the boundary narrow:

- use Forge skills for Forge-authored CLIs and local Forge management
- use `openai-docs` when the task is about OpenAI products rather than the Forge toolchain
- prefer `gh`-driven GitHub workflows and GitHub plugin skills when the task is about repository hosting, PRs, issues, reviews, or CI
- for substantial issue or PR bodies, prefer local markdown files with `gh ... --body-file` over inline multiline shell strings
- avoid direct use of the native GitHub Codex app path when an equivalent `gh` or plugin-skill workflow exists, because it is less deterministic and more likely to trigger extra permission prompts

## Forge Command Policy

For Forge-managed tools, prefer the Forge CLI first and use shell tools as fallbacks, not as the primary product interface.

- use Forge commands when the task is part of a stable contract and the output should be reusable, low-token, and deterministic
- use `jq` for one-off local reshaping after a CLI already returned the right data
- use `rg` for repository and local-file exploration, not as a substitute for domain-specific CLI reads
- when the same `jq` cleanup keeps recurring, treat that as a signal to add a narrow Forge flag, subcommand, or output mode

The pattern is: repeated pain in agent runs should first be observed, then named as a narrow task, then folded into Forge as the smallest stable primitive that removes the repeated shell shaping.
