# Release Process Templates

These templates are starting points for repos that should use Forge's
`release.toml` contract.

Copy the closest `*.release.toml` file to the target repo as `release.toml`,
then adjust the release name, provider, checks, tag prefix, notes policy, and
version files.

Use `scripts/calver_day_serial.py` when a repo wants semver-compatible CalVer
versions like `YYYYMMDD.0.0`, `YYYYMMDD.0.1`, and the next date's
`YYYYMMDD.0.0`. The final numeric component starts at `0` each date.

Keep the Python script stdlib-only when adapting it. If the behavior grows into
a shared primitive, move that behavior into Forge itself rather than growing a
large repo-local script.
