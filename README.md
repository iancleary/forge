# forge

Agent-friendly CLIs built as Rust binaries.

## Install (User-First)

Prerequisite: install Rust and Cargo first with `rustup` from <https://rustup.rs>.

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
- installs the Forge binaries listed in `scripts/install-forge-release.sh` from that tagged release source
- installs Forge-managed skills into `~/.agents/skills` by default
- installs the managed Codex baseline into `~/.codex/` by default

### 2. Deterministic Bootstrap

If you want a deterministic install pinned to a specific release:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.sh | sh -s -- --tag 20260412.0.2
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

### 3. Steady-State Updates

After bootstrap, prefer the Forge-managed update path:

```sh
forge self update-check
forge self update
```

In release mode, that path now:

- checks GitHub release tags by querying the Forge repo tags
- updates installed Forge binaries to the newest tagged release with Cargo when needed
- reconciles Forge-managed skills
- reapplies the managed Codex baseline

The `curl` installer remains useful for first install and recovery, but it is no longer the steady-state update path.

## What Gets Installed

Forge installs two managed surfaces by default:

- skills under `~/.agents/skills` (`forge skills install --all`)
- Codex user baseline under `~/.codex/` (`forge codex install`)

These are intentionally narrower than taking ownership of all local Codex state.

## Key Commands

```sh
forge doctor
forge self update-check
forge self update
forge skills status
forge codex diff
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

Useful commands:

```sh
forge skills list --source all
forge skills validate --all
forge skills diff design-algorithm --target user
forge skills diff linear-cli --target user
forge skills status --scope all
forge codex render
forge codex diff
forge codex install
forge self update-check
forge self update
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
- `slack-cli` for Slack research workflows
- `linear` for Linear issue, project, and milestone workflows

Planned CLIs:

- `openclaw-slack` for stricter OpenClaw Slack workflows

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
