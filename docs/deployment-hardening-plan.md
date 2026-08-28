# Forge Binary Deployment Hardening Plan

## Purpose

Implement a checksum-first binary deployment model for Forge without requiring development toolchains. Add native Windows x64 to the existing macOS and Linux release targets.

This document is an implementation handoff. The product and security decisions below are fixed. An implementation agent must not redesign the trust model or expand the platform matrix.

## Current branch state

Branch: `refactor/ai-workbench-scope`

The branch already contains partial deployment work:

- the POSIX installer downloads `forge-release-sha256sums.txt`
- the installer verifies the selected archive checksum
- implicit source-build fallback was removed
- `--build-from-source` remains explicit
- local GitHub attestation verification is no longer required by default
- installer and release documentation was partially updated

Treat these edits as work in progress. Review the full branch diff before implementation. Preserve the broader scope contraction in [scope.md](scope.md).

## Fixed decisions

### Product boundary

- Forge bootstrap deploys published Forge binaries and managed AI assets.
- Bootstrap does not install Rust, Cargo, Node, Python, uv, package managers, or other development toolchains.
- Forge development uses the repository toolchain separately.
- Nix and Home Manager remain the preferred declarative deployment layer on managed Unix systems.

### Supported targets

Support only these release targets in this change:

- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

Do not add 32-bit Windows, Windows ARM64, or other macOS and Linux architectures. WSL uses the Linux artifact. Native Windows uses the Windows artifact.

### Artifact formats

- macOS and Linux use `forge-<version>-<target>.tar.gz`.
- Native Windows uses `forge-<version>-x86_64-pc-windows-msvc.zip`.
- Windows entries retain `.exe`.
- Every release publishes `forge-release-sha256sums.txt` with one entry per archive.
- Release metadata and GitHub attestation bundles remain published.

### Verification policy

- SHA-256 verification is mandatory for install and update.
- The manifest and archive must come from the same validated release tag.
- Missing, malformed, duplicate, or mismatched checksum entries are hard failures.
- No failure triggers an implicit source build.
- `--build-from-source` is the only source-build entry point.
- GitHub attestation verification is optional hardening.
- An explicit attestation request fails closed if `gh` is missing or verification fails.
- Do not add Minisign or another signing-key system.

### Native Windows interface

- Add a PowerShell installer for native Windows.
- Default Windows deployment must not require a POSIX shell, `tar`, Rust, Cargo, or Git.
- Prefer built-in PowerShell and .NET archive and SHA-256 facilities.
- Match the POSIX version, checksum, archive, destination, and failure contracts.

## Mandatory hardening requirements

These are the accepted hardening items 1 through 5 and 8.

### 1. Strict archive validation

Before changing the destination:

- require the exact expected entry set
- reject missing, additional, and duplicate entries
- reject absolute paths and traversal components
- reject symbolic links, hard links, directories, and other non-regular entries
- validate every extracted binary

The POSIX archive contains exactly:

```text
forge
codex-threads
linear
mermaid
slack-agent
slack-query
```

The Windows archive contains exactly:

```text
forge.exe
codex-threads.exe
linear.exe
mermaid.exe
slack-agent.exe
slack-query.exe
```

Keep this list synchronized with `config/release-tools.toml` and the installer-list check.

### 2. Staged atomic replacement

- Download and extract into temporary directories.
- Validate the complete candidate set before installing any file.
- Stage replacements on the destination filesystem.
- Replace destinations with atomic renames.
- Do not copy directly over existing binaries.
- On failure, preserve or restore the previous complete set.
- Remove temporary and backup files on success.
- Report recovery paths if rollback fails.

A handled failure must not leave a mixed-version binary set.

### 3. Unique checksum selection

- Match the complete archive filename.
- Require exactly one matching entry.
- Require a 64-character hexadecimal SHA-256 value.
- Normalize hexadecimal case before comparison.
- Reject malformed and duplicate matching lines.
- Compute the checksum before extraction.

### 4. Hardened network requests

POSIX downloads must provide behavior equivalent to:

```sh
curl \
  --fail \
  --show-error \
  --location \
  --proto '=https' \
  --tlsv1.2 \
  --retry 3 \
  --retry-all-errors
```

Use only fixed GitHub hosts, a validated Forge CalVer tag, and known asset names. Do not accept arbitrary download URLs from ordinary installer arguments.

PowerShell downloads must require HTTPS, stop on HTTP errors, and use bounded retries. Do not disable certificate validation.

### 5. Destination safety

- Never require or recommend administrator or root execution.
- Set a predictable POSIX umask.
- Reject destination directories and existing binary destinations that are symbolic links.
- On POSIX, verify destination ownership when a portable owner check is available.
- On Windows, install within a current-user destination by default.
- Do not mutate the machine-wide `PATH` automatically.
- Print explicit PATH guidance when necessary.

Prefer `%LOCALAPPDATA%\Forge\bin` as the Windows destination rather than a Cargo-owned directory. Record the final choice in the command documentation.

### 8. Offline adversarial test harness

Add deterministic tests that do not contact GitHub. Cover POSIX and PowerShell behavior on their native CI runners.

Fixtures must cover:

