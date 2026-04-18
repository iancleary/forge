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
- take destructive actions with material blast radius

## Git

- No `Co-Authored-By` lines in commits.
- Use one branch per feature or fix.
- Prefer worktrees for parallel work when helpful.
- Run mutating git commands serially, not in parallel.
- Do not overlap `git add`, `git commit`, `git push`, branch updates, or other index-writing commands in concurrent tool calls.
- Clean up branches after merge.
- Use conventional commits when the repo already uses them.

## Tools

- Prefer `rg` for search and `eza`/`bat` for local inspection when available.
- In Rust repos, prefer `just` tasks over raw cargo when a `justfile` exists.
- Use the repo's documented toolchain instead of forcing a preferred stack.
- Prefer deterministic, low-noise product surfaces over ad hoc shell reconstruction when the tool already exists.
- Prefer narrow reads: use `--json`, small limits, and targeted queries.
- Extract only the few fields needed to proceed; do not paste full payloads.

## Safety

- Never commit secrets or credentials.
- Prefer safe, explicit operations over destructive shortcuts.
- Treat private repositories and local state as private by default.
- Avoid actions that create destructive trajectory even when a single step looks locally reversible.
- Prefer preview, diff, and verification before apply when a command mutates state.

## Workflows

### GitHub

- Prefer `gh` for GitHub issue, pull request, review, and comment workflows when an equivalent CLI path exists.
- For substantial issue bodies, pull request bodies, or markdown-heavy updates, write the content to a local markdown file and pass it with `--body-file` when the CLI supports it.
- Keep short one-line bodies inline only when there is no meaningful quoting or interpolation risk.
- Prefer file-backed bodies because they are easier to review and less likely to break on backticks, `$HOME`-style paths, angle brackets, or multiline markdown.

### Debugging

- Investigate root cause before proposing fixes.
- Read the full error, reproduce it, inspect recent changes, and trace backward from the failure.
- If multiple fix attempts fail, stop and reassess the architecture instead of piling on patches.
- Do not hide uncertainty with confident language. State what is known, what is inferred, and what still needs verification.

### Planning

- For non-trivial work, explore context before coding.
- Break multi-file work into concrete tasks with file paths, verification steps, and intended outcomes.
- Prefer narrow, testable increments.
- Optimize for speed with bounded risk, not for motion alone.

### Code Review

- Review diffs for bugs, regressions, missing tests, style drift, and unsafe assumptions.
- Lead with findings, not summaries.

### Mutation Discipline

- Keep read, diff, and validation paths cheap and normal.
- Keep public, destructive, or high-blast-radius actions explicit.
- Minimize approval prompts by making the safe path obvious, not by broadening risky defaults.
- When changing user-facing or persistent state, prefer: inspect -> diff/preview -> apply -> verify.

### Skill Routing

- Treat skill `description` frontmatter as the primary routing contract.
- Use router skills first when the right narrow skill is not obvious.
- Prefer user-scoped skills installed under `~/.agents/skills` for portable cross-repo behavior.
- Let repo-local `AGENTS.md` files refine project behavior, not replace the portable user baseline.
- Distinguish workflow-maintenance skills from executable repo commands: if a repo already has a documented task runner or checked-in script for the actual job, use that command for ordinary execution and use the skill only when creating, auditing, or changing the workflow.
- In repos like Forge, that means `create-release-process` maintains the release workflow, while the `cut-release` skill executes the workflow by calling `just cut-release`.

## Notes

- Durable heuristics may be authored in Forge fragments, but they should not require separate runtime files.
- The user baseline should optimize for truth, safety, speed, and deterministic behavior rather than personality.
- When Forge manages this baseline, prefer `forge codex diff` before `forge codex install` so mutation stays explicit and reviewable.
