# forge

First-party AI workflow tools, managed skills, and harness support.

Forge is the tools repo for Ian Cleary's agent workflows. It makes portable agent behavior deterministic and reviewable by owning the source and validation contracts for Codex configuration, Forge-managed skills, AI harnesses, and narrow agent-facing CLIs. The system design centers on the agent operating loop in [docs/agent-operating-loop.md](docs/agent-operating-loop.md): observe, orient, choose, act, verify, and accrete. Claude is part of the product scope, but its managed configuration adapter is not implemented yet.

Forge does not manage general dotfiles, application preferences, operating-system settings, global packages, or language toolchains. Use Nix and Home Manager for those concerns. See [docs/scope.md](docs/scope.md).

## Install

For agent-driven setup, use [install.md](install.md). It defines the tools-repo installation contract: install Forge, verify the binaries, and report the available tool surface.

Nix and Home Manager are the preferred ownership layer for macOS, WSL Arch Linux, and NixOS systems. The initial Nix packaging and Home Manager module are follow-up work. Until they exist, install a release with the portable installer:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.sh | sh
```

Native Windows x64 uses PowerShell:

```powershell
irm https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.ps1 | iex
```

Forge publishes only `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, and
`x86_64-pc-windows-msvc`. WSL uses the Linux installer. Windows ARM and
32-bit Windows are not supported.

The installer:

- resolves the latest published release by default
- downloads the platform archive and checksum manifest from one release tag
- verifies the archive SHA-256 before extraction
- validates the complete archive before replacement
- replaces the complete binary set atomically
- fails without installing if no matching binary artifact is available
- installs Forge-managed skills under `~/.agents/skills`
- installs the Forge-managed Codex baseline under `~/.codex`

Pin a release when deterministic recovery is required:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/20260802.0.0/scripts/install-forge-release.sh | sh -s -- --tag 20260802.0.0
```

Use `--skip-codex` to install binaries and skills without changing the Codex baseline. Use `--build-from-source` only for an explicit development or recovery build. The default deployment path never installs or requires a Rust toolchain.

Use `--verify-attestation` for explicit GitHub provenance verification. It
requires `gh` and fails closed if verification is unavailable or fails. The
POSIX destination is `~/.cargo/bin`. The Windows destination is
`%LOCALAPPDATA%\Forge\bin`; neither installer changes `PATH` automatically.

## Managed AI surfaces

Forge directly supports:

- portable user-scoped agent skills
- Codex user policy and targeted configuration fragments
- AI harnesses such as restartable loops
- local Codex session search through `codex-threads`
- narrow agent-facing Linear and Slack CLIs
- skill-backed artifact tools such as Mermaid and bytefield rendering

The companion `iancleary/skills` repository is the portable instruction distribution experiment. Use that repo to teach agents when to reach for Forge workflows; use this repo to install, update, verify, and release the underlying toolbelt.

Machine/profile policy for external skill sources belongs in Forge. The planned policy manager is documented in [docs/policy.md](docs/policy.md); it will reconcile tools repos, skills repos, product skill packs, and product CLIs from one machine policy file. Over time, portable non-Forge-CLI skills should move from Forge-managed release payloads to `iancleary/skills` and be installed by policy when desired.

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
