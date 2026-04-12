# AGENTS.md

Operating guidance for contributors and coding agents working in `forge`.

## Purpose

`forge` is a Rust workspace for agent-friendly CLIs.

The repo has two layers:

- `docs/` for product specs, command contracts, and workflow notes
- `crates/` for CLI implementations

Do not treat `docs/` as the implementation area.

## Before Changing Code

- read the relevant CLI spec in `docs/`
- read [docs/algorithm.md](docs/algorithm.md) when shaping or reviewing non-trivial changes
- inspect the target crate in `crates/`
- check `git status`
- run `cargo check` before finalizing changes

## Implementation Rules

- use Rust for CLI implementations
- keep one binary crate per CLI under `crates/`
- prefer explicit, typed command contracts over thin wrappers around prompts or shell scripts
- keep output JSON-first and stable for agent consumption
- use singular top-level resource nouns where possible:
  - `team`
  - `issue`
  - `project`
  - `milestone`
- aliases are fine when they improve compatibility, but the primary command surface should stay consistent

## Auth And Config

- prefer local config directories over env vars as the default user setup
- keep env vars as overrides for ad hoc use and CI
- do not commit secrets, copied tokens, or account-specific setup artifacts
- auth details belong in each CLI's own doc, not in this file

## Safety

- reads should be safe by default
- writes should use explicit verbs
- destructive actions should require an explicit flag such as `--force`
- avoid hidden side effects such as implicit Git actions, browser launching, or background mutations

## Docs Pattern

For each new CLI:

- add or update a dedicated spec in `docs/<tool>.md`
- document command surface, auth model, safety rules, and examples
- keep docs aligned with the real implemented behavior

## Verification

Before committing:

- run `cargo check`
- if the CLI talks to a real external API, verify at least the core read path live when practical
- prefer doing heavy lifting inside the CLI rather than leaving filtering or normalization to the LLM

## Adding A New CLI

When you add, remove, or rename a binary CLI crate under `crates/`:

- update the embedded `FORGE_BINARIES` list in `scripts/install-forge-release.sh`
- ensure the crate directory name matches the binary name (`crates/<bin>/src/main.rs`)
- run `just install-list-check` (fails if a binary crate exists but is not listed)

## Versioning And Releases

- use semver-compatible CalVer: `YYYY.MMDD.N`
- keep crate versions aligned across the workspace
- current release flow is:
  - `git push origin main`
  - `gh release create <version> --target main --title <version> --generate-notes --latest`

## Scope Discipline

- build narrow, composable primitives first
- keep assistant-specific workflow CLIs separate from shared general-purpose CLIs
- do not add broad automation or repo-specific behavior unless it is explicitly part of the CLI contract

## Design Algorithm

Use the repo algorithm in `docs/algorithm.md` when deciding what to build:

- question every requirement
- delete parts or process before optimizing
- simplify and optimize only what remains necessary
- accelerate cycle time after the contract is clean
- automate last

For Forge, this means avoiding automation of noisy or speculative workflows. Prefer deleting shell shaping, duplicated policy, or unnecessary command surface before adding new primitives.
