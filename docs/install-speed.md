# Install Speed And Integrity

This document defines the recommended direction for making Forge installs and updates faster without weakening the release integrity model.

Current status:

- single-workspace source builds are implemented for install and update fallback
- verified release artifacts are implemented for the curated supported platforms
- release metadata now includes a JSON manifest plus published SHA-256 sums
- stronger signing and provenance are still future hardening work

## Problem

Today, Forge release installs and release updates rebuild each managed binary from source with Cargo.

That has a few good properties:

- uses tagged source
- uses the checked-in `Cargo.lock`
- avoids trusting prebuilt artifacts

But it is slower than it needs to be because:

- each managed binary is installed separately
- source fetch and build setup repeat across packages
- first install requires a local Rust toolchain even when the user only wants the released binaries

## Current Behavior

The current release path is intentionally simple:

- bootstrap script resolves a release tag
- installer runs `cargo install --git ... --tag ... --locked --force <bin>` for each managed binary
- `forge self update` follows the same source-build model

This keeps dependency resolution pinned to `Cargo.lock`, but it does not yet provide:

- platform release artifacts
- artifact checksums
- artifact signatures or provenance
- a single-build install path for the whole managed tool set

## Goals

- reduce first-install and update time materially
- keep deterministic dependency resolution
- preserve a secure fallback path when prebuilt artifacts are unavailable
- make release integrity explicit instead of implicit
- keep the install surface narrow and understandable

## Non-Goals

- publish to crates.io
- add native package manager formulas as the primary distribution path
- require reproducible builds across all user environments
- compare locally built binaries byte-for-byte against release binaries

The last item is important: binary equality is not a practical primary integrity check unless the project commits to reproducible builds across toolchains and platforms.

## Security Model

The secure inputs to verify are:

- release tag
- source commit SHA
- `Cargo.lock`
- Rust toolchain version used for release artifacts
- per-artifact checksums

The secure outputs to trust are:

- a locally built binary from a pinned tagged source tree using `--locked`
- or a downloaded release artifact whose checksum matches published release metadata

The following are not sufficient by themselves:

- hashing only `Cargo.lock`
- trusting a GitHub release tag without tying it to source metadata
- comparing a local binary to a release binary without reproducible build guarantees

## Recommended Direction

Implement all four of these, in order.

### 1. Single Workspace Source Build

Keep the current trust model, but remove repeated package installs.

Target behavior:

- fetch the tagged source tree once
- build the managed binaries in one workspace release build with `--locked`
- copy the resulting executables into `~/.cargo/bin`

Why:

- fastest low-risk improvement
- keeps the current source-based security story
- reduces repeated Cargo work without introducing artifact trust yet

This should become the source-build fallback path as later steps land.

### 2. Platform Release Artifacts

Publish prebuilt release bundles for supported targets.

Recommended shape:

- one archive per target triple
- each archive contains all managed Forge binaries for that platform
- archive naming includes version and target triple

Examples:

- `forge-20260415.0.1-x86_64-apple-darwin.tar.gz`
- `forge-20260415.0.1-aarch64-apple-darwin.tar.gz`
- `forge-20260415.0.1-x86_64-unknown-linux-gnu.tar.gz`

Why:

- biggest user-visible speedup
- removes Rust toolchain setup from the common install path
- keeps install/update work closer to simple download, verify, unpack, reconcile

### 3. Release Manifest And Checksums

Every release should publish explicit metadata that lets the installer verify what it is about to install.

Recommended manifest contents:

- release version
- source commit SHA
- `Cargo.lock` SHA-256
- Rust toolchain version
- list of supported artifacts
- SHA-256 for each artifact

A simple JSON manifest attached to the GitHub release is sufficient for the richer metadata path in v1. For bootstrap consumers that avoid JSON parsing, publish `forge-release-sha256sums.txt` alongside it.

This manifest is the trust bridge between the release tag, source inputs, and downloadable binaries.

### 4. Verified Source-Build Fallback

Keep a source-build path for cases where:

