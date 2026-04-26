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

## Repo Skills

- use the Forge-managed `create-release-process` skill when you are establishing, auditing, or changing the Forge release workflow itself
- use the Forge-managed `cut-release` skill for an ordinary request to publish a Forge release; that skill should execute the checked-in runner via `just cut-release` / `scripts/cut-release.sh` rather than reconstructing the flow by hand
- use `just cut-release --print-current-version` when you need the current workspace release version without starting the release flow
- use `just cut-release --print-next-version` when you need the inferred next release version without starting the release flow
- prefer `just cut-release --dry-run` before the real release when validating the next version or the enforced sequence
- the release script owns workspace version bumps in `Cargo.lock` and all `crates/*/Cargo.toml` manifests
- the deployed release-process skills provide portable defaults, while this repo's `AGENTS.md`, `docs/release.md`, `just cut-release`, and `scripts/cut-release.sh` tailor the Forge-specific CalVer, notes, validation, and publish behavior
- if the release flow changes, update the script, [docs/release.md](docs/release.md), and the release-process skills together

## Adding A New CLI

When you add, remove, or rename a binary CLI crate under `crates/`:

- update the embedded `FORGE_BINARIES` list in `scripts/install-forge-release.sh`
- ensure the crate directory name matches the binary name (`crates/<bin>/src/main.rs`)
- run `just install-list-check` (fails if a binary crate exists but is not listed)

## Adding A New Managed Skill

When adding a new Forge-managed skill, update these three locations together:

- add the skill directory and `SKILL.md` under `.agents/skills/<skill-name>/`
- add the skill to `config/release-skills.toml`
- add `embedded_skill!("<skill-name>")` to `release_skills()` in `crates/forge/src/main.rs`

## Versioning And Releases

- use semver-compatible CalVer: `YYYYMMDD.0.N`
- for this repo specifically, releases use Phoenix-date CalVer such as `20260415.0.0`
- keep crate versions aligned across the workspace
- omitted `--version` can be inferred safely from fetched git tags for the current Phoenix calendar day
- current release flow is:
  - `just cut-release`
- read-only version queries are:
  - `just cut-release --print-current-version`
  - `just cut-release --print-next-version`
- normal validation path is:
  - `just cut-release --dry-run`
  - `just cut-release`
- use the repo-local release script instead of reconstructing release commands by hand
- use the `create-release-process` skill to maintain the release process; use the `cut-release` skill to execute the release through `just cut-release`

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
