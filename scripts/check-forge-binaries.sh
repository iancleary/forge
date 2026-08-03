#!/bin/sh

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RELEASE_INSTALLER="$ROOT/scripts/install-forge-release.sh"
RELEASE_WINDOWS_INSTALLER="$ROOT/scripts/install-forge-release.ps1"
RELEASE_TOOLS="$ROOT/config/release-tools.toml"

fail() {
  echo "error: $*" >&2
  exit 1
}

extract_binaries() {
  # Extract lines between the embedded markers in the release installer.
  sed -n '/^  # BEGIN FORGE_BINARIES$/,/^  # END FORGE_BINARIES$/p' "$RELEASE_INSTALLER" \
    | sed -e '1d' -e '$d' \
    | sed -e "/^  cat <<'EOF'$/d" -e '/^EOF$/d' \
    | sed -e 's/[[:space:]]*$//' -e '/^$/d'
}

normalize_list() {
  # stdin -> normalized: strip trailing whitespace, drop blanks and comments
  sed -e 's/[[:space:]]*$//' -e '/^$/d' -e '/^#/d'
}

bins="$(extract_binaries | normalize_list || true)"
[ -n "$bins" ] || fail "failed to extract embedded binaries list from $RELEASE_INSTALLER"

# Ensure the list contains no duplicates.
dups="$(echo "$bins" | sort | uniq -d || true)"
if [ -n "$dups" ]; then
  fail "duplicate binaries in embedded list: $dups"
fi

# Ensure every listed binary maps to a real crate path.
echo "$bins" | while IFS= read -r bin; do
  [ -n "$bin" ] || continue

  crate_dir="$ROOT/crates/$bin"
  [ -f "$crate_dir/Cargo.toml" ] || fail "missing crate Cargo.toml for $bin at $crate_dir/Cargo.toml"
  [ -f "$crate_dir/src/main.rs" ] || fail "expected binary crate for $bin at $crate_dir/src/main.rs"
done

contract_bins="$(sed -n 's/^binary = "\([^"]*\)"$/\1/p' "$RELEASE_TOOLS")"
[ -n "$contract_bins" ] || fail "failed to extract binaries from $RELEASE_TOOLS"
[ "$(printf '%s\n' "$bins" | sort)" = "$(printf '%s\n' "$contract_bins" | sort)" ] ||
  fail "POSIX installer binary list differs from $RELEASE_TOOLS"

[ -f "$RELEASE_WINDOWS_INSTALLER" ] || fail "missing native Windows installer: $RELEASE_WINDOWS_INSTALLER"
windows_bins="$(sed -n '/^\$ForgeBinaryNames = @($/,/^)$/p' "$RELEASE_WINDOWS_INSTALLER" |
  sed -e '1d' -e '$d' -e 's/[",]//g' -e 's/\.exe$//' -e 's/[[:space:]]*//g' |
  sed -e '/^$/d' || true)"
[ -n "$windows_bins" ] || fail "failed to extract native Windows binary list"
[ "$(printf '%s\n' "$bins" | sort)" = "$(printf '%s\n' "$windows_bins" | sort)" ] ||
  fail "native Windows installer binary list differs from $RELEASE_INSTALLER"

# Ensure every binary crate in crates/ is listed.
# This prevents silently forgetting to ship a new CLI.
for main_rs in "$ROOT"/crates/*/src/main.rs; do
  # If the glob doesn't match, some shells pass it through literally.
  case "$main_rs" in
    *\**)
      break
      ;;
  esac

  bin="$(basename "$(dirname "$(dirname "$main_rs")")")"
  echo "$bins" | grep -Fx "$bin" >/dev/null 2>&1 || fail "binary crate '$bin' exists but is not listed in $RELEASE_INSTALLER"
done

echo "ok: forge embedded binaries list is valid"