- valid checksum and archive
- checksum mismatch
- missing, duplicate, and malformed checksum records
- missing, unexpected, and duplicate archive entries
- path traversal, symbolic-link, and hard-link entries
- destination-directory and existing-binary symlinks
- unsupported platforms
- failed replacement with no mixed-version result
- explicit source-build selection
- absence of implicit source-build fallback
- optional attestation success, absence, and failure

Use fake downloads or local fixture responses. Do not mock checksum calculation, archive parsing, extraction, or replacement.

## Delegated implementation tasks

### Task A: Normalize release artifacts

Files:

- `config/release-tools.toml`
- `scripts/build-forge-release-artifact.sh`
- `scripts/build-forge-release-manifest.sh`
- `scripts/verify-forge-release.sh`
- `scripts/check-forge-binaries.sh`

Outcomes:

- packaging supports `.tar.gz` and `.zip`
- Windows binaries retain `.exe`
- manifest and checksum assembly accepts both formats
- expected targets and binaries come from one canonical contract where practical
- no release script silently omits Windows

### Task B: Harden the POSIX installer

Files:

- `scripts/install-forge-release.sh`
- new POSIX fixture test and fixtures
- `justfile`

Required surface:

```text
install-forge-release.sh [--tag <version>] [--skip-codex]
install-forge-release.sh --verify-attestation [--tag <version>]
install-forge-release.sh --build-from-source [--tag <version>]
```

Do not run attestation verification merely because `gh` happens to exist.

### Task C: Add native Windows bootstrap

Files:

- new `scripts/install-forge-release.ps1`
- new PowerShell fixture test
- shared binary-contract checks
- relevant docs

Required surface:

```text
install-forge-release.ps1 [-Tag <version>] [-SkipCodex]
install-forge-release.ps1 -VerifyAttestation [-Tag <version>]
install-forge-release.ps1 -BuildFromSource [-Tag <version>]
```

If source build remains available on Windows, require the existing Rust and MSVC environment. Do not install it.

### Task D: Expand release CI

Files:

- `.github/workflows/release-artifacts.yml`
- `scripts/check-release-process.sh`
- workflow contract tests

Outcomes:

- add `windows-2025` and `x86_64-pc-windows-msvc`
- build on native Windows, not through Linux cross-compilation
- require all three platform builds before publication
- include the Windows ZIP, checksum, metadata, and attestation
- verify every expected target before release creation

Windows ARM runners are in public preview. Do not add them.

### Task E: Align self-update

Files:

- `crates/forge/src/main.rs`, or a focused release-install module
- `crates/forge/tests/cli.rs`
- focused unit tests

Outcomes:

- bootstrap and self-update use the same artifact naming rules
- checksum verification is mandatory
- optional attestation is explicit and fail-closed
- default self-update never builds from source
- source build requires an explicit flag
- Windows ZIP installation does not need a POSIX toolchain
- skills and application assets reconcile only after binary replacement succeeds

Changing self-update flags is a public API change. Keep help, JSON output, error codes, tests, and [forge.md](forge.md) synchronized.

### Task F: Align documentation

Files:

- `README.md`
- `docs/forge.md`
- `docs/install-speed.md`
- `docs/release.md`
- `docs/scope.md`

Document native Windows x64 as supported. Explicitly exclude 32-bit Windows and Windows ARM. Describe WSL as Linux deployment. Keep checksum, optional-attestation, explicit-source-build, and Nix/Home Manager ownership rules consistent.

## Sequence

1. Add failing POSIX adversarial fixtures.
2. Harden the POSIX installer until they pass.
3. Normalize packaging and checksum generation for both formats.
4. Add the Windows x64 build and PowerShell installer.
5. Add Windows fixture tests on native Windows CI.
6. Align `forge self update`.
7. Update release assembly and staged verification.
8. Align durable documentation.
9. Run full verification and structured review.

Do not enable GitHub immutable releases in this work. That repository setting and a draft-upload-publish workflow require a separate reviewed change.

## Acceptance criteria

The work is complete only when:

- all three targets build on native hosted runners
- every release contains every expected archive and exactly one checksum entry per archive
- POSIX bootstrap needs no Rust, Cargo, Git, or `gh`
- Windows bootstrap needs no Rust, Cargo, Git, POSIX shell, or `gh`
- default bootstrap and self-update never compile
- explicit source build remains available
- optional attestation verifies valid provenance and fails closed otherwise
- hostile archives cannot escape staging or the validated destination
- handled failures preserve the previous complete binary set
- asset reconciliation occurs only after binary installation succeeds
- command help, JSON, docs, and tests agree

## Verification

At minimum, run:

```sh
sh -n scripts/install-forge-release.sh
scripts/install-forge-release.sh --help
just install-list-check
just release-process-check
just ci
git diff --check
```

Run the new POSIX harness directly. Run the PowerShell harness on native Windows. Exercise artifact construction for both archive formats.

Run structured review and secret scanning after tests pass. Do not bypass the secret scanner prerequisite. Report the missing prerequisite if review cannot start.

## Delegation constraints

- Read `AGENTS.md`, [scope.md](scope.md), [release.md](release.md), and this plan first.
- Inspect the dirty branch before editing and preserve unrelated scope-contraction work.
- Do not push, publish, enable immutable releases, or change repository settings.
- Do not add toolchain installation, Minisign, or another platform.
- Use primary GitHub and Rust sources when external behavior is uncertain.
- Stop for review if implementation requires another archive format, trust root, destination model, or public command contract.
