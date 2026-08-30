# Forge Release Examples

These examples are copyable starting points for repos that want to use
`forge release` through a checked-in `release.toml`.

Pick the closest file, copy it to the target repo as `release.toml`, then adjust
the package name, provider, checks, tag prefix, notes policy, and version files.

## CalVer With Day Serial

Use `cargo-calver-day-serial.release.toml` with
`scripts/calver_day_serial.py` for semver-compatible CalVer tags such as:

```sh
20260830.0.0
20260830.0.1
20260831.0.0
```

The final component starts at `0` each date. The script fetches tags, reads
existing tags for the current date, chooses the next serial, then calls:

```sh
python scripts/calver_day_serial.py --dry-run
python scripts/calver_day_serial.py --apply
```

Useful environment overrides:

```sh
RELEASE_TIMEZONE=America/Phoenix
RELEASE_TAG_PREFIX=v
```

## SemVer

Use `cargo-semver-github.release.toml` for ordinary Rust crates that release as
`v1.2.3` and can use:

```sh
forge release run --dry-run --bump patch
forge release run --apply --bump patch
```

Use `cargo-semver-exact.release.toml` when the repo should require an explicit
version, such as a prerelease or build-metadata version:

```sh
forge release run --dry-run --version 1.3.0-rc.1
forge release run --apply --version 1.3.0-rc.1
```

Use `cargo-semver-gitea.release.toml` for the same Cargo release flow on a
Gitea or Forgejo repository managed with `tea`.