- no verified artifact exists for the platform
- a user prefers building from source
- release recovery is needed

Target behavior:

- installer prefers a verified platform artifact when available
- installer falls back to a pinned tagged source build when needed
- source builds continue to use `--locked`

This preserves a trustworthy path even if release artifact publishing is temporarily broken or incomplete.

## Bootstrap Hardening

The bootstrap story should also be tightened.

Current bootstrap examples fetch the installer script from `main` and then optionally pass `--tag <version>`.

That is not fully pinned, because:

- the installer logic comes from the moving `main` branch
- only the source build target is pinned afterward

Recommended direction:

- fetch the installer script from the same release tag being installed
- or publish the installer itself as a release asset

For a deterministic bootstrap, both the installer logic and the installed payload should come from the same release. The simplest practical way to do that is to re-exec the installer from the resolved release tag before performing the install work.

## Installer Behavior

Target installer behavior for release installs:

1. Resolve target version.
2. Resolve platform.
3. Fetch release manifest.
4. If a matching artifact exists:
5. Download artifact and verify SHA-256.
6. Unpack binaries into `~/.cargo/bin`.
7. Run post-install Forge reconciliation for managed skills and Codex assets.
8. If no matching artifact exists, fall back to source build with `--locked`.

The installer should remain explicit and fail closed on checksum mismatch.

## Update Behavior

`forge self update` should follow the same model as bootstrap:

- resolve latest target version
- fetch release manifest
- prefer verified platform artifact
- fall back to pinned source build when necessary
- after binary install, reconcile managed skills and Codex assets using the installed release payload

This keeps first install and steady-state update behavior aligned.

## Artifact Verification

For v1, checksum verification is the minimum acceptable artifact validation layer.

Recommended verification rules:

- manifest is fetched from the release being installed
- artifact checksum must match manifest checksum exactly
- installer aborts on mismatch
- installer does not silently continue with an unverified artifact

Stronger later options:

- signed checksums
- Sigstore signing
- GitHub artifact provenance attestation

Those are valuable, but they should be layered on top of a clean checksum-and-manifest model rather than replacing it.

## Why Not Binary Comparison

Do not design the install flow around comparing a locally built binary to a downloaded release binary.

Reasons:

- Rust release binaries are not guaranteed to be reproducible across environments
- linker behavior, toolchain patch level, build path inputs, and platform details can change the final bytes
- a failed binary comparison does not clearly distinguish tampering from ordinary build variance

If reproducible builds become a project goal later, binary comparison can become an optional validation layer. It should not be the base release integrity model.

## Why `Cargo.lock` Hash Alone Is Not Enough

`Cargo.lock` is necessary, but not sufficient.

It constrains dependency versions, but it does not identify:

- the exact Forge source commit
- the installer logic
- the Rust toolchain used to build release artifacts
- whether the downloaded artifact actually came from those inputs

Use the `Cargo.lock` hash as one field in release metadata, not as the sole integrity check.

## Rollout Plan

### Phase 1

- change release install/update from repeated `cargo install --git` calls to a single workspace source build
- keep current release distribution model otherwise unchanged

### Phase 2

- add CI release builds for supported target triples
- attach archives and manifest to GitHub releases

### Phase 3

- update bootstrap installer to prefer verified artifacts
- update `forge self update` to prefer verified artifacts
- keep source-build fallback

### Phase 4

- add stronger signing or provenance if desired
- expand target coverage only after the core flow is stable

## Open Design Questions

- which initial target triples should Forge support
- whether the release manifest should be a standalone asset or embedded in release metadata
- whether to publish a single platform archive containing all binaries or per-binary archives
- whether the source-build fallback should build from a downloaded release tarball source tree or a pinned git tag checkout

## Recommendation

The recommended implementation order is:

1. single workspace source build
2. platform release artifacts
3. release manifest with checksums and source metadata
4. verified source-build fallback

All four should land. The ordering matters because it preserves the current trust model first, then adds speed, then makes artifact trust explicit, while retaining a secure escape hatch.
