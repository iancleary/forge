# Release Workflow

This document defines the current release process and the intended future `forge release` workflow.

## Current Process

Use GitHub CLI directly for now.

Recommended sequence:

```sh
git push origin main
gh release create 2026.410.0 --target main --title 2026.410.0 --generate-notes --latest
```

Shell note:

- do not wrap the `gh release create` command across lines unless you use trailing `\`
- if `--generate-notes` ends up on its own line, zsh will try to execute it as a separate command

Why:

- `gh release create` already has a good command surface
- releases are still infrequent
- the repo does not need a wrapper until the process becomes repetitive enough to justify one

## Version Source Of Truth

Release tags should match the crate version policy:

- format: `YYYY.MMDD.N`
- example: `2026.410.0`

The release tag should match the versions in:

- `crates/forge/Cargo.toml`
- `crates/slack-cli/Cargo.toml`

## Future `forge release cut`

Target command shape:

```sh
forge release cut
forge release cut --version 2026.410.1
forge release cut --dry-run
forge release cut --notes-file notes.md
```

Target behavior:

1. Verify git state
2. Verify versions match
3. Run release checks
4. Push `main`
5. Create the GitHub release

### 1. Verify git state

- ensure branch is `main` unless explicitly overridden
- ensure working tree is clean
- ensure local branch is not behind remote

### 2. Verify versions match

- read crate versions from relevant `Cargo.toml` files
- ensure all release-participating crates match
- ensure the requested release tag matches those versions

### 3. Run release checks

Initial default:

```sh
cargo check
```

Later this can expand to:

- `cargo test`
- artifact builds
- checksums

### 4. Push `main`

Push before creating the release:

```sh
git push origin main
```

### 5. Create the release

Use GitHub CLI under the hood:

```sh
gh release create <version> --target main --title <version> --generate-notes --latest
```

## Suggested Flags

- `--version <v>`
- `--dry-run`
- `--notes-file <path>`
- `--no-check`
- `--target <branch>`
- `--not-latest`

## Out Of Scope For Now

- automatic version bumping
- crates.io publishing
- automatic branch merging
- automatic cross-platform artifact packaging
