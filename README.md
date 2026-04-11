# forge

Agent-friendly CLIs built as Rust binaries.

## Layout

- `docs/` contains product and design specs for each CLI
- `crates/` contains Rust crates for each CLI implementation

Key design notes:

- `docs/agent-friendly-clis.md` for the cross-repo CLI contract
- `docs/algorithm.md` for the Forge design sequence: question, delete, simplify, accelerate, automate

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

## Install For Codex

If you want Codex to use these CLIs and the Forge-managed consumer skills outside local development, install the binaries and then install the skills into your Codex user skills directory at `~/.agents/skills` unless you override it in Forge config.

From a local checkout:

```sh
cargo install --path crates/forge
cargo install --path crates/slack-cli
cargo install --path crates/codex-threads
cargo install --path crates/linear
forge skills install --all --source repo --target user
```

From an installed Forge release or after pointing an agent at this repo:

```sh
forge skills install --all --target user
forge skills status
```

Useful commands:

```sh
forge skills list --source all
forge skills validate --all
forge skills diff linear-cli --target user
forge skills status --scope all
forge self update-check
forge self update
```

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

## Recommended Codex Companions

Forge covers the Forge-authored CLIs and their managed skills. For adjacent work, a minimal high-value Codex setup should also include:

- `openai-docs` for OpenAI product, model, and API questions that need current official documentation
- GitHub plugin skills plus the `gh` CLI for issues, PR review threads, CI triage, and release-adjacent repository workflows

These are recommended companions, not part of Forge's managed skill lifecycle. Keep the boundary narrow:

- use Forge skills for Forge-authored CLIs and local Forge management
- use `openai-docs` when the task is about OpenAI products rather than the Forge toolchain
- prefer `gh`-driven GitHub workflows and GitHub plugin skills when the task is about repository hosting, PRs, issues, reviews, or CI
- avoid direct use of the native GitHub Codex app path when an equivalent `gh` or plugin-skill workflow exists, because it is less deterministic and more likely to trigger extra permission prompts

## Forge Command Policy

For Forge-managed tools, prefer the Forge CLI first and use shell tools as fallbacks, not as the primary product interface.

- use Forge commands when the task is part of a stable contract and the output should be reusable, low-token, and deterministic
- use `jq` for one-off local reshaping after a CLI already returned the right data
- use `rg` for repository and local-file exploration, not as a substitute for domain-specific CLI reads
- when the same `jq` cleanup keeps recurring, treat that as a signal to add a narrow Forge flag, subcommand, or output mode

The pattern is: repeated pain in agent runs should first be observed, then named as a narrow task, then folded into Forge as the smallest stable primitive that removes the repeated shell shaping.
