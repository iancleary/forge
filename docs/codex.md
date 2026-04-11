# Codex Source Of Truth

This document defines how `forge` acts as the first-party source of truth for Ian's Codex configuration.

The goal is not to absorb every local dotfile. The goal is to make the portable, durable parts of Codex behavior deterministic and reviewable in one repo.

## What Forge Owns

Forge should own portable Codex behavior such as:

- user-scoped skills that should be available everywhere
- router skills that help Codex choose the right narrower skill
- shared operating workflow skills such as `design-algorithm`
- high-signal policy and workflow guidance that should survive across machines and repos
- documented install and update flows for those assets

Forge should be the place where Ian defines:

- what Codex should know globally
- which portable skills should be available in user scope
- how skill routing should work
- how those assets are installed and updated deterministically

## What Forge Should Not Own

Forge should not become a dump for machine-local accident.

Keep these out of repo unless there is a clear abstraction boundary:

- secrets and tokens
- machine-specific paths and host details
- personal experiments that are not durable policy
- unrelated dotfiles that do not materially affect Codex behavior

## How Codex Knows Which Skill To Use

Codex skill selection should be treated as a product surface.

The main inputs are:

1. skill `description` frontmatter
2. explicit skill mentions such as `$forge-tools`
3. scope and install location
4. repo-local reinforcement such as `AGENTS.md`

### 1. Description Frontmatter

The `description` field is the primary implicit trigger surface.

For Forge-managed skills, descriptions should state:

- what the skill does
- when to use it
- what it should be preferred over
- when not to use it if confusion is likely

Descriptions should not be treated as incidental prose. They are part of the contract.

### 2. Explicit Mentions

Explicit mentions override ambiguity.

If the user or another skill names a skill directly, that is the strongest routing signal short of a hard conflict with the actual task.

### 3. Scope And Install Location

Portable user-scoped Forge skills belong in the Codex `USER` skill location:

```text
$HOME/.agents/skills
```

That location is deterministic. Forge should install managed user-scope skills there via `forge skills install --target user`.

Repo-local skills and repo `AGENTS.md` guidance are still useful, but they should reinforce user-scope behavior rather than replace it.

### 4. Repo-Local Reinforcement

`AGENTS.md` can strengthen routing inside a repo, but it is not the primary cross-repo trigger mechanism.

If a routing rule matters outside one checkout, it should usually become:

- a Forge-managed skill
- a documented policy in Forge
- or both

## Router Pattern

Forge should use router skills to keep routing explicit and compact.

Current pattern:

- `forge-tools` is the entry router for Forge-authored tools
- crate-specific skills such as `linear-cli` and `slack-cli-research` handle domain execution
- shared operating skills such as `design-algorithm` handle shaping and reduction work that crosses domains

Router skills should:

- point to the narrowest useful next skill
- explain the boundary between neighboring skills
- stay short
- avoid duplicating the full body of the skills they route to

## Trigger Contract For Forge Skills

Forge-managed skills should follow this trigger contract:

- the frontmatter `description` is the primary trigger contract
- the body should reinforce boundaries with concise "use this when" and "do not use this when" guidance when needed
- router skills should mention the skills they route to by name
- shared workflow skills should state the expected output shape, not just the philosophy

This is what makes skill routing deterministic enough to be maintained as a first-party system.

## Installation Model

If Forge is the source of truth, then portable Codex assets should be deployable through Forge rather than copied manually.

Current deployable surface:

- Forge-managed skills installed through `forge skills install`

Likely next candidates:

- documented templates for top-level Codex policy files
- generated or installable first-party Codex policy assets where the contract is stable

The boundary should stay narrow. Forge should manage Codex assets that are durable, portable, and worth versioning.

## Acceptance Test For Putting Something In Forge

Move a Codex behavior or asset into Forge when it is:

- portable across machines or repos
- durable enough to version
- high-signal for Codex behavior
- not secret or machine-local
- worth installing or updating deterministically

Keep it out of Forge when it is:

- private
- machine-specific
- experimental noise
- easier to express as a local one-off than as a maintained contract
