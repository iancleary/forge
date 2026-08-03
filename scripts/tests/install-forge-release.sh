#!/bin/sh

set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="20260802.0.0"
SYSTEM_PATH="$PATH"
TEST_ROOT="$(mktemp -d)"
FAKE_BIN="$TEST_ROOT/fake-bin"
mkdir -p "$FAKE_BIN"
trap 'rm -rf "$TEST_ROOT"' EXIT HUP INT TERM

fail() {
  echo "installer fixture test failed: $*" >&2
  exit 1
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

cat > "$FAKE_BIN/curl" <<'EOF'
#!/bin/sh
set -eu
output=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output|-o) output="$2"; shift 2 ;;
    *) url="$1"; shift ;;
  esac
done
name="${url##*/}"
[ "$name" != "latest" ] || name="releases-latest.json"
[ -n "${FORGE_TEST_FIXTURE_ROOT:-}" ] || exit 1
cp "$FORGE_TEST_FIXTURE_ROOT/$name" "$output"
EOF

cat > "$FAKE_BIN/gh" <<'EOF'
#!/bin/sh
set -eu
printf '%s\n' "$*" >> "${FORGE_TEST_GH_LOG:?}"
[ "${FORGE_TEST_GH_MODE:-fail}" = success ]
EOF

cat > "$FAKE_BIN/cargo" <<'EOF'
#!/bin/sh
printf 'cargo %s\n' "$*" >> "${FORGE_TEST_TOOL_LOG:?}"
exit 1
EOF

cat > "$FAKE_BIN/git" <<'EOF'
#!/bin/sh
printf 'git %s\n' "$*" >> "${FORGE_TEST_TOOL_LOG:?}"
exit 1
EOF

cat > "$FAKE_BIN/uname" <<'EOF'
#!/bin/sh
case "${FORGE_TEST_UNAME_MODE:-native}:$1" in
  unsupported:-s) printf '%s\n' SunOS ;;
  unsupported:-m) printf '%s\n' sparc ;;
  *) exec /usr/bin/uname "$@" ;;
