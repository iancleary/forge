---
name: cut-release
description: "Create or update a repo-local release process, then route release work through a checked-in `scripts/cut-release.sh` plus repo-local instructions in `AGENTS.md`, adapted to the repo’s package manager and release situation."
---

# Cut Release

Use this skill to establish or update a repo-local release process. The goal is not just to run a release once. The goal is to leave behind a deterministic local pattern that future agent runs can follow.

Primary commands:

- `just cut-release`
- `just cut-release --dry-run`
- `just cut-release --print-current-version`
- `just cut-release --print-next-version`
- `./scripts/cut-release.sh --version <repo-version>`

In Forge itself, read [docs/release.md](../../../docs/release.md) when you need the full release contract or need to update the documented workflow.

Forge-specific contract today:

- prefer `just cut-release` as the default entrypoint
- use `just cut-release --print-current-version` for a read-only current-version query
- use `just cut-release --print-next-version` for a read-only next-version query
- use `just cut-release --dry-run` before the real release when validating the next version or sequence
- the script owns `Cargo.lock` and all workspace crate manifests under `crates/*/Cargo.toml`
- omitted `--version` resolves the next Phoenix-date CalVer from fetched git tags for the current Phoenix day
- the final publish step is `gh release create`

## Pattern

When a repo does not already have the release pattern in place:

- inspect the repo to identify the package manager, version files, lockfiles, validation command, and release mode
- create `scripts/` if it does not exist
- create `.agents/skills/cut-release/` if it does not exist
- add a repo-local `scripts/cut-release.sh`
- add or update the repo-local `.agents/skills/cut-release/SKILL.md`
- add repo-local instructions in `AGENTS.md` that tell future agents when to use the skill and script

The pattern should stay deterministic:

- a checked-in script is the source of truth for the release flow
- the skill routes release tasks to that script instead of hand-built shell sequences
- the script supports `--dry-run`
- normal use is dry-run first, then real release
- corrections to an already-published release stay explicit and manual

## Detection

Decide the release process from repo evidence, not habit.

Look for:

- package manager and version source: `Cargo.toml`, `pyproject.toml`, `package.json`, workspace files
- versioning scheme: semver, CalVer, date-based tags, or another documented repo-specific scheme
- lockfiles: `Cargo.lock`, `uv.lock`, `pnpm-lock.yaml`, `package-lock.json`
- task runner: `justfile`, `Makefile`, `package.json` scripts, repo scripts
- release mode: checked-in script already exists, GitHub release is manual via `gh`, or the repo has another explicit release contract in docs

If the repo already has a deterministic checked-in release process, refine it instead of replacing it.

Do not infer the version scheme from the package manager alone. Cargo can use semver or CalVer. Python and JavaScript repos can do the same.

## Versioning Scheme

Treat the versioning scheme as a separate contract from the package manager.

Common cases:

- Forge/Cargo in this repo: Phoenix-date CalVer `YYYYMMDD.0.N`
- semver repos: `MAJOR.MINOR.PATCH`
- date-based repos: `YYYY.MM.DD`, `YYYYMMDD`, or similar
- repo-specific variants: documented prerelease/build suffixes or custom tag formats

The checked-in script should:

- validate the repo’s actual version format
- resolve the next version only when the repo contract makes that deterministic
- require `--version` when the next version cannot be safely inferred
- keep version files, tags, and release names aligned with the repo contract

## Repo Adaptation

Keep the workflow shape stable, but swap the toolchain-specific steps for each repo:

- Cargo: bump crate manifests, update `Cargo.lock`, run `cargo check`
- `uv` / `pyproject.toml`: bump `pyproject.toml`, refresh the lockfile if the repo tracks one, run the repo’s Python verification command
- `pnpm`: bump `package.json` or workspace package manifests, refresh `pnpm-lock.yaml`, run the repo’s package-manager verification command
- manual GitHub release flow: keep versioning and validation in the script, but let the final publish step call `gh release create` explicitly if that is the repo contract
- non-`gh` publish flow: document the repo’s actual final publish command and keep it explicit in the script

The script should verify the actual files and checks used by that repo rather than assuming Cargo.

## Agent Instructions

After creating the local release process, leave repo-local instructions that future agents can follow without rediscovering the workflow.

Those instructions should say:

- when to use the `cut-release` skill
- which entrypoint to prefer: `just cut-release`, `./scripts/cut-release.sh`, or another repo wrapper
- whether dry-run is required before the real release
- which versioning scheme the repo uses and whether the script can infer the next version
- which package manager and version files the script owns
- whether the final publish step is `gh release create` or another explicit repo-local release action

Working rules:

- Prefer `just cut-release` as the default entrypoint when a `justfile` exists; otherwise use the checked-in repo wrapper that fits the repo.
- Use `--dry-run` before mutating when you need to verify the next version or the enforced sequence.
- Use `--version <v>` whenever the repo contract does not make next-version inference deterministic.
- If the repo does support safe inference, encode the repo’s real scheme in the script rather than assuming CalVer or semver.
- Do not reconstruct the flow with separate bump, check, push, and `gh release create` commands unless you are explicitly correcting a previously published release.
- Keep the normal release diff limited to the repo’s version files, lockfiles, and the release script/skill/docs when you are establishing the pattern.
- If you change the release flow itself in Forge, update [docs/release.md](../../../docs/release.md), [AGENTS.md](../../../AGENTS.md), and this skill together.

Safety:

- `just cut-release` commits, pushes `main`, and creates a public GitHub release.
- Corrections to an already-published tag or release stay explicit. Inspect the live tag, fix the repo state first, then recreate the release deliberately.
