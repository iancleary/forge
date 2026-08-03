#!/bin/sh

set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="20260802.0.0"
TEST_ROOT="$(mktemp -d)"
FAKE_BIN="$TEST_ROOT/fake-bin"
mkdir -p "$FAKE_BIN"
trap 'rm -rf "$TEST_ROOT"' EXIT HUP INT TERM

cat > "$FAKE_BIN/cargo" <<'EOF'
#!/bin/sh
set -eu
target=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--target" ]; then
    target="$2"
    shift 2
  else
    shift
  fi
done
[ -n "$target" ]
mkdir -p "target/$target/release"
suffix=""
[ "$target" = "x86_64-pc-windows-msvc" ] && suffix=".exe"
for binary in forge codex-threads linear mermaid slack-agent slack-query; do
  printf 'fake %s\n' "$binary" > "target/$target/release/$binary$suffix"
done
EOF
chmod 755 "$FAKE_BIN/cargo"

PATH="$FAKE_BIN:$PATH" \
  "$ROOT/scripts/build-forge-release-artifact.sh" \
  --version "$VERSION" \
  --target x86_64-pc-windows-msvc \
  --output-dir "$TEST_ROOT/windows"

PATH="$FAKE_BIN:$PATH" \
  "$ROOT/scripts/build-forge-release-artifact.sh" \
  --version "$VERSION" \
  --target x86_64-unknown-linux-gnu \
  --output-dir "$TEST_ROOT/linux"

windows_asset="$TEST_ROOT/windows/forge-$VERSION-x86_64-pc-windows-msvc.zip"
linux_asset="$TEST_ROOT/linux/forge-$VERSION-x86_64-unknown-linux-gnu.tar.gz"
windows_entries="$(unzip -Z1 "$windows_asset" | sort)"
linux_entries="$(tar -tzf "$linux_asset" | sort)"
expected_windows="$(printf '%s\n' codex-threads.exe forge.exe linear.exe mermaid.exe slack-agent.exe slack-query.exe)"
expected_posix="$(printf '%s\n' codex-threads forge linear mermaid slack-agent slack-query)"
[ "$windows_entries" = "$expected_windows" ] || exit 1
[ "$linux_entries" = "$expected_posix" ] || exit 1

echo "ok: release artifact format fixtures passed"
