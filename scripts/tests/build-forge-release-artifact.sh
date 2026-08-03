#!/bin/sh

set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="$(sed -n 's/^version = \"\(.*\)\"$/\1/p' "$ROOT/crates/forge/Cargo.toml" | head -n 1)"
TEST_ROOT="$(mktemp -d)"
FAKE_BIN="$TEST_ROOT/fake-bin"
mkdir -p "$FAKE_BIN"
trap 'rm -rf "$TEST_ROOT"' EXIT HUP INT TERM

cat > "$FAKE_BIN/cargo" <<'EOF'
#!/bin/sh
set -eu
target=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--manifest-path" ]; then
    manifest_path="$2"
    shift 2
  elif [ "$1" = "--target" ]; then
    target="$2"
    shift 2
  else
    shift
  fi
done
[ -n "$target" ]
repo_root="$(dirname "$manifest_path")"
mkdir -p "$repo_root/target/$target/release"
suffix=""
[ "$target" = "x86_64-pc-windows-msvc" ] && suffix=".exe"
for binary in forge codex-threads linear mermaid slack-agent slack-query; do
  printf 'fake %s\n' "$binary" > "$repo_root/target/$target/release/$binary$suffix"
done
EOF
chmod 755 "$FAKE_BIN/cargo"

PATH="$FAKE_BIN:$PATH" \
  "$ROOT/scripts/build-forge-release-artifact.sh" \
  --version "$VERSION" \
  --target x86_64-pc-windows-msvc \
  --output-dir "$TEST_ROOT/windows"

(
  cd "$TEST_ROOT"
  PATH="$FAKE_BIN:$PATH" \
    "$ROOT/scripts/build-forge-release-artifact.sh" \
    --version "$VERSION" \
    --target x86_64-pc-windows-msvc \
    --output-dir relative-output
)

PATH="$FAKE_BIN:$PATH" \
  "$ROOT/scripts/build-forge-release-artifact.sh" \
  --version "$VERSION" \
  --target x86_64-unknown-linux-gnu \
  --output-dir "$TEST_ROOT/linux"

windows_asset="$TEST_ROOT/windows/forge-$VERSION-x86_64-pc-windows-msvc.zip"
relative_windows_asset="$TEST_ROOT/relative-output/forge-$VERSION-x86_64-pc-windows-msvc.zip"
linux_asset="$TEST_ROOT/linux/forge-$VERSION-x86_64-unknown-linux-gnu.tar.gz"
windows_entries="$(unzip -Z1 "$windows_asset" | sort)"
relative_windows_entries="$(unzip -Z1 "$relative_windows_asset" | sort)"
linux_entries="$(tar -tzf "$linux_asset" | sort)"
expected_windows="$(printf '%s\n' codex-threads.exe forge.exe linear.exe mermaid.exe slack-agent.exe slack-query.exe)"
expected_posix="$(printf '%s\n' codex-threads forge linear mermaid slack-agent slack-query)"
[ "$windows_entries" = "$expected_windows" ] || exit 1
[ "$relative_windows_entries" = "$expected_windows" ] || exit 1
[ "$linux_entries" = "$expected_posix" ] || exit 1

echo "ok: release artifact format fixtures passed"
