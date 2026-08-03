# Install Speed And Integrity

This document records Forge's binary deployment and self-update trust models.

## Bootstrap deployment

The release installer is a consumer deployment path. It does not provision a Forge development environment.

The default path:

1. Resolves the latest published release tag, or accepts an explicit tag.
2. Validates the tag as Forge CalVer.
3. Re-executes the installer from that exact tag.
4. Detects the supported platform target.
5. Downloads `forge-release-sha256sums.txt` from that tag's GitHub release.
6. Selects the exact checksum entry for the target archive.
7. Downloads the archive from the same release tag.
8. Verifies SHA-256 before extraction.
9. Rejects unexpected archive entries.
10. Installs Forge binaries, skills, and the selected Codex assets.

The default path requires `curl`, `tar`, and one supported SHA-256 utility. It does not require `gh`, Rust, Cargo, Git, Node, Python, or uv.

The installer stops without installing binaries when:

- the platform has no published artifact
- the checksum manifest is unavailable
- the archive has no exact checksum entry
- the checksum entry is malformed
- the downloaded checksum does not match
- the archive contains an unexpected path

It does not silently build from source. `--build-from-source` is an explicit development or recovery operation and requires Cargo and Git.

## Trust boundary

The binary installer obtains the checksum manifest and archive from the same selected release tag over HTTPS. The checksum detects corruption and mismatched assets. It does not provide an independent trust root if the GitHub release and checksum manifest are both replaced by an attacker.

Release CI still creates GitHub provenance attestations and verifies them before publication. Users that require provenance verification can verify the published attestation separately. Local attestation verification is not a prerequisite for the portable bootstrap path.

## Self-update

`forge self update` retains its existing attestation-aware contract for non-Nix installations. It can use an attested artifact or a tagged source build according to its explicit flags and failure policy.

Nix-managed installations must update the Forge package through Nix. They must not use `forge self update` to replace a store-owned binary.

The bootstrap and self-update contracts are intentionally distinct:

- bootstrap optimizes for deploying published binaries without development toolchains
- self-update preserves the existing non-Nix provenance and recovery policy
- Forge development uses the repository toolchain and normal Cargo checks

## Published release files

Each release contains:

- one archive per supported target triple
- `forge-release-manifest.json`
- `forge-release-sha256sums.txt`
- per-artifact `*.attestation.json` bundles
- a metadata attestation bundle

Release CI completes checks, builds every supported target, assembles and verifies release files, and verifies attestations before it creates the tag and GitHub release.
