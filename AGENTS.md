# AGENTS.md

Operating guidance for contributors and coding agents working in `forge`.

## Purpose

`forge` is a Rust workspace for first-party AI workflow tools, skills, harnesses, and agent-friendly CLIs.

The repo has two layers:

- `docs/` for product specs, command contracts, and workflow notes
- `crates/` for CLI implementations

Do not treat `docs/` as the implementation area.

Nix and Home Manager own packages, toolchains, general dotfiles, application preferences, and host state. Forge owns portable AI workflow behavior. Read [docs/scope.md](docs/scope.md) before adding a new top-level management surface.

## Writing Guidance

Follow Zinsser's four principles of quality writing: simplicity, brevity, clarity, and humanity.

The rule is simple. Use short sentences. Use the active voice. Give each word one meaning. Cut the clutter. Keep the writing warm and human: a person wrote it, not a manual.

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

- use `source-driven-development` when a change depends on external API, library, CLI, or vendor behavior that should be verified from primary sources
- use `debugging-and-error-recovery` for failing tests, broken commands, unexpected behavior, or repeated fix attempts
- use `api-and-interface-design` when adding or changing public CLI/API/module/JSON contracts after the need survives design review
- use `security-and-hardening` when a change touches trust boundaries such as input, auth, secrets, files, shell commands, network calls, permissions, or persisted state
- use `test-strategy` to choose focused proof for features, bug fixes, refactors, and regressions without forcing ceremony
- use `code-simplification` when behavior is known and the goal is to reduce complexity while preserving proof
- use `documentation-and-adrs` when a change affects durable docs, workflow policy, command contracts, or architecture decisions
- use `librarian` when the user points you at a remote git repository as reference, unless you are developing Forge itself and need the active Forge checkout
- use the Forge-managed `create-release-process` skill when you are establishing, auditing, or changing the Forge release workflow itself
- use the Forge-managed `release-runner` or `cut-release` skill for an ordinary request to publish a Forge release; those skills should execute `forge release` through `release.toml` rather than reconstructing the flow by hand
- use `forge release current-version` when you need the current workspace release version without starting the release flow
- use `forge release next-version` when you need the inferred next release version without starting the release flow
- prefer `forge release run --dry-run --json` before `forge release run --apply --json` when validating the next version or the enforced sequence
- the Forge release runner owns workspace version bumps in `Cargo.lock` and all `crates/*/Cargo.toml` manifests
- the Forge release runner pushes the version commit and dispatches the release-artifacts workflow; that workflow must pass checks, build every supported platform, stage and verify artifacts, and create the tag and GitHub release only in its final step
- the deployed release-process skills provide portable defaults, while this repo's `AGENTS.md`, `docs/release.md`, `release.toml`, and the built-in `forge release` runner tailor the Forge-specific CalVer, notes, validation, and publish behavior
- if the release flow changes, update `release.toml`, [docs/release.md](docs/release.md), and the release-process skills together

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

For adapted third-party skills, also add `THIRD_PARTY_NOTICES.md` with the upstream repo, inspected commit, upstream path, and license notice. Use [docs/skill-intake.md](docs/skill-intake.md) before importing upstream skills, commands, hooks, or helper scripts.

## Versioning And Releases

- use semver-compatible CalVer: `YYYYMMDD.0.N`
- for this repo specifically, releases use Phoenix-date CalVer such as `20260415.0.0`
- keep crate versions aligned across the workspace
- omitted `--version` can be inferred safely from fetched git tags for the current Phoenix calendar day
- current release flow is:
  - `forge release run --apply --json`
- read-only version queries are:
  - `forge release current-version`
  - `forge release next-version`
- normal validation path is:
  - `forge release check --json`
  - `forge release plan --json`
  - `forge release run --dry-run --json`
  - `forge release run --apply --json`
- release ordering is checks, all-platform builds, artifact assembly and verification, then tag and GitHub release creation
- use `release.toml` through `forge release` instead of reconstructing release commands by hand
- use the `create-release-process` skill to maintain the release process; use the `release-runner` or `cut-release` skill to execute the release through `forge release`

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
