---
name: cut-release
description: "Use the repo-local `just cut-release` or `scripts/cut-release.sh` workflow when the task is to cut a Forge GitHub release, bump the CalVer, dry-run the release path, or update the deterministic Forge release procedure."
---

# Cut Release

Use this skill for repo-local release work. Prefer a checked-in release path over reconstructing the sequence by hand.

Primary commands:

- `just cut-release`
- `just cut-release --dry-run`
- `./scripts/cut-release.sh --version <YYYYMMDD.0.N>`

In Forge itself, read [docs/release.md](../../../docs/release.md) when you need the full release contract or need to update the documented workflow.

## Pattern

When a repo does not already have the release pattern in place:

- create `scripts/` if it does not exist
- create `.agents/skills/cut-release/` if it does not exist
- add a repo-local `scripts/cut-release.sh`
- add or update the repo-local `.agents/skills/cut-release/SKILL.md`
- reference the skill and script in the repo `AGENTS.md`

The pattern should stay deterministic:

- a checked-in script is the source of truth for the release flow
- the skill routes release tasks to that script instead of hand-built shell sequences
- the script supports `--dry-run`
- normal use is dry-run first, then real release
- corrections to an already-published release stay explicit and manual

## Repo Adaptation

Keep the workflow shape stable, but swap the toolchain-specific steps for each repo:

- Cargo: bump crate manifests, update `Cargo.lock`, run `cargo check`
- `uv` / `pyproject.toml`: bump `pyproject.toml`, refresh the lockfile if the repo tracks one, run the repo’s Python verification command
- `pnpm`: bump `package.json` or workspace package manifests, refresh `pnpm-lock.yaml`, run the repo’s package-manager verification command

The script should verify the actual files and checks used by that repo rather than assuming Cargo.

Working rules:

- Prefer `just cut-release` as the default entrypoint.
- Use `--dry-run` before mutating when you need to verify the next version or the enforced sequence.
- Use `--version <v>` only when the release version is already decided; otherwise let the script resolve the next Phoenix-date CalVer.
- Do not reconstruct the flow with separate bump, check, push, and `gh release create` commands unless you are explicitly correcting a previously published release.
- Keep the normal release diff limited to the repo’s version files, lockfiles, and the release script/skill/docs when you are establishing the pattern.
- If you change the release flow itself in Forge, update [docs/release.md](../../../docs/release.md), [AGENTS.md](../../../AGENTS.md), and this skill together.

Safety:

- `just cut-release` commits, pushes `main`, and creates a public GitHub release.
- Corrections to an already-published tag or release stay explicit. Inspect the live tag, fix the repo state first, then recreate the release deliberately.
