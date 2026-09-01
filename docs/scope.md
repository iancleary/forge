# Forge Product Scope

## Decision

Forge is the first-party source and toolchain for portable AI workflow behavior.

Nix and Home Manager own machine state. Forge owns AI workflow behavior.

Forge should be designed as one agent operating loop, not as a bag of unrelated utilities. The shared loop is documented in [agent-operating-loop.md](agent-operating-loop.md): observe, orient, choose, act, verify, and accrete.

## Context

Forge began as a collection of agent-friendly Rust CLIs. It later added global tool updates and targeted application preference management. Those features created overlapping desired-state systems and made the product boundary unclear.

The target environments are:

- macOS with Nix and Home Manager
- WSL Arch Linux with standalone Home Manager
- NixOS and Home Manager test platforms

A declarative machine layer can install packages and materialize files consistently across these environments. Forge does not need to duplicate that responsibility.

## Forge ownership

Forge owns:

- portable Codex configuration and policy
- future portable Claude configuration and policy
- shared agent skills and skill routing
- skill intake, validation, provenance, catalogs, and compatibility
- AI harnesses, evaluation workflows, and restartable loops
- local agent-session retrieval and indexing
- narrow agent-facing CLIs for recurring external-system workflows
- deterministic artifact primitives required by managed skills
- non-Nix release artifacts and a checksum-verified binary installer

An integration belongs in Forge only when it provides a stable reusable primitive that materially improves an AI coding or knowledge-work loop.

That primitive should fit one clear plane:

- context: cheap reads that make current state accurate
- routing: skills, help, and docs that point to the right next abstraction
- execution: explicit commands that act with safe preview and verification paths
- accretion: durable skills, docs, tests, and examples that reduce future agent work

## Machine-layer ownership

Nix and Home Manager own:

- Forge binary installation and version selection on managed systems
- global packages and package upgrades
- Rust, Node, Python, uv, and other language toolchains
- shells, fonts, terminals, editors, and application preferences
- general dotfiles and host-specific configuration
- platform and host composition
- references to secrets, without storing secret values in Forge

## Explicit exclusions

Forge must not provide:

- generic dotfiles management
- arbitrary structured-file merge facilities
- operating-system or application preference management
- global package-manager orchestration
- host provisioning
- an alternative to Home Manager

The fact that an agent can call a tool is not sufficient reason to add that tool to Forge.

## Deployment model

The desired steady-state model is:

```text
Nix or Home Manager
  installs Forge and companion binaries
  selects a Forge release
  materializes managed AI assets

Forge
  defines and validates those assets
  renders and compares application-specific configuration
  supplies portable non-Nix installation paths
```

The initial Nix flake, package definitions, and Home Manager module are follow-up work. Until they exist, the checksum-verified release installers, `forge skills install`, and `forge codex install` remain supported portability paths. Native Windows uses the PowerShell installer and `%LOCALAPPDATA%\Forge\bin`. WSL uses the Linux artifact. Windows ARM and 32-bit Windows are excluded. Source builds are explicit development or recovery operations.

## Consequences

- Global tool update commands are removed.
- Application preference commands are removed.
- The release installer no longer bootstraps unrelated global tools.
- Codex, skills, harnesses, and agent-facing CLIs remain first-party.
- Claude requires an explicit managed-asset contract before implementation.
- New Nix work must consume Forge's asset contracts instead of duplicating their policy.
