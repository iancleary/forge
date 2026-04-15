#!/bin/sh

set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEFAULT_BRANCH="main"
PHOENIX_TZ="America/Phoenix"
DRY_RUN=0
LATEST=1
VERSION=""
NOTES_FILE=""

usage() {
  cat <<'EOF'
Cut a Forge GitHub release from main.

Usage:
  cut-release.sh [--version <version>] [--notes-file <path>] [--not-latest] [--dry-run]

Examples:
  cut-release.sh
  cut-release.sh --version 20260415.0.1
  cut-release.sh --notes-file notes.md
  cut-release.sh --dry-run
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

run() {
  echo "+ $*"
  if [ "$DRY_RUN" -eq 0 ]; then
    "$@"
  fi
}

resolve_next_version() {
  today="$(TZ="$PHOENIX_TZ" date +%Y%m%d)"
  latest_today_tag="$(
    git -C "$ROOT" tag --list "${today}.0.*" --sort=-version:refname | head -n 1
  )"

  if [ -z "$latest_today_tag" ]; then
    printf '%s.0.0\n' "$today"
    return
  fi

  latest_n="${latest_today_tag##*.}"
  printf '%s.0.%s\n' "$today" "$((latest_n + 1))"
}

current_release_version() {
  sed -n 's/^version = "\(.*\)"$/\1/p' "$ROOT/crates/forge/Cargo.toml" | head -n 1
}

expected_release_diff() {
  printf '%s\n' Cargo.lock
  find "$ROOT/crates" -mindepth 2 -maxdepth 2 -name Cargo.toml -print |
    sed "s#^$ROOT/##" |
    sort
}

validate_release_diff() {
  actual="$(
    git -C "$ROOT" diff --name-only --relative | sort
  )"
  expected="$(expected_release_diff)"

  if [ "$actual" != "$expected" ]; then
    echo "expected release diff:" >&2
    printf '%s\n' "$expected" >&2
    echo "actual release diff:" >&2
    printf '%s\n' "$actual" >&2
    die "release changes must be limited to Cargo.lock and crate manifests"
  fi
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      [ "$#" -ge 2 ] || die "missing value for --version"
      VERSION="$2"
      shift 2
      ;;
    --notes-file)
      [ "$#" -ge 2 ] || die "missing value for --notes-file"
      NOTES_FILE="$2"
      shift 2
      ;;
    --not-latest)
      LATEST=0
      shift
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

require_cmd git
require_cmd just
require_cmd cargo
require_cmd gh

if [ -n "$(git -C "$ROOT" status --short)" ]; then
  die "working tree must be clean"
fi

branch="$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)"
[ "$branch" = "$DEFAULT_BRANCH" ] || die "releases must be cut from $DEFAULT_BRANCH, got $branch"

echo "+ git -C $ROOT fetch origin $DEFAULT_BRANCH"
git -C "$ROOT" fetch origin "$DEFAULT_BRANCH"

git -C "$ROOT" rev-parse --verify "origin/$DEFAULT_BRANCH" >/dev/null 2>&1 ||
  die "missing origin/$DEFAULT_BRANCH tracking ref"

if ! git -C "$ROOT" merge-base --is-ancestor "origin/$DEFAULT_BRANCH" HEAD; then
  die "local $DEFAULT_BRANCH must include origin/$DEFAULT_BRANCH before releasing"
fi

if [ -z "$VERSION" ]; then
  VERSION="$(resolve_next_version)"
fi

case "$VERSION" in
  [0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9].0.[0-9]*)
    ;;
  *)
    die "version must match YYYYMMDD.0.N"
    ;;
esac

[ "$VERSION" != "$(current_release_version)" ] ||
  die "version $VERSION already matches the current crate version"

if git -C "$ROOT" rev-parse -q --verify "refs/tags/$VERSION" >/dev/null 2>&1; then
  die "local tag $VERSION already exists"
fi

if gh release view "$VERSION" >/dev/null 2>&1; then
  die "GitHub release $VERSION already exists"
fi

[ -z "$NOTES_FILE" ] || [ -f "$NOTES_FILE" ] || die "notes file not found: $NOTES_FILE"

run just -f "$ROOT/justfile" bump-version "$VERSION"
run cargo check --manifest-path "$ROOT/Cargo.toml"

if [ "$DRY_RUN" -eq 0 ]; then
  validate_release_diff
fi

run git -C "$ROOT" add Cargo.lock crates/*/Cargo.toml
run git -C "$ROOT" commit -m "chore: bump version to $VERSION"
run git -C "$ROOT" push origin "$DEFAULT_BRANCH"

if [ -n "$NOTES_FILE" ]; then
  if [ "$LATEST" -eq 1 ]; then
    run gh release create "$VERSION" --title "$VERSION" --notes-file "$NOTES_FILE" --latest
  else
    run gh release create "$VERSION" --title "$VERSION" --notes-file "$NOTES_FILE"
  fi
else
  if [ "$LATEST" -eq 1 ]; then
    run gh release create "$VERSION" --title "$VERSION" --generate-notes --latest
  else
    run gh release create "$VERSION" --title "$VERSION" --generate-notes
  fi
fi

echo "release ready: $VERSION"
