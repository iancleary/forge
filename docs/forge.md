# Forge CLI

This document defines the top-level `forge` manager CLI.

## Goal

Manage Forge's first-party AI workflow surfaces. These surfaces include portable skills, Codex configuration, harness support, release lifecycle, and narrow helpers used by managed skills.

Forge does not manage general packages, language toolchains, dotfiles, terminal preferences, editor preferences, or operating-system state. Nix and Home Manager own those concerns on managed systems. See [scope.md](scope.md).

Claude support is within the product boundary but is not implemented by the current CLI.

## Output contract

Commands use human-readable output by default. `--json` emits a compact stable envelope for agent consumption. Errors use the same mode selected by the caller.

## Commands

### `forge doctor`

```sh
forge doctor [--json]
```

Checks whether Forge and its supported agent workflows can run. Checks include required commands, configured Forge integration credentials, the Forge config directory, and release-attestation capability.

Doctor reports missing dependencies and remediation. It does not install or update global packages.

### `forge version`

```sh
forge version [--json] [--update]
```

Reports the running release, latest release, update state, Git hash, binary path, and platform. `--update` invokes the portable Forge self-update path when an update exists.

### `forge self update-check`

```sh
forge self update-check [--json]
```

Checks release drift, managed skill drift, and managed Codex asset drift. It does not mutate state.

### `forge self update`

```sh
forge self update [--build-from-source] [--attestation-failure <prompt|source|fail>] [--json]
```

Updates a non-Nix Forge installation from an attested release artifact or a tagged source build. It reconciles installed Forge-managed skills and Codex assets after updating the binary.

Nix-managed installations must use Nix to update binaries and packages. Forge asset rendering, validation, status, and diff operations remain valid in that environment. Nix ownership detection and Home Manager activation are not implemented yet.

### `forge dev install`

```sh
forge dev install --repo-path <path> [--no-force] [--json]
```

Builds and installs Forge workspace binaries from a local checkout for development.

### `forge permissions check|fix`

```sh
forge permissions check [--json]
forge permissions fix [--json]
```

Checks or repairs permissions only for Forge-owned configuration and credential paths. These commands do not manage unrelated user files.

### `forge skills`

```sh
forge skills list
forge skills status
forge skills validate [<skill>|--all]
forge skills install [<skill>|--all]
forge skills diff <skill>
forge skills revert [<skill>|--all]
```

Lists, validates, compares, and installs Forge-managed agent skills. The detailed source, target, collision, and state contracts are in [forge-skills.md](forge-skills.md).

Home Manager should eventually materialize release-selected skills declaratively. The imperative install commands remain the portable non-Nix path and the development path.

### `forge codex`

```sh
forge codex render
forge codex diff
forge codex install
forge codex config diff
forge codex config install
```

Renders, compares, and installs Forge-managed Codex user assets. These commands own only the files and targeted fragments defined in [codex.md](codex.md).

### `forge bytefield`

```sh
forge bytefield install
forge bytefield render --source <path> --output <path>
```

Provides the pinned bytefield renderer used by the managed bytefield skill. This is a skill-backed AI artifact primitive, not a general package manager.

## Removed commands

The following commands were removed because their owner is the machine configuration layer:

- `forge tool update`
- `forge preference check|diff|apply`

Use Nix and Home Manager for packages, toolchains, dotfiles, and application preferences. Forge must not add replacement commands for these concerns.
