# Forge Skills

This document defines the Forge-specific lifecycle management contract for consumer skills shipped with Forge.

This is not a general skill manager. It only manages Forge-authored skills and their installed targets.

Recommended adjacent Codex capabilities such as `openai-docs` and GitHub plugin skills may still be useful in a full Codex setup, but they are out of scope for Forge-managed skill lifecycle commands. For GitHub work, prefer `gh`-driven and plugin-skill workflows over direct native GitHub Codex app usage when both can handle the task.

## Goal

Treat Forge as the single source of truth for Forge-managed consumer skills while supporting two source modes:

- `repo_checkout`: the user is working from a local Forge checkout
- `release`: the user installed Forge normally and is not doing local development

In both modes:

- the Forge source is canonical
- installed skill copies are deployed artifacts
- Forge may overwrite managed installed copies during explicit install or update flows

## Managed Skills

Initial Forge-managed skills:

- `forge-tools`
- `linear-cli`
- `slack-cli-research`
- `codex-threads-cli`
- `forge-cli-admin`

Repo source of truth during development:

```text
.agents/skills/
```

Release source of truth for consumers:

- the skill payload bundled with the installed Forge release

## Ownership Model

Forge distinguishes between:

- Forge-managed skill installs
- unmanaged user skills that happen to share a name

Forge-managed installs may be overwritten by `forge skills install` or `forge self update`.

Unmanaged collisions must fail unless the user explicitly opts into taking ownership.

This is the key rule:

- overwriting Forge-managed installed skills is normal deployment behavior
- overwriting unmanaged skill directories is not

## Commands

### `forge skills list`

```sh
forge skills list [--source repo|release|all] [--json]
```

Lists Forge-managed skills available from the current source and known installed targets.

### `forge skills status`

```sh
forge skills status [--scope mainline|development|all] [--target user|forge_repo|path:<abs-path>] [--json]
```

Summarizes Forge-managed skill state across configured targets.

Default behavior:

- scope defaults to `mainline`
- this keeps consumer-facing status focused on the installs that `forge self update-check` and `forge self update` care about by default

States:

- `up_to_date`
- `out_of_date`
- `diverged`
- `missing`
- `unmanaged_collision`

### `forge skills validate`

```sh
forge skills validate [<skill>|--all] [--json]
```

Checks:

- skill directory exists
- `SKILL.md` exists
- frontmatter includes `name` and `description`
- skill name matches folder name policy where applicable
- router references point to skills that exist in the current Forge source

### `forge skills install`

```sh
forge skills install <skill> [--target user|forge_repo|path:<abs-path>] [--source release|repo] [--target-role mainline|development] [--force] [--force-unmanaged] [--json]
forge skills install --all [--target user|forge_repo|path:<abs-path>] [--source release|repo] [--target-role mainline|development] [--force] [--force-unmanaged] [--json]
```

Installs or updates Forge-managed skills to a target.

Behavior:

- `--source repo` uses the configured Forge repo checkout
- `--source release` uses the installed Forge release payload
- when omitted, Forge resolves the source automatically:
  - use `repo` if a valid Forge repo checkout is configured
  - otherwise use `release`
- target role defaults:
  - `user` => `mainline`
  - `forge_repo` => `development`
  - `path:<abs-path>` => `development`
- use `--target-role mainline` when a non-user target should be treated as part of the primary managed install set
- overwrite is allowed by default for existing Forge-managed targets
- overwrite fails for unmanaged collisions unless `--force-unmanaged` is set

### `forge skills diff`

```sh
forge skills diff <skill> [--target user|forge_repo|path:<abs-path>] [--source release|repo] [--json]
```

Shows differences between the current Forge source copy and the installed target.

### `forge skills revert`

```sh
forge skills revert <skill> [--target user|forge_repo|path:<abs-path>] [--target-role mainline|development] [--json]
forge skills revert --all [--target user|forge_repo|path:<abs-path>] [--target-role mainline|development] [--json]
```

Switches a managed target back to the standard Forge install source.

This command exists for the case where a consumer temporarily deployed a skill from a local checkout and now wants to return to the standard installed Forge release.

Behavior:

- resolves the canonical source to `release`
- overwrites the target if the target is Forge-managed
- updates the install manifest to record `source_kind = "release"`
- fails clearly if the target is unmanaged unless `--force-unmanaged` is set on install first

## Target Model

Avoid a generic `repo` target because it is ambiguous.

Supported target kinds:

- `user`
- `forge_repo`
- `path:<abs-path>`

### `user`

The Codex user skill directory.

This location should default to:

```text
~/.agents/skills
```

Forge may allow an explicit config override, but the Codex-compatible default should match the documented user skill location.

### `forge_repo`

