---
name: create-release-process
description: "Create, audit, or update a repo-local release process by discovering the repo's versioning, notes, validation, and publish requirements, then leaving a checked-in runner plus local agent instructions that future releases can execute deterministically."
---

# Create Release Process

Use this skill to establish or revise a repo-local release process. The goal is not to run one release by hand. The goal is to leave behind a deterministic local pattern that future agent runs can follow.

This is a portable Forge-managed skill. It should adapt to the target repo instead of imposing Forge's own release policy.

## Use This When

- the user asks to create, define, audit, repair, or change a release workflow
- the repo does not have a deterministic release runner yet
- the repo's release process exists but does not match its current versioning, notes, validation, or publishing requirements
- ordinary release requests keep requiring reconstructed shell sequences

## Do Not Use This When

- the user is asking to cut or publish the next normal release and the repo already has a working release runner
- the task is only to query a current or next version through an existing read-only command
- the user is correcting an already-published release and needs explicit repair steps

For ordinary release execution, use `cut-release` after this workflow exists.

## Discovery

Decide the release process from repo evidence, not habit.

Inspect:

- local agent instructions: `AGENTS.md`, `CLAUDE.md`, `README.md`, `docs/release.md`
- package manager and version source: `Cargo.toml`, `pyproject.toml`, `package.json`, workspace manifests
- versioning scheme: SemVer, CalVer, date-based tags, or another documented repo-specific policy
- lockfiles: `Cargo.lock`, `uv.lock`, `pnpm-lock.yaml`, `package-lock.json`
- task runner: `justfile`, `Makefile`, package scripts, repo scripts
- validation path: tests, checks, lint, build, release artifact generation
- release notes source: generated changelog, curated markdown file, GitHub release notes, conventional commits, or repo-specific template
- publish mode: GitHub release, package registry publish, artifact upload, deploy, or manual handoff

Ask the user only when the repo evidence does not determine a safe policy.

## Tailoring Contract

The deployed skill supplies the pattern. The target repo supplies the contract.

Make the repo-local release process explicit about:

- versioning scheme and whether the next version can be inferred
- required version argument shape when inference is unsafe
- release notes format and source, such as `--notes-file`, generated notes, or a checked-in template
- files the runner may mutate, including manifests and lockfiles
- validation commands that must pass before publishing
- branch, clean-tree, tag, and remote requirements
- final public-facing action, such as `gh release create`, registry publish, deploy, or manual stop point
- dry-run behavior and read-only version query commands, when useful

## Implementation Pattern

Prefer the repo's existing task runner.

If a `justfile` exists, prefer a `just cut-release` recipe backed by a checked-in script. Otherwise use the closest local convention, such as `scripts/cut-release.sh`, `make release`, or a package script.

When creating or updating the workflow:

- create or update a checked-in release runner
- make `--dry-run` available for previewing the mutating sequence
- add read-only version query flags when useful, such as `--print-current-version` and `--print-next-version`
- route ordinary release requests through a `cut-release` execution skill when the repo wants agent routing
- update local agent instructions so future sessions know which skill and runner to use
- update release docs when the repo has a release document

Keep the runner narrow and auditable. Do not hide broad automation behind prompts.

## Forge Repo Contract

When this skill is used inside Forge itself:

- read `docs/release.md` before changing the release flow
- prefer `just cut-release` as the default entrypoint
- use `just cut-release --print-current-version` for a read-only current-version query
- use `just cut-release --print-next-version` for a read-only next-version query
- use `just cut-release --dry-run` before the real release when validating the next version or sequence
- the script owns `Cargo.lock` and all workspace crate manifests under `crates/*/Cargo.toml`
- omitted `--version` resolves the next Phoenix-date CalVer from fetched git tags for the current Phoenix day
- the final publish step is `gh release create`
- if the Forge release flow changes, update `scripts/cut-release.sh`, `docs/release.md`, `AGENTS.md`, this skill, and the `cut-release` skill together

## Output

Leave behind:

- the checked-in release runner or task recipe
- local instructions describing maintenance vs. execution
- release docs, if the repo has them
- a clear note on dry-run, versioning, notes, validation, and publishing behavior

## Safety

Release workflows are public-facing. Prefer inspect, dry-run, diff, then apply.

Do not publish, push, tag, or upload packages while creating or repairing the workflow unless the user explicitly asks for an actual release.
