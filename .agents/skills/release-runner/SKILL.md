---
name: release-runner
description: "Use repo-local release.toml plus `forge release` to check, plan, dry-run, or apply deterministic release workflows without reconstructing release steps by hand."
---

# Release Runner

Use this skill when a repository has `release.toml` and the user asks to check, plan, dry-run, cut, publish, or run a normal release.

The repo-local `release.toml` is the contract. It names the release runner and the read-only checks. The runner may be an ordinary command or a Forge built-in such as `builtin:cargo-release`. Do not infer a separate release flow from memory or from generic Cargo habits.

For ordinary SemVer releases, pass the intended bump through `forge release run`:

```sh
forge release run --dry-run --bump patch --json
forge release run --apply --bump minor --json
```

Use `--version <v>` only when the user or repo contract requires an exact version, prerelease, build metadata, CalVer, date tag, or other repo-specific version. Do not pass both `--bump` and `--version`.

## Required Sequence

1. Read `release.toml`.
2. Run:

```sh
forge release check --json
```

3. Run:

```sh
forge release plan --json
```

4. For validation without publishing, run:

```sh
forge release run --dry-run --json
```

5. Only when the user explicitly asks to publish or apply the release, run:

```sh
forge release run --apply --json
```

## Safety

- Treat `forge release check` and `forge release plan` as read-only.
- Treat `forge release run --dry-run` as a runner-controlled dry-run. It may contact remotes if the repo runner does that.
- Treat `forge release run --apply` as mutating and public. It may commit, push, tag, publish packages, create GitHub releases, upload artifacts, or dispatch workflows.
- Never substitute manual `cargo publish`, `git tag`, `gh release`, version bump, or changelog commands unless the checked-in runner is broken and the user explicitly asks for repair.
- Stop if the JSON result reports `ready: false`, a failed check, or a runner failure.

## Output

Report:

- release name
- current and next version when present
- check summary
- runner command
- whether dry-run or apply was executed
- failure details from the Forge JSON error when a command fails
