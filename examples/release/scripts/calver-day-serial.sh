#!/bin/sh
set -eu

usage() {
  cat <<'USAGE'
Usage: calver-day-serial.sh [--dry-run|--apply] [forge release run options]

Chooses the next semver-compatible CalVer version for the current date:
  YYYYMMDD.0.0, YYYYMMDD.0.1, ...

Environment:
  RELEASE_TIMEZONE    Timezone for the date stamp. Default: UTC.
  RELEASE_TAG_PREFIX  Tag prefix configured in release.toml. Default: empty.
USAGE
}

mode="--dry-run"
case "${1:-}" in
  --apply)
    mode="--apply"
    shift
    ;;
  --dry-run)
    mode="--dry-run"
    shift
    ;;
  -h|--help)
    usage
    exit 0
    ;;
esac

timezone="${RELEASE_TIMEZONE:-UTC}"
tag_prefix="${RELEASE_TAG_PREFIX:-}"
today="$(TZ="$timezone" date +%Y%m%d)"
latest_tag="$(git tag --list "${tag_prefix}${today}.0.*" --sort=-version:refname | sed -n '1p')"

if [ -z "$latest_tag" ]; then
  version="${today}.0.0"
else
  latest_version="${latest_tag#"$tag_prefix"}"
  serial="${latest_version##*.}"
  version="${today}.0.$((serial + 1))"
fi

printf 'release version: %s\n' "$version" >&2
exec forge release run "$mode" --version "$version" "$@"
