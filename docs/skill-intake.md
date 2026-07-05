# Skill Intake

Forge can adapt useful upstream skills, commands, and workflow patterns, but Forge remains the source of truth for the managed copy it ships.

The goal is deterministic cross-machine behavior, not mirroring public skill repositories.

## Intake Rule

Only bring a skill or helper into Forge when it is:

- portable across Ian's machines and repos
- durable enough to version
- high-signal for agent behavior
- compatible with Forge's safety and JSON-first CLI contracts
- small enough to maintain as a Forge-owned surface

Do not import material only because it exists in a popular upstream repo.

## Required Review

Before adding or updating adapted upstream material:

1. Read the upstream source at a concrete commit.
2. Decide whether the skill should be copied, adapted, folded into an existing Forge skill, or rejected.
3. Remove upstream assumptions that are machine-specific, tool-specific, too broad, or inconsistent with Forge policy.
4. Keep the Forge version concise; use references or scripts only when they earn their cost.
5. Add provenance and license notice for any adapted source.
6. Validate from the repo source and the embedded release payload.

## Required Files

For an adapted managed skill:

- `.agents/skills/<skill>/SKILL.md`
- `.agents/skills/<skill>/THIRD_PARTY_NOTICES.md`
- `config/release-skills.toml`
- `embedded_skill!(...)` in `crates/forge/src/main.rs`
- router references when the skill should be discoverable through `forge-tools`

If the skill has scripts or other files, embed them explicitly and preserve executable bits where direct execution is part of the contract.

## Commands

Treat upstream slash commands and command recipes as workflow ideas, not runtime assets, until Forge has a stable managed target for them.

Prefer this sequence:

1. Express the workflow as a skill or doc.
2. Use existing deterministic Forge commands where possible.
3. Add a Forge CLI primitive only after repeated manual shaping proves the contract.
4. Install command assets only when the target runtime surface is stable, inspectable, and reversible.

## Hooks

Hooks are opt-in by default.

Accept a hook into Forge only when it is:

- visible in source
- non-destructive by default
- cheap enough for the lifecycle event
- disableable without editing generated files
- backed by an explicit dry-run or status path

Good candidates:

- repo-local pre-commit validation for managed skill metadata
- lightweight stale-state checks that print guidance only

Poor candidates:

- session-start mutation
- broad install or update hooks
- hooks that fetch the network or write to user config without explicit action

## Verification

For ordinary skill intake, run:

```sh
cargo run -p forge -- skills validate --all --source repo --repo-path /path/to/forge
cargo test -p forge embedded_release_skills
cargo test -p forge embedded_release_skill_frontmatter_names_match_release_names
cargo check
```

For skills with helper scripts, also install to a temporary `path:` target from the release source and smoke test the helper.

## Current Upstream Sources

Current imported or adapted sources:

- `steipete/agent-scripts`: operational helper and review patterns, especially `autoreview`
- `addyosmani/agent-skills`: engineering workflow patterns, adapted into compact Forge-managed skills
- `microsoft/Webwright`: Webwright browser-task skill and reference workflow, adapted into a Forge-managed skill snapshot

Do not assume future upstream changes are automatically accepted. Re-run the intake review for each update.
