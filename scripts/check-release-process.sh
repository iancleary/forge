#!/bin/sh

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORKFLOW="$ROOT/.github/workflows/release-artifacts.yml"
TARGETS="$ROOT/config/release-targets.toml"
LOG="$(mktemp)"
STUBS="$(mktemp -d)"

cleanup() {
  rm -f "$LOG"
  rm -rf "$STUBS"
}
trap cleanup EXIT HUP INT TERM

fail() {
  echo "release process check failed: $*" >&2
  exit 1
}

line_number() {
  awk -v pattern="$1" '$0 ~ pattern { print NR; exit }' "$WORKFLOW"
}

assert_before() {
  first="$(line_number "$1")"
  second="$(line_number "$2")"
  [ -n "$first" ] || fail "missing workflow marker: $1"
  [ -n "$second" ] || fail "missing workflow marker: $2"
  [ "$first" -lt "$second" ] || fail "$1 must precede $2"
}

grep -Eq '^  workflow_dispatch:$' "$WORKFLOW" || fail "workflow must be manually dispatched"
if grep -Eq 'types: \[published\]|gh release upload' "$WORKFLOW"; then
  fail "release publication must not trigger or precede artifact construction"
fi
grep -Eq 'x86_64-unknown-linux-gnu' "$WORKFLOW" || fail "missing Linux release target"
grep -Eq 'aarch64-apple-darwin' "$WORKFLOW" || fail "missing macOS release target"
grep -Eq 'windows-2025' "$WORKFLOW" || fail "missing native Windows runner"
grep -Eq 'x86_64-pc-windows-msvc' "$WORKFLOW" || fail "missing Windows x64 release target"
grep -Eq 'dist/\*\.zip' "$WORKFLOW" || fail "release publication must include Windows ZIP assets"
for target in aarch64-apple-darwin x86_64-unknown-linux-gnu x86_64-pc-windows-msvc; do
  grep -Fq "triple = \"$target\"" "$TARGETS" || fail "release target contract is missing $target"
done
grep -Fq 'triple = "x86_64-pc-windows-msvc"' "$TARGETS" || fail "target contract must include native Windows x64"
if grep -Eq 'x86_64-apple-darwin|aarch64-pc-windows-msvc|i686-pc-windows' "$TARGETS"; then
  fail "target contract includes an excluded platform"
fi
grep -Eq 'forge-release-sha256sums.txt' "$ROOT/scripts/build-forge-release-manifest.sh" || fail "manifest builder must emit checksums"
grep -Eq 'forge-release-manifest.json' "$ROOT/scripts/build-forge-release-manifest.sh" || fail "manifest builder must emit release metadata"
grep -Eq -- '--source-digest "\$RELEASE_SHA"' "$WORKFLOW" || fail "workflow attestations must be pinned to the release commit"
grep -Eq 'forge-release-sha256sums.txt' "$ROOT/scripts/install-forge-release.sh" || fail "installer must download the published checksum manifest"
grep -Eq 'checksum mismatch' "$ROOT/scripts/install-forge-release.sh" || fail "installer must reject checksum mismatches"
assert_before 'Validate release inputs' 'Run contributor checks'
assert_before 'Run contributor checks' 'Build Release Artifact'
assert_before 'Build Release Artifact' 'Upload Build Outputs'
assert_before 'Upload Build Outputs' 'Verify staged attestations'
assert_before 'Verify staged attestations' 'Publish GitHub Release'
assert_before 'Publish GitHub Release' 'gh release create'

cat >"$STUBS/git" <<'EOF'
#!/bin/sh
echo "git $*" >> "$RELEASE_PROCESS_TEST_LOG"
case "$*" in
  *"status --short") exit 0 ;;
  *"rev-parse --abbrev-ref HEAD") echo main ;;
  *"rev-parse --verify origin/main") exit 0 ;;
  *"merge-base --is-ancestor origin/main HEAD") exit 0 ;;
  *"rev-parse -q --verify refs/tags/"*) exit 1 ;;
  *"diff --name-only --relative")
    printf '%s\n' Cargo.lock
    find "$RELEASE_PROCESS_TEST_ROOT/crates" -mindepth 2 -maxdepth 2 -name Cargo.toml -print |
      sed "s#^$RELEASE_PROCESS_TEST_ROOT/##" |
      sort
    ;;
  *"rev-parse HEAD") printf '%040d\n' 1 ;;
esac
EOF

cat >"$STUBS/just" <<'EOF'
#!/bin/sh
echo "just $*" >> "$RELEASE_PROCESS_TEST_LOG"
EOF

cat >"$STUBS/cargo" <<'EOF'
#!/bin/sh
echo "cargo $*" >> "$RELEASE_PROCESS_TEST_LOG"
EOF

cat >"$STUBS/gh" <<'EOF'
#!/bin/sh
echo "gh $*" >> "$RELEASE_PROCESS_TEST_LOG"
case "$*" in
  "release view "*) exit 1 ;;
  "run list "*) echo 12345 ;;
esac
EOF

chmod +x "$STUBS/git" "$STUBS/just" "$STUBS/cargo" "$STUBS/gh"

PATH="$STUBS:$PATH" \
  RELEASE_PROCESS_TEST_LOG="$LOG" \
  RELEASE_PROCESS_TEST_ROOT="$ROOT" \
  "$ROOT/scripts/cut-release.sh" --version 20260801.0.0 >/dev/null

ci_line="$(awk '/^just .* ci$/ { print NR; exit }' "$LOG")"
push_line="$(awk '/^git .* push origin main$/ { print NR; exit }' "$LOG")"
dispatch_line="$(awk '/^gh workflow run release-artifacts.yml / { print NR; exit }' "$LOG")"
watch_line="$(awk '/^gh run watch 12345 --compact --exit-status$/ { print NR; exit }' "$LOG")"

[ -n "$ci_line" ] || fail "runner did not execute the full contributor checks"
[ -n "$push_line" ] || fail "runner did not push the release commit"
[ -n "$dispatch_line" ] || fail "runner did not dispatch the release workflow"
[ -n "$watch_line" ] || fail "runner did not wait for the release workflow"
[ "$ci_line" -lt "$push_line" ] || fail "checks must precede the release commit push"
[ "$push_line" -lt "$dispatch_line" ] || fail "push must precede workflow dispatch"
[ "$dispatch_line" -lt "$watch_line" ] || fail "dispatch must precede workflow wait"
if grep -Eq '^gh release create ' "$LOG"; then
  fail "the local runner must not create the release before artifacts are ready"
fi

: >"$LOG"
current_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$ROOT/crates/forge/Cargo.toml" | head -n 1)"
PATH="$STUBS:$PATH" \
  RELEASE_PROCESS_TEST_LOG="$LOG" \
  RELEASE_PROCESS_TEST_ROOT="$ROOT" \
  "$ROOT/scripts/cut-release.sh" --version "$current_version" >/dev/null

if grep -Eq '^just .* bump-version |^git .* commit ' "$LOG"; then
  fail "resuming an unpublished version must not create another version commit"
fi
grep -Eq '^gh workflow run release-artifacts.yml .*target_sha=0000000000000000000000000000000000000001' "$LOG" ||
  fail "resuming an unpublished version must dispatch the current commit"

echo "release process ordering is valid"
