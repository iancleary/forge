---
name: cut-release
description: "Execute an existing repo-local release workflow for an ordinary release request by reading local release instructions, validating the intended version and notes path, then running the checked-in release runner instead of reconstructing the flow by hand."
---

# Cut Release

Use this skill when the user wants to cut, publish, or ship the next normal release and the repo already has a deterministic release workflow.

This is a portable Forge-managed skill. It must follow the target repo's local release contract.

## Use This When

- the release workflow already exists in the repo
- the user asks to cut, publish, ship, or create the next release
- a checked-in runner or task recipe owns the release sequence
- you should execute that runner instead of hand-building separate bump, tag, push, and publish commands

## Do Not Use This When

- the user wants to create, redesign, audit, or repair the release workflow itself
- the repo has no deterministic release runner yet
- the release script or docs are broken and need maintenance
- you are correcting an already-published release and need explicit manual repair steps

For workflow maintenance, use `create-release-process`.

## Local Contract First

Before running anything mutating:

- read repo-local instructions such as `AGENTS.md`, `CLAUDE.md`, `README.md`, and `docs/release.md`
- inspect the task runner and release script named by those instructions
- identify the versioning scheme, such as SemVer, CalVer, or repo-specific date tags
- identify whether the repo can infer the next version or requires `--version`
- identify how release notes are supplied, such as `--notes-file`, generated notes, or a template
- inspect git status and confirm the intended branch

The local repo contract overrides generic expectations in this skill.

## Execution Pattern

Prefer the repo's documented entrypoint:

- `just cut-release`
- `just cut-release --dry-run`
- `scripts/cut-release.sh`
- `make release`
- package-manager release scripts

Use dry-run first when the repo supports it and either the user asked for verification or version inference, notes generation, file mutation, or publishing behavior has meaningful risk.

Use read-only version query commands when available instead of parsing manifests by hand.

Do not reconstruct the flow with separate version bump, check, push, tag, and publish commands unless the user is explicitly repairing a release outside the normal path.

## GitHub Auth Boundary

Run every `gh auth ...` command outside the sandbox. The sandbox can hide or misreport keyring-backed GitHub credentials, so a sandboxed auth failure is not authoritative.

- `gh auth status`
- `gh auth login`
- `gh auth refresh`

For release work, check GitHub CLI auth with an outside-sandbox `gh auth status` before treating auth as missing or broken. Run other release commands outside the sandbox only when they need Git metadata writes, network access, public mutation, or keyring-backed credentials in the current environment.

Repo-local release runners are allowed to vary, especially outside Forge. Do not assume another repo's release runner matches Forge's `scripts/cut-release.sh`. Before executing a runner, inspect the entrypoint and directly delegated helper scripts for `gh auth`. If the runner calls `gh auth`, may call it through an uninspected helper, or is too opaque to verify, run the whole runner outside the sandbox to avoid a hidden sandboxed auth failure.

## Forge Repo Contract

When this skill is used inside Forge itself:

- read `docs/release.md` if you need the full contract
- inspect git state and confirm the repo is on the intended branch
- run `gh auth ...` checks outside the sandbox before treating GitHub CLI auth failures as blockers
- prefer `just cut-release` as the entrypoint
- prefer `just cut-release --dry-run` before the real release when validating the next version or sequence
- use `just cut-release --print-current-version` for a read-only current-version query
- use `just cut-release --print-next-version` for a read-only next-version query
- the script owns workspace version bumps in `Cargo.lock` and all `crates/*/Cargo.toml` manifests
- omitted `--version` resolves the next Phoenix-date CalVer from fetched git tags for the current Phoenix day
- the final publish step is `gh release create`

## Output

Report:

- whether you ran dry-run, real release, or both
- the version that was printed or cut
- how release notes were supplied
- whether the repo runner succeeded
- the release URL or failing step if publication did not complete

## Safety

Release runners may commit, push, tag, upload artifacts, publish packages, deploy, or create public releases.

If the repo is not in a safe release state, stop and explain why instead of forcing the flow.
