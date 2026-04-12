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

If Forge is also the first-party source of truth for Codex configuration, then these managed skills are not just convenience installs. They are the deployable portable policy surface for Codex user-scope behavior.

That skill lifecycle is separate from `forge codex render|diff|install`, which manages the narrow v1 set of user-scoped Codex files under `~/.codex/`.

## Managed Skills

Initial Forge-managed skills:

- `design-algorithm`
- `gh-body-file`
- `forge-tools`
- `linear-cli`
- `slack-query-cli`
- `slack-agent-cli`
- `codex-threads-cli`
- `forge-cli`

Repo source of truth during development:

```text
<forge-repo>/.agents/skills/
```

Release source of truth for consumers:

- the skill payload bundled with the installed Forge release

The trigger contract for those skills is documented in `docs/codex.md`.

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

Output contract for `forge skills ...`:

- human-readable text by default
- compact JSON envelope with `--json`
- no pretty-printed JSON on the agent path

### `forge skills list`

```sh
forge skills list [--source repo|release|all] [--repo-path <path>] [--json]
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
forge skills validate [<skill>|--all] [--source repo|release] [--repo-path <path>] [--json]
```

Checks:

- skill directory exists
- `SKILL.md` exists
- frontmatter includes `name` and `description`
- skill name matches folder name policy where applicable
- router references point to skills that exist in the current Forge source

The intent is to keep the user-scope skill surface deterministic. Descriptions and router references are part of the maintained contract, not free-form text.

### `forge skills install`

```sh
forge skills install <skill> [--target user|forge_repo|path:<abs-path>] [--source release|repo] [--repo-path <path>] [--target-role mainline|development] [--force] [--force-unmanaged] [--json]
forge skills install --all [--target user|forge_repo|path:<abs-path>] [--source release|repo] [--repo-path <path>] [--target-role mainline|development] [--force] [--force-unmanaged] [--json]
```

Installs or updates Forge-managed skills to a target.

Behavior:

- `--source repo` requires `--repo-path <path>`
- `--source release` uses the installed Forge release payload
- when omitted, Forge resolves the source automatically:
  - use `repo` only when `--repo-path` is provided
  - otherwise use `release`
- `--target` defaults to `user` (installs into `~/.agents/skills`)
- target role defaults:
  - `user` => `mainline`
  - `forge_repo` => `development`
  - `path:<abs-path>` => `development`
- use `--target-role mainline` when a non-user target should be treated as part of the primary managed install set
- overwrite is allowed by default for existing Forge-managed targets
- overwrite fails for unmanaged collisions unless `--force-unmanaged` is set

### `forge skills diff`

```sh
forge skills diff <skill> [--target user|forge_repo|path:<abs-path>] [--source release|repo] [--repo-path <path>] [--json]
```

Shows differences between the current Forge source copy and the installed target.

### `forge skills revert`

```sh
forge skills revert <skill> [--target user|forge_repo|path:<abs-path>] [--repo-path <path>] [--target-role mainline|development] [--json]
forge skills revert --all [--target user|forge_repo|path:<abs-path>] [--repo-path <path>] [--target-role mainline|development] [--json]
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

Treat this as a deterministic Codex location, not a Forge-configurable alias. For testing or non-default installs, use `path:<abs-path>` instead of redefining `user`.

For a Forge-first Codex setup, this is the primary deploy target for portable skills that should be available across repos.

### `forge_repo`

A path relative to the explicit `--repo-path <path>` Forge repo root.

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
source_repo_slug = "owner/forge"
source_repo_path = "/abs/path/to/forge"
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
target_path = "/Users/alice/.agents/skills/linear-cli"
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
- whether the managed Codex baseline is stale relative to the active Forge source

Recommended result shape:

```json
{
  "ok": true,
  "data": {
    "source_kind": "release",
    "update_available": true,
    "skills_out_of_date": true,
    "managed_skill_count": 6,
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

- updates the active Forge release
- reconciles managed skill installs against that release
- reconciles `mainline` managed skill installs against that release by default
- overwrites Forge-managed targets as needed
- leaves unmanaged collisions untouched unless the user explicitly took ownership beforehand
- compares the running Forge version to the newest repo tag
- installs the newest tagged release with Cargo when needed
- then uses the installed release payload as source
- uses the release tool contract to migrate declared legacy config dirs and remove declared legacy binaries safely
- uses the release skill contract to migrate declared legacy managed skill names and installed directories safely

Release skill naming and migration are defined in `config/release-skills.toml`.

That file is the release-scoped source of truth for:

- current Forge-managed skill names
- legacy skill names that should migrate to a current name during `forge self update`

Forge does not guess skill renames. Any managed skill rename must be declared there explicitly.

## Switching Back To Standard Install

Consumers may temporarily install Forge skills from a local checkout for testing.

They must have a clean path back to the normal release-backed install.

Supported pattern:

1. User installs from checkout:

```sh
forge skills install linear-cli --source repo --repo-path /abs/path/to/forge --json
```

2. Later the user returns to the standard installed source:

```sh
forge skills revert linear-cli --target user --json
```

or:

```sh
forge skills install linear-cli --source release --json
```

Persistent non-user target example:

```sh
forge skills install --all --source repo --repo-path /abs/path/to/forge --target path:/opt/forge-skills --target-role mainline --json
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
- treating skill descriptions and routing boundaries as incidental prose instead of a maintained trigger contract

## Review Notes

This contract is intentionally narrow.

Good:

- Forge remains the single source of truth for Forge-managed skills
- shared operating workflow skills can propagate with normal Forge installs, not only inside the repo checkout
- managed overwrite behavior is explicit and predictable
- both local-dev and consumer installs have a first-class source model
- consumers can cleanly revert from repo-sourced testing back to the standard release install

Tradeoff:

- manifest ownership and target resolution add some statefulness
- that complexity is justified because it eliminates ambiguous overwrite behavior and cwd-based guessing
