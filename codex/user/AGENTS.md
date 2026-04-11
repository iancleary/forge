# AGENTS.md — User Baseline

Portable Codex operating manual managed by Forge.

This file is intended as the baseline user-scoped `~/.codex/AGENTS.md` source, not as a repo-local project override.

## Before You Start

- Read the project `AGENTS.md`, `CLAUDE.md`, or `README.md` for repo-specific conventions.
- Check `git status` and the current branch before making changes.
- Identify the toolchain from files such as `justfile`, `Cargo.toml`, and `package.json`.
- Do not assume conventions from one repo apply to another.

## Autonomy

Do freely:

- read files and explore the codebase
- run checks, tests, and linters
- create local branches
- write code and commit locally

Ask first:

- push to a remote
- create or edit PRs in public repositories
- delete remote branches
- publish packages or make other public-facing changes

## Git

- No `Co-Authored-By` lines in commits.
- Use one branch per feature or fix.
- Prefer worktrees for parallel work when helpful.
- Clean up branches after merge.
- Use conventional commits when the repo already uses them.

## Tools

- Prefer `rg` for search and `eza`/`bat` for local inspection when available.
- In Rust repos, prefer `just` tasks over raw cargo when a `justfile` exists.
- Use the repo's documented toolchain instead of forcing a preferred stack.

## Safety

- Never commit secrets or credentials.
- Prefer safe, explicit operations over destructive shortcuts.
- Treat private repositories and local state as private by default.

## Workflows

### Debugging

- Investigate root cause before proposing fixes.
- Read the full error, reproduce it, inspect recent changes, and trace backward from the failure.
- If multiple fix attempts fail, stop and reassess the architecture instead of piling on patches.

### Planning

- For non-trivial work, explore context before coding.
- Break multi-file work into concrete tasks with file paths, verification steps, and intended outcomes.
- Prefer narrow, testable increments.

### Code Review

- Review diffs for bugs, regressions, missing tests, style drift, and unsafe assumptions.
- Lead with findings, not summaries.

### Skill Routing

- Treat skill `description` frontmatter as the primary routing contract.
- Use router skills first when the right narrow skill is not obvious.
- Prefer user-scoped skills installed under `$HOME/.agents/skills` for portable cross-repo behavior.
- Let repo-local `AGENTS.md` files refine project behavior, not replace the portable user baseline.

## Notes

- Durable voice and decision heuristics may be authored in Forge fragments, but they should not require separate runtime files.
- Character or role switching, if introduced later, should be modeled as explicit Forge-managed profiles rather than ad hoc file sprawl in `~/.codex/`.
