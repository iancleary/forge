# Install Speed And Integrity

This document records Forge's binary deployment and self-update trust models.

## Supported release targets

Forge publishes only these targets:

- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

WSL uses the Linux artifact. Windows ARM and 32-bit Windows are excluded.

macOS and Linux archives use `.tar.gz`. Native Windows uses `.zip` and keeps
the `.exe` suffix on every binary.

## Bootstrap deployment

The release installers are consumer deployment paths. They do not provision a
Forge development environment.

The POSIX default path:

1. Resolves the latest published release tag, or accepts an explicit tag.
2. Validates the tag as Forge CalVer.
3. Re-executes the installer from that exact tag.
4. Detects the supported platform target.
5. Downloads `forge-release-sha256sums.txt` from that tag's GitHub release.
6. Requires exactly one valid checksum entry for the selected archive.
7. Downloads the archive from the same release tag.
8. Verifies SHA-256 before extraction.
9. Rejects missing, additional, duplicate, unsafe, symlink, hard-link, and
   non-regular archive entries.
10. Stages and atomically replaces the complete binary set.
11. Reconciles skills and Codex assets only after binary replacement succeeds.

The POSIX default path requires `curl`, `tar`, and one supported SHA-256
utility. It does not require `gh`, Rust, Cargo, Git, Node, Python, or uv.

Native Windows uses `scripts/install-forge-release.ps1`. It uses PowerShell,
.NET ZIP and SHA-256 facilities, HTTPS-only bounded retries, and the current
user destination `%LOCALAPPDATA%\Forge\bin`. It does not require a POSIX
shell, `tar`, Rust, Cargo, Git, or `gh` for the default path. It prints PATH
guidance and does not modify the machine-wide PATH.

The installers stop without installing binaries when the platform is
unsupported, the checksum manifest is unavailable or malformed, the selected
entry is missing or duplicated, the checksum does not match, or the archive
does not contain the exact regular-file binary set. A handled replacement
failure restores the previous complete set. If rollback fails, the installer
reports the recovery directory.

Source builds are explicit only:

```sh
scripts/install-forge-release.sh --build-from-source --tag 20260802.0.0
```

```powershell
./scripts/install-forge-release.ps1 -BuildFromSource -Tag 20260802.0.0
```

These modes require the existing Cargo and Git environment. They never install
toolchains and are never selected as an implicit fallback.

## Trust boundary

The default binary path obtains the checksum manifest and archive from the
same validated release tag over HTTPS. The checksum detects corruption and
mismatched assets. It does not provide an independent trust root if the
GitHub release and checksum manifest are both replaced by an attacker.

GitHub provenance attestation is optional hardening. Request it explicitly:

```sh
scripts/install-forge-release.sh --verify-attestation --tag 20260802.0.0
```

```powershell
./scripts/install-forge-release.ps1 -VerifyAttestation -Tag 20260802.0.0
```

An explicit request fails closed when `gh` is missing or verification fails.
The default path does not inspect whether `gh` happens to be installed.

Do not add Minisign or another signing-key system to this deployment model.

## Self-update

`forge self update` uses the same supported target mapping, release archive
names, checksum manifest, exact archive validation, and atomic binary
replacement as the bootstrap path. Its default path never compiles.

`forge self update --build-from-source` is the only source-build mode.
`forge self update --verify-attestation` is optional hardening and fails closed
when `gh` is missing or verification fails. Attestation failure never selects a
source build automatically.

Nix-managed installations must update the Forge package through Nix. They
must not use `forge self update` to replace a store-owned binary.

## Published release files

Each release contains:

- one archive per supported target triple
- `forge-release-manifest.json`
- `forge-release-sha256sums.txt` with one entry per archive
- per-artifact `*.attestation.json` bundles
- a metadata attestation bundle

Release CI completes checks, builds all three supported targets on native
hosted runners, assembles and verifies release files, and verifies attestations
before it creates the tag and GitHub release.
