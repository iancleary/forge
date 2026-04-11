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

## Official Codex Surfaces

OpenAI's Codex docs treat these as first-class configuration surfaces:

- Config File
- Rules
- AGENTS.md
- Skills

The Skills docs define the deterministic `USER` skill location as `$HOME/.agents/skills`. The Rules docs define user rules under `~/.codex/rules/`, with `~/.codex/rules/default.rules` as the user-layer file Codex writes when approvals are accepted in the UI.

This implies the main portable user-config surfaces Forge should reason about are:

- user skills
- user `AGENTS.md`
- user rules
- selected config templates or fragments

## Current Local Files: Keep, Move, Or Fold

Based on the current local Codex directory, this is the recommended plan.

The important distinction is:

- source files you want to author and version in Forge
- installed runtime files Codex actually consumes from standard locations

Do not mirror the entire `~/.codex/` tree into Forge.

### Move Into Forge As First-Party Sources

These are durable enough to belong in repo as managed source material:

- `~/.codex/agents.md`
- `~/.codex/rules/user-policy.rules`
- selected user-scoped skills already managed under `.agents/skills/`

Recommended repo shape:

- a repo-managed user `AGENTS.md` template or installable asset
- a repo-managed user rules template or installable asset
- Forge-managed skills as the deployable user-scope skill layer

### Fold Into A Smaller Managed Surface

These local files look like content, not canonical runtime surfaces:

- `~/.codex/principles.md`
- `~/.codex/soul.md`

Recommendation:

- do not keep them as separate runtime files
- preserve them only as Forge authoring fragments when they still add value

Reason:

- they overlap with voice, working style, and decision heuristics already better expressed in `AGENTS.md`
- keeping them separate creates more drift without giving Codex a clearer documented contract

### Keep Local Or Private

These should stay out of Forge-managed source-of-truth scope:

- `~/.codex/auth.json`
- `~/.codex/installation_id`
- `~/.codex/history.jsonl`
- `~/.codex/session_index.jsonl`
- `~/.codex/sessions/`
- `~/.codex/archived_sessions/`
- `~/.codex/log/`
- `~/.codex/logs_2.sqlite*`
- `~/.codex/state_5.sqlite*`
- `~/.codex/cache/`
- `~/.codex/plugins/cache/`
- `~/.codex/vendor_imports/`
- backups such as `*.backup.*` and `*.bak`

These are either secrets, caches, machine-local state, or generated artifacts.

### Keep Mostly Local, But Define Templates In Forge

`~/.codex/config.toml` should usually not be copied into Forge verbatim.

Your current file mixes:

- portable defaults such as model, reasoning effort, and personality
- machine-local trust entries
- connector-specific approval state
- local installation details

Recommendation:

- keep the live `config.toml` local
- define a documented Forge-owned template or fragment set for the portable parts
- do not put machine-specific trust entries or connector IDs into the repo

Good candidates for a Forge-owned config template:

- preferred default model
- preferred default reasoning effort
- portable personality choices
- stable plugin enablement that is not machine-specific

Bad candidates:

- per-path trust levels
- connector installation IDs
- generated approval history
- local-only environment assumptions

## Proposed Repo Layout For User Config

If Forge expands beyond skills, the next clean shape is:

- keep skill sources in `.agents/skills/`
- add a dedicated repo area for first-party Codex user config sources

Suggested shape:

```text
codex/
  user/
    AGENTS.md
    rules/
      user-policy.rules
    config/
      config.toml.example
      config.portable.toml
    fragments/
      principles.md
      characters/
        pragmatic-builder.md
```

Use this directory as source material, not as the live installed location.

Deployment model:

- install skills to `$HOME/.agents/skills`
- install or render `AGENTS.md` to `~/.codex/AGENTS.md` if Forge eventually manages it
- install or render user rules to `~/.codex/rules/`
- keep live machine-specific config local, with optional Forge-generated fragments

Recommended interpretation:

- `codex/user/AGENTS.md` is the runtime target source
- `codex/user/fragments/` is optional authoring input and not part of the runtime contract
- `codex/user/config/` is templates and portable fragments, not a promise to fully own the live local config

## Better Than A Raw Dotfiles Mirror

Your current dotfiles layout is a useful historical input, but Forge should probably improve on it rather than copy it exactly.

The dotfiles pattern:

- `agents.md`
- `principles.md`
- `soul.md`
- `config.toml`
- `rules/user-policy.rules`

is understandable as an authoring system, but the official Codex surfaces are narrower. My recommendation is:

- keep `AGENTS.md`, rules, and skills as the main managed Codex runtime surfaces
- treat `principles.md` and character fragments as optional source inputs, not required runtime files
- treat `config.toml` as template-driven and only partially managed

That gives you a cleaner contract:

- fewer runtime files
- less drift
- closer alignment with the documented Codex model
- still enough room to preserve your preferred voice and heuristics upstream in Forge

## Skill Metadata Opportunity

The Codex Skills docs also support optional `agents/openai.yaml` metadata, including:

- display metadata
- dependency declarations
- `allow_implicit_invocation`

That suggests another useful refinement for Forge-managed skills:

- add `agents/openai.yaml` to the skills where UI metadata or invocation policy materially improves routing
- consider setting `allow_implicit_invocation: false` on skills that should only run by explicit mention or router handoff

This is likely better than overloading `AGENTS.md` with all routing nuance.

## Characters And Roles

Do not make `soul.md` a required runtime file.

If you want switchable voices or roles, the cleaner model is:

- keep the installed runtime surface small
- keep character definitions as Forge authoring fragments
- later add an explicit Forge command that renders or installs a chosen character into the managed user `AGENTS.md`

Recommended source shape:

- `codex/user/fragments/characters/*.md`

Potential future CLI surface:

- `forge codex profile list`
- `forge codex profile preview <name>`
- `forge codex profile apply <name>`

That is better than proliferating undocumented runtime files in `~/.codex/`.

## Recommended Next Steps

1. Add a Forge-managed source file for the user `AGENTS.md` baseline.
2. Add a Forge-managed source file for `user-policy.rules`.
3. Keep `principles.md` and character fragments as authoring inputs only, not as separate runtime files.
4. Add a documented portable config template, not a full managed replacement for live `config.toml`.
5. Evaluate `agents/openai.yaml` for the Forge-managed skills where invocation policy or dependencies would improve determinism.
6. Decide later whether Forge should install these assets directly or generate them from templates.

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
