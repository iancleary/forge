---
name: cut-release
description: "Execute the repo's existing release workflow for an ordinary release request by running the checked-in `just cut-release` / `scripts/cut-release.sh` flow, validating the version path first when needed."
---

# Cut Release

Use this skill when the user wants to publish the next normal Forge release.

Do not use this skill to create, redesign, or repair the release workflow itself. For that, use `create-release-process`.

## Use this when

- the release workflow already exists in the repo
- the user asks to cut, publish, or ship the next release
- you should follow the checked-in repo runner instead of reconstructing the flow manually

## Do not use this when

- the user wants to create or change the release process
- the release script or docs are broken and need maintenance
- you are correcting an already-published release and need explicit manual repair steps

## Canonical runner

In Forge, the execution surface is the checked-in runner:

- `just cut-release`
- `just cut-release --dry-run`
- `./scripts/cut-release.sh`

The skill should call that runner. It should not replace it.

## Forge-specific contract

- prefer `just cut-release` as the entrypoint
- prefer `just cut-release --dry-run` before the real release when validating the next version or sequence
- use `just cut-release --print-current-version` for a read-only current-version query
- use `just cut-release --print-next-version` for a read-only next-version query
- the script owns workspace version bumps in `Cargo.lock` and all `crates/*/Cargo.toml` manifests
- omitted `--version` resolves the next Phoenix-date CalVer from fetched git tags for the current Phoenix day
- the final publish step is `gh release create`

## Execution checklist

1. Read [docs/release.md](../../../docs/release.md) if you need the full contract.
2. Inspect git state and confirm the repo is on the intended branch.
3. If the user wants verification first, run:
   - `just cut-release --dry-run`
4. For a normal release request, run:
   - `just cut-release`
5. Report the release outcome clearly, including:
   - version/tag used
   - whether dry-run was run first
   - whether the GitHub release was created successfully

## Working rules

- Use this skill to execute an ordinary release through the repo runner.
- Keep the release flow deterministic by delegating to `just cut-release` / `scripts/cut-release.sh`.
- Do not reconstruct the release from separate bump, check, push, tag, and `gh release create` commands unless the user is explicitly repairing a previously published release.
- If the release workflow itself needs to change, stop and switch to `create-release-process`.

## Output contract

When you use this skill, the final response should state:

- whether you ran dry-run, real release, or both
- the version that was printed or cut
- whether the repo runner succeeded
- the release URL or the failing step if publication did not complete

## Safety

- `just cut-release` is public-facing: it commits, pushes `main`, tags the release, and creates a GitHub release.
- If the repo is not in a safe release state, stop and explain why instead of forcing the flow.
