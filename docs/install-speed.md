# Install Speed And Integrity

This document records the completed install and update design for Forge at the current project scope.

Status:

- complete for the current GitHub-only trust model
- fast path is implemented for curated macOS and Linux targets
- secure fallback is implemented for every other case
- second-site hosting and non-GitHub-native signing are not required for this project right now

## Final Decision

Forge uses a GitHub-only release trust model.

That means:

- GitHub Actions builds release artifacts
- GitHub Releases publishes the artifacts and release metadata
- GitHub provenance attestations are published for release archives and release metadata
- the fast install and update path requires GitHub CLI with `gh release verify-asset`
- if that verification path is unavailable locally, Forge falls back to a tagged source build with `--locked`

This is the intended tradeoff for the project today:

- fast on machines that have `gh`
- still secure on machines that do not
- no second trust root
- no second hosting surface
- no additional signing system to operate yet

## Why This Is Enough

For a solo project, the current model is a fair trade.

It gives:

- materially faster first install and self-update on supported platforms
- explicit integrity checks for downloaded binaries
- a fail-closed path on checksum or attestation verification failure
- a source-built escape hatch when the fast path cannot be verified locally

It does not try to defend against every possible GitHub-side compromise. That higher bar is real, but it is not a hard requirement for this project right now.

## Fast Path Requirements

The fast verified artifact path requires all of the following:

- supported platform artifact
- GitHub release manifest and checksums for the target release
- local GitHub CLI with `gh release verify-asset`
- successful checksum verification
- successful GitHub attestation verification

If any of those are missing, Forge does not install the artifact.

Instead it uses the fallback path:

- clone the exact release tag
- build the managed binaries once in a single workspace release build
- use `cargo build --release --locked`
- install the built binaries into `~/.cargo/bin`

## Security Story

Forge trusts two outputs:

- a locally built binary from a pinned tagged source tree using `--locked`
- a downloaded release artifact that matches the published manifest/checksum data and passes GitHub attestation verification

The key verified inputs are:

- release tag
- source commit SHA
- `Cargo.lock` SHA-256
- Rust toolchain used for the release build
- per-artifact SHA-256

Release metadata currently includes:

- `forge-release-manifest.json`
- `forge-release-sha256sums.txt`
- per-artifact `*.attestation.json` bundles
- metadata attestation bundle for the release metadata files

## Install Behavior

Bootstrap uses the release installer script.

Current behavior:

1. Resolve the target release tag.
2. Re-exec the installer from that exact release tag.
3. Detect the current platform.
4. Attempt the verified artifact path only if `gh release verify-asset` is available locally.
5. Verify artifact SHA-256.
6. Verify GitHub release attestation.
7. Install the binaries.
8. Reconcile Forge-managed skills and Codex assets.
9. If the verified artifact path is unavailable, fall back to a tagged source build with `--locked`.

Important failure rules:

- checksum mismatch is a hard failure
- attestation verification failure is a hard failure
- Forge does not silently install an unverified artifact

## Update Behavior

`forge self update` follows the same contract as bootstrap.

Current behavior:

1. Resolve the latest release tag.
2. Load the release contracts from that release.
3. Attempt the verified artifact path only if `gh release verify-asset` is available locally.
4. Verify checksum and attestation before installing an artifact.
5. Fall back to tagged source build with `--locked` when the verified artifact path is unavailable.
6. Apply release migrations.
7. Reconcile Forge-managed skills and Codex assets with the installed release.

The fast path and the fallback path are intentionally aligned so first install and steady-state update use the same trust model.

## Release Publishing

The release workflow now publishes:

- one archive per supported target triple
- `forge-release-manifest.json`
- `forge-release-sha256sums.txt`
- GitHub provenance attestations for the archives
- GitHub provenance attestations for the release metadata

Supported artifact targets are intentionally curated:

- `x86_64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

Broader target coverage is optional future work, not part of the current security requirement.

## Why Not Binary Comparison

Forge does not compare a locally built binary to a downloaded release binary as the primary integrity check.

That is the right choice because Rust release binaries are not guaranteed to be reproducible across user environments. Toolchain patch level, linker behavior, build paths, and platform details can all change the final bytes without implying tampering.

Binary comparison only becomes useful if reproducible builds become an explicit project goal.

## Why `Cargo.lock` Hash Alone Is Not Enough

`Cargo.lock` matters, but it is not a complete trust anchor.

By itself it does not identify:

- the exact Forge source commit
- the installer logic
- the release toolchain
- whether the downloaded artifact actually came from those inputs

Forge uses the `Cargo.lock` hash as one field inside release metadata, not as the only integrity signal.

## What Is Intentionally Out Of Scope

These are optional future hardening directions, not current requirements:

- second-site hosting such as a Vercel mirror
- non-GitHub-native signed manifests
- Sigstore or cosign verification inside the installer
- a second independent trust root

Those become more compelling if:

- other users start depending on Forge binaries
- offline or non-`gh` verification becomes important
- GitHub-only distribution stops being an acceptable operational dependency
- stronger supply-chain guarantees become a concrete project requirement

## Recommendation

Treat the current model as complete for the project today:

- GitHub-only verified artifact fast path
- `gh` required for that fast path
- tagged `--locked` source-build fallback
- hard failure on checksum or attestation mismatch

If the project grows beyond that threat model later, the next step is a signed release manifest published on GitHub Releases. It is not required now.