A path relative to the configured Forge repo root.

Use a fixed subpath rather than cwd inference.

Example:

```toml
forge_repo_install_subpath = ".agents/skills-installed"
```

Resolution rule:

- resolve relative to configured `repo_path`
- fail if `repo_path` is missing or invalid

### `path:<abs-path>`

An explicit absolute filesystem path.

Use this when the user wants a deterministic non-default target.

## Source Model

Do not use filesystem path alone as the source identity.

Forge tracks two source modes:

- `repo_checkout`
- `release`

Canonical source identity fields:

- `source_repo_slug`
- `source_ref`
- `source_hash`

Optional local provenance:

- `source_repo_path`

### `repo_checkout`

Used when Forge is reading skills from a configured local checkout.

Example:

```toml
source_kind = "repo_checkout"
source_repo_slug = "iancleary/forge"
source_repo_path = "/Users/iancleary/Development/forge"
source_ref = "main"
source_hash = "abc123"
```

### `release`

Used when Forge is reading skills from the installed Forge release payload.

Example:

```toml
source_kind = "release"
source_repo_slug = "iancleary/forge"
source_ref = "2026.0411.0"
source_hash = "def456"
```

For normal consumers, `release` is the default and does not require a local Git checkout.

## Manifest

Forge records managed installs in:

```text
~/.config/forge/state.toml
```

The state file is Forge-managed internal metadata. It is not intended to be hand-edited.

During early development, the schema may change. If a newer Forge build can no longer parse an older `state.toml`, the supported recovery is:

1. remove or archive the old state file
2. rerun `forge skills install ...` for the targets you want Forge to manage

Longer-term migrations can be added later if the state format stabilizes and compatibility becomes worth preserving in code.

Suggested shape:

```toml
[[managed_skill_installs]]
skill_name = "linear-cli"
managed_by = "forge"

source_kind = "release"
source_repo_slug = "iancleary/forge"
source_ref = "2026.0411.0"
source_hash = "def456"

target_kind = "user"
target_role = "mainline"
target_path = "/Users/alice/.codex/skills/linear-cli"
installed_at = "2026-04-11T14:22:00Z"
state = "up_to_date"
```

Optional local-dev provenance:

```toml
source_repo_path = "/Users/alice/Development/forge"
```

## Update Behavior

### `forge self update-check`

Checks:

- whether the Forge repo or installed release is out of date
- whether any mainline managed skill install is stale relative to the active Forge source

Recommended result shape:

```json
{
  "ok": true,
  "data": {
    "source_kind": "release",
    "update_available": true,
    "skills_out_of_date": true,
    "managed_skill_count": 5,
    "skills": [
      {
        "name": "linear-cli",
        "target_kind": "user",
        "state": "out_of_date"
      }
    ]
  }
}
```

### `forge self update`

Behavior:

- updates the active Forge source
- reconciles managed skill installs against that source
- reconciles `mainline` managed skill installs against that source by default
- overwrites Forge-managed targets as needed
- leaves unmanaged collisions untouched unless the user explicitly took ownership beforehand

In `repo_checkout` mode:

- update the local repo first
- then reconcile managed installs from the repo source

In `release` mode:

- use the installed release payload as source
- reconcile managed installs without requiring Git

## Switching Back To Standard Install

Consumers may temporarily install Forge skills from a local checkout for testing.

They must have a clean path back to the normal release-backed install.

Supported pattern:

1. User installs from checkout:

```sh
forge skills install linear-cli --source repo --target user --json
```

2. Later the user returns to the standard installed source:

```sh
forge skills revert linear-cli --target user --json
```

or:

```sh
forge skills install linear-cli --source release --target user --json
```

Persistent non-user target example:

```sh
forge skills install --all --source repo --target path:/opt/forge-skills --target-role mainline --json
forge skills status --target path:/opt/forge-skills --json
```

After revert:

- the deployed skill content comes from the installed Forge release
- the manifest records `source_kind = "release"`
- future `self update-check` and `self update` compare against the standard installed source again

## Anti-Patterns To Avoid

- hidden skill install during unrelated commands
- generic `repo` target semantics tied to cwd
- assuming the user ran Forge from a Git root
- treating deployed skill copies as peer sources of truth
- overwriting unmanaged skill collisions by default
- making Forge a general-purpose skill package manager

## Review Notes

This contract is intentionally narrow.

Good:

- Forge remains the single source of truth for Forge-managed skills
- managed overwrite behavior is explicit and predictable
- both local-dev and consumer installs have a first-class source model
- consumers can cleanly revert from repo-sourced testing back to the standard release install

Tradeoff:

- manifest ownership and target resolution add some statefulness
- that complexity is justified because it eliminates ambiguous overwrite behavior and cwd-based guessing