esac
EOF
chmod 755 "$FAKE_BIN"/*

make_fixture() {
  label="$1"
  fixture="$TEST_ROOT/$label"
  files="$fixture/files"
  mkdir -p "$files"
  for binary in forge codex-threads linear mermaid slack-agent slack-query; do
    printf '#!/bin/sh\nexit 0\n' > "$files/$binary"
    chmod 755 "$files/$binary"
  done
  printf '%s\n' "fixture release $label" > "$fixture/release-note"
  printf '%s\n' "$fixture"
}

write_manifest() {
  fixture="$1"
  archive="$2"
  manifest_mode="${3:-valid}"
  archive_sha="$(sha256 "$archive")"
  case "$manifest_mode" in
    valid) printf '%s  %s\n' "$archive_sha" "$ASSET_NAME" > "$fixture/forge-release-sha256sums.txt" ;;
    mismatch) printf '%064d  %s\n' 0 "$ASSET_NAME" > "$fixture/forge-release-sha256sums.txt" ;;
    missing) printf '%064d  other.zip\n' 0 > "$fixture/forge-release-sha256sums.txt" ;;
    duplicate)
      {
        printf '%s  %s\n' "$archive_sha" "$ASSET_NAME"
        printf '%s  %s\n' "$archive_sha" "$ASSET_NAME"
      } > "$fixture/forge-release-sha256sums.txt"
      ;;
    malformed) printf 'not a checksum record\n' > "$fixture/forge-release-sha256sums.txt" ;;
    *) fail "unknown manifest mode: $manifest_mode" ;;
  esac
}

make_archive() {
  fixture="$1"
  mode="${2:-valid}"
  files="$fixture/files"
  raw="$fixture/archive.tar"
  entries="forge codex-threads linear mermaid slack-agent slack-query"
  case "$mode" in
    missing) entries="forge codex-threads linear mermaid slack-agent" ;;
    unexpected)
      printf 'extra\n' > "$files/extra"
      entries="$entries extra"
      ;;
    duplicate)
      tar -C "$files" -cf "$raw" $entries
      tar -C "$files" -rf "$raw" forge
      gzip -c "$raw" > "$fixture/$ASSET_NAME"
      return
      ;;
    traversal)
      printf 'escape\n' > "$fixture/escape"
      tar -C "$files" -cf "$raw" $entries
      tar -C "$files" -rf "$raw" ../escape
      gzip -c "$raw" > "$fixture/$ASSET_NAME"
      return
      ;;
    symlink)
      ln -s forge "$files/link"
      entries="$entries link"
      ;;
    hardlink)
      ln "$files/forge" "$files/hardlink"
      entries="$entries hardlink"
      ;;
    valid) ;;
    *) fail "unknown archive mode: $mode" ;;
  esac
  tar -C "$files" -czf "$fixture/$ASSET_NAME" $entries
}

prepare_fixture() {
  label="$1"
  archive_mode="${2:-valid}"
  manifest_mode="${3:-valid}"
  fixture="$(make_fixture "$label")"
  make_archive "$fixture" "$archive_mode"
  write_manifest "$fixture" "$fixture/$ASSET_NAME" "$manifest_mode"
  printf '%s\n' "$fixture"
}

run_installer() {
  fixture="$1"
  shift
  home="$TEST_ROOT/home-$(basename "$fixture")-${RANDOM:-0}"
  cargo_home="$home/cargo"
  mkdir -p "$home" "$cargo_home"
  log="$TEST_ROOT/tools-$(basename "$fixture")-${RANDOM:-0}.log"
  gh_log="$TEST_ROOT/gh-$(basename "$fixture")-${RANDOM:-0}.log"
  : > "$log"
  : > "$gh_log"
  RUN_HOME="$home"
  RUN_LOG="$log"
  RUN_GH_LOG="$gh_log"
  if ! (
    export PATH="$FAKE_BIN:$SYSTEM_PATH"
    export HOME="$home"
    export CARGO_HOME="$cargo_home"
    export FORGE_INSTALLER_PINNED=1
    export FORGE_TAG="$VERSION"
    export FORGE_TEST_FIXTURE_ROOT="$fixture"
    export FORGE_TEST_TOOL_LOG="$log"
    export FORGE_TEST_GH_LOG="$gh_log"
    export FORGE_TEST_GH_MODE="${FORGE_TEST_GH_MODE:-fail}"
    export FORGE_TEST_UNAME_MODE="${FORGE_TEST_UNAME_MODE:-native}"
    "$ROOT/scripts/install-forge-release.sh" --tag "$VERSION" --skip-codex "$@"
  ) > "$TEST_ROOT/stdout.log" 2> "$TEST_ROOT/stderr.log"; then
    cat "$TEST_ROOT/stderr.log" >&2
    return 1
  fi
}

run_installer_in_existing_home() {
  fixture="$1"
  shift
  (
    export PATH="$FAKE_BIN:$SYSTEM_PATH"
    export HOME="$RUN_HOME"
    export CARGO_HOME="$RUN_HOME/cargo"
    export FORGE_INSTALLER_PINNED=1
    export FORGE_TAG="$VERSION"
    export FORGE_TEST_FIXTURE_ROOT="$fixture"
    export FORGE_TEST_TOOL_LOG="$RUN_LOG"
    export FORGE_TEST_GH_LOG="$RUN_GH_LOG"
    export FORGE_TEST_GH_MODE="${FORGE_TEST_GH_MODE:-fail}"
    export FORGE_TEST_UNAME_MODE="${FORGE_TEST_UNAME_MODE:-native}"
    "$ROOT/scripts/install-forge-release.sh" --tag "$VERSION" --skip-codex "$@"
  ) > "$TEST_ROOT/stdout.log" 2> "$TEST_ROOT/stderr.log"
}

expect_failure() {
  fixture="$1"
  shift
  if run_installer "$fixture" "$@"; then
    fail "expected installer failure for $(basename "$fixture")"
  fi
}

ASSET_NAME="forge-${VERSION}-$(uname -s | tr '[:upper:]' '[:lower:]')"
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64|Darwin/aarch64) TARGET="aarch64-apple-darwin" ;;
  Linux/x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
  *) fail "native runner is not one of the supported POSIX fixture platforms" ;;
esac
ASSET_NAME="forge-${VERSION}-${TARGET}.tar.gz"

valid="$(prepare_fixture valid)"
run_installer "$valid"
[ -x "$RUN_HOME/cargo/bin/forge" ] || fail "valid release did not install forge"
[ ! -s "$RUN_LOG" ] || fail "default release install invoked a toolchain command"

for case_name in mismatch missing duplicate malformed; do
  fixture="$(prepare_fixture "checksum-$case_name" valid "$case_name")"
  expect_failure "$fixture"
done

for case_name in missing unexpected duplicate traversal symlink hardlink; do
  fixture="$(prepare_fixture "archive-$case_name" "$case_name")"
  expect_failure "$fixture"
done

fixture="$(prepare_fixture destination-symlink)"
home="$TEST_ROOT/destination-symlink-home"
mkdir -p "$home/cargo"
ln -s "$TEST_ROOT/redirect" "$home/cargo/bin"
if (
  export PATH="$FAKE_BIN:$SYSTEM_PATH" HOME="$home" CARGO_HOME="$home/cargo" FORGE_INSTALLER_PINNED=1 FORGE_TAG="$VERSION" FORGE_TEST_FIXTURE_ROOT="$fixture"
  "$ROOT/scripts/install-forge-release.sh" --tag "$VERSION" --skip-codex
); then
  fail "destination-directory symlink was accepted"
fi

fixture="$(prepare_fixture binary-symlink)"
run_installer "$fixture"
rm -f "$RUN_HOME/cargo/bin/codex-threads"
ln -s "$TEST_ROOT/elsewhere" "$RUN_HOME/cargo/bin/codex-threads"
if run_installer_in_existing_home "$fixture"; then
  fail "binary destination symlink was accepted"
fi

fixture="$(prepare_fixture rollback)"
run_installer "$fixture"
for binary in forge codex-threads linear mermaid slack-agent slack-query; do
  printf 'old-%s\n' "$binary" > "$RUN_HOME/cargo/bin/$binary"
done
if FORGE_TEST_FAIL_REPLACEMENT_AFTER=1 run_installer_in_existing_home "$fixture"; then
  fail "replacement failure hook unexpectedly succeeded"
fi
for binary in forge codex-threads linear mermaid slack-agent slack-query; do
  if ! grep -Fqx "old-$binary" "$RUN_HOME/cargo/bin/$binary"; then
    echo "rollback content for $binary:" >&2
    sed -n '1,3p' "$RUN_HOME/cargo/bin/$binary" >&2 || true
    fail "rollback left mixed-version binary set"
  fi
done
echo "ok: rollback"
unset FORGE_TEST_FAIL_REPLACEMENT_AFTER

fixture="$(prepare_fixture attestation-success)"
FORGE_TEST_GH_MODE=success run_installer "$fixture" --verify-attestation
grep -Fq "attestation verify" "$RUN_GH_LOG" || fail "explicit attestation was not requested"
echo "ok: attestation success"

fixture="$(prepare_fixture attestation-failure)"
FORGE_TEST_GH_MODE=fail
expect_failure "$fixture" --verify-attestation
echo "ok: attestation failure"

fixture="$(prepare_fixture explicit-source)"
expect_failure "$fixture" --build-from-source
grep -Fqx "git clone --depth 1 --branch $VERSION $ROOT" "$RUN_LOG" 2>/dev/null ||
  grep -Fq 'git clone' "$RUN_LOG" || fail "explicit source mode did not invoke git"
echo "ok: explicit source selection"

fixture="$(prepare_fixture unsupported)"
if FORGE_TEST_UNAME_MODE=unsupported run_installer "$fixture"; then
  fail "unsupported platform was accepted"
fi
echo "ok: unsupported platform"

echo "ok: POSIX release installer adversarial fixtures passed"
