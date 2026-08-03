# forge

First-party AI workflow tools, skills, and harness support.

Forge makes portable agent behavior deterministic and reviewable. It owns the source and validation contracts for Codex configuration, shared agent skills, AI harnesses, and narrow agent-facing CLIs. Claude is part of the product scope, but its managed configuration adapter is not implemented yet.

Forge does not manage general dotfiles, application preferences, operating-system settings, global packages, or language toolchains. Use Nix and Home Manager for those concerns. See [docs/scope.md](docs/scope.md).

## Install

Nix and Home Manager are the preferred ownership layer for macOS, WSL Arch Linux, and NixOS systems. The initial Nix packaging and Home Manager module are follow-up work. Until they exist, install a release with the portable installer:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.sh | sh
```

The installer:

- resolves the latest published release by default
- downloads the platform archive and checksum manifest from one release tag
- verifies the archive SHA-256 before extraction
- fails without installing if no matching binary artifact is available
- installs Forge-managed skills under `~/.agents/skills`
- installs the Forge-managed Codex baseline under `~/.codex`

Pin a release when deterministic recovery is required:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/20260802.0.0/scripts/install-forge-release.sh | sh -s -- --tag 20260802.0.0
```

Use `--skip-codex` to install binaries and skills without changing the Codex baseline. Use `--build-from-source` only for an explicit development or recovery build. The default deployment path never installs or requires a Rust toolchain.

## Managed AI surfaces

Forge directly supports:

- portable user-scoped agent skills
- Codex user policy and targeted configuration fragments
- AI harnesses such as autoreview, autoresearch, and restartable loops
- local Codex session search through `codex-threads`
- narrow agent-facing Linear and Slack CLIs
- skill-backed artifact tools such as Mermaid and bytefield rendering

Claude configuration and shared cross-agent policy are intended first-party surfaces. Their command and file contracts must be designed before implementation.

## Common commands

```sh
forge doctor
forge version
forge self update-check
forge skills status
forge skills validate --all
forge codex diff
codex-threads --json sync
linear --help
slack-query --help
slack-agent --help
mermaid --help
```

`forge self update` is the portable non-Nix update path. A future Nix package must leave binary and package updates to Nix while retaining Forge's read, render, diff, validation, and asset contracts.

## Development

Read the relevant spec under `docs/` before changing a command. Then run:

```sh
cargo check
cargo test
just install-list-check
```

Install a development checkout with:

```sh
cargo run -p forge -- dev install --repo-path "$(pwd)"
forge skills install --all --source repo --repo-path "$(pwd)"
```

## Workspace binaries

- `forge`: managed AI assets, releases, diagnostics, and skill-backed helpers
- `codex-threads`: local Codex session retrieval
- `linear`: agent-friendly Linear operations
- `slack-query`: deterministic Slack retrieval
- `slack-agent`: assistant-oriented Slack operations
- `mermaid`: deterministic Mermaid rendering

All read-oriented CLI surfaces should provide stable compact JSON for agents and useful human output by default.
