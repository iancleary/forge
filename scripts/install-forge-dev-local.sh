#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev-local.sh

Dev-only installer for an already-present local checkout:
  - builds the managed binaries in one workspace release build
  - installs them into ~/.cargo/bin

Usage:
  scripts/install-forge-dev-local.sh [--path PATH] [--no-force]

Options:
  --path      Repo root (default: `git rev-parse --show-toplevel` or current dir)
  --no-force  Do not overwrite existing installed binaries
  -h,--help   Show help
EOF
}

repo_root=""
force=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)
      repo_root="${2:-}"
      shift 2
      ;;
    --no-force)
      force=0
      shift 1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: missing required command: $1" >&2
    exit 1
  fi
}

need_cmd cargo

if [[ -z "$repo_root" ]]; then
  if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    repo_root="$(git rev-parse --show-toplevel)"
  else
    repo_root="$(pwd)"
  fi
fi

if [[ ! -f "$repo_root/Cargo.toml" ]]; then
  echo "error: no Cargo.toml at repo root: $repo_root" >&2
  exit 1
fi

release_installer="$repo_root/scripts/install-forge-release.sh"
if [[ ! -f "$release_installer" ]]; then
  echo "error: missing release installer: $release_installer" >&2
  exit 1
fi

echo "Installing Forge binaries from local checkout: $repo_root"
if [[ "$force" -eq 0 ]]; then
  echo "Existing binaries will not be overwritten."
fi

extract_bins() {
  # Single source of truth: binaries list embedded in scripts/install-forge-release.sh.
  sed -n '/^  # BEGIN FORGE_BINARIES$/,/^  # END FORGE_BINARIES$/p' "$release_installer" \
    | sed -e '1d' -e '$d' \
    | sed -e "/^  cat <<'EOF'$/d" -e '/^EOF$/d' \
    | sed -e 's/[[:space:]]*$//' -e '/^$/d' -e '/^#/d'
}

bins=()
while IFS= read -r bin; do
  [[ -n "$bin" ]] || continue
  bins+=("$bin")
  crate_dir="$repo_root/crates/$bin"
  if [[ ! -f "$crate_dir/Cargo.toml" ]]; then
    echo "error: expected crate directory with Cargo.toml: $crate_dir" >&2
    exit 1
  fi
done < <(extract_bins)

if [[ "${#bins[@]}" -eq 0 ]]; then
  echo "error: no managed binaries found in $release_installer" >&2
  exit 1
fi

cargo_args=(build --manifest-path "$repo_root/Cargo.toml" --release --locked)
for bin in "${bins[@]}"; do
  cargo_args+=(-p "$bin" --bin "$bin")
done

cargo "${cargo_args[@]}"

cargo_bin_root="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$cargo_bin_root"

for bin in "${bins[@]}"; do
  echo "  - $bin"
  src="$repo_root/target/release/$bin"
  dest="$cargo_bin_root/$bin"
  if [[ ! -f "$src" ]]; then
    echo "error: expected built binary: $src" >&2
    exit 1
  fi
  if [[ -f "$dest" && "$force" -eq 0 ]]; then
    echo "error: refusing to overwrite existing binary without force: $dest" >&2
    exit 1
  fi
  cp "$src" "$dest"
  chmod 755 "$dest" 2>/dev/null || true
done
