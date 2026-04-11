---
name: linear-cli
description: Use the Forge `linear` CLI for Linear issue, project, milestone, team, and viewer workflows. Prefer this over raw GraphQL when the task can be handled through the local CLI. See `forge-tools` first if the right Forge CLI is not obvious.
---

# Linear CLI

This skill covers the Forge `linear` binary. If the task might belong to another Forge CLI, check `forge-tools` first and then return here once `linear` is clearly the right tool.

Start with the smallest command that proves context:

- `linear --json viewer`
- `linear --json team list`
- `linear --json project list --limit 20`
- `linear --json issue read <ISSUE-ID>`

Working rules:

- Use `--json` for reads because the skill is consumed by agents, not by a human scanning terminal output directly.
- Reuse IDs from prior CLI JSON instead of guessing names.
- Prefer `issue read` before `issue update`.
- Prefer `project list` or `team list` before asking for identifiers the CLI can discover.
- Use `--description-file` for longer bodies instead of fragile shell quoting.

Safety:

- `issue create`, `issue update`, `milestone create`, `milestone update`, and `milestone delete` are mutations.
- Do not mutate Linear state unless the user asked for that outcome.
- `milestone delete` requires explicit intent and should include `--force` only when deletion is clearly requested.

Common flow:

1. Confirm auth with `viewer`.
2. Discover IDs with `team list`, `project list`, or `issue read`.
3. Perform the narrow read or explicit mutation the user requested.
4. Return the key fields from JSON rather than dumping full payloads.
