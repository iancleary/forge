---
name: create-release-process
description: "Create or update a repo-local release process, then route ordinary release work through a checked-in `scripts/cut-release.sh` or `just cut-release` plus repo-local instructions in `AGENTS.md`, adapted to the repo’s package manager and release situation."
---

# Create Release Process

Use this skill to establish or update a repo-local release process. The goal is not just to run a release once. The goal is to leave behind a deterministic local pattern that future agent runs can follow.

Do not use this skill as a substitute for the repo's normal release command when the workflow already exists. In Forge itself, a normal "cut the next release" request should go through the repo-local `cut-release` execution skill, which should in turn run `just cut-release` (often after `just cut-release --dry-run`).

Primary commands and related surfaces:

- `just cut-release`
- `just cut-release --dry-run`
- `just cut-release --print-current-version`
- `just cut-release --print-next-version`
- `./scripts/cut-release.sh --version <repo-version>`
- `.agents/skills/cut-release/SKILL.md`

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
- create `.agents/skills/create-release-process/` if it does not exist
- add a repo-local `scripts/cut-release.sh`
- add or update the repo-local `.agents/skills/create-release-process/SKILL.md`
- if ordinary release requests should route through a repo-local execution skill, add or update `.agents/skills/cut-release/SKILL.md`
- add repo-local instructions in `AGENTS.md` that tell future agents when to use the maintenance skill, the execution skill, and the checked-in runner

The pattern should stay deterministic:

- a checked-in script is the source of truth for the release flow
- the maintenance skill updates the script, docs, and repo-local guidance
- the execution skill routes ordinary release requests to that script instead of hand-built shell sequences
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

## Agent Instructions

After creating the local release process, leave repo-local instructions that future agents can follow without rediscovering the workflow.

Those instructions should say:

- when to use the `create-release-process` skill
- when to use the `cut-release` execution skill
- which entrypoint to prefer: `just cut-release`, `./scripts/cut-release.sh`, or another repo wrapper
- whether dry-run is required before the real release
- which versioning scheme the repo uses and whether the script can infer the next version
- which package manager and version files the script owns
- whether the final publish step is `gh release create` or another explicit repo-local release action

Working rules:

- Prefer `just cut-release` as the default entrypoint when a `justfile` exists; otherwise use the checked-in repo wrapper that fits the repo.
- Use this skill to maintain or repair the release workflow.
- Use the `cut-release` skill to execute an ordinary release through the repo runner.
- Use `--dry-run` before mutating when you need to verify the next version or the enforced sequence.
- Use `--version <v>` whenever the repo contract does not make next-version inference deterministic.
- Do not reconstruct the flow with separate bump, check, push, and `gh release create` commands unless you are explicitly correcting a previously published release.
- If you change the release flow itself in Forge, update [docs/release.md](../../../docs/release.md), [AGENTS.md](../../../AGENTS.md), this skill, and the `cut-release` execution skill together.

Safety:

- `just cut-release` commits, pushes `main`, and creates a public GitHub release.
- Corrections to an already-published tag or release stay explicit. Inspect the live tag, fix the repo state first, then recreate the release deliberately.
