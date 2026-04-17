---
name: forge-tools
description: Route Codex to the right Forge CLI skill for Linear, Slack retrieval or assistant actions, Codex session retrieval, or local Forge management. Use when a task may involve more than one Forge CLI or when the correct CLI is not obvious yet.
---

# Forge Tools

Use this skill as the lightweight entry point for the Forge CLI bundle.

Pick the narrowest CLI skill that matches the job:

- `linear-cli`: Linear issue, project, milestone, and viewer workflows.
- `slack-query-cli`: Slack permalink resolution, search, thread reads, and nearby message context.
- `slack-agent-cli`: assistant-oriented Slack thread replies, reactions, file uploads, DMs, and channel joins.
- `codex-threads-cli`: local Codex session sync, search, thread resolution, and event inspection.
- `forge-cli`: local Forge config, permission checks, self-update commands, and managed-surface deployment.
- `autoresearch-create`: start an autonomous experiment loop for measurable optimization work.
- `autoresearch-finalize`: turn an autoresearch branch into clean, reviewable branches.
- `design-algorithm`: shaping and scope-reduction workflow for deciding what should exist before building or automating it.

Default operating rules:

- Use `design-algorithm` when the user is shaping a feature, debating command surface, or deciding whether recurring shell work should become a Forge primitive.
- Prefer the crate-specific skill once the target CLI is clear.
- Prefer `--json` for all reads because it is the deterministic, low-token contract agents consume directly.
- Fetch a small amount of data first with the tool's `--limit` or narrowest read command.
- Treat write commands as explicit actions; do not infer a mutation from a read request.
- Keep work inside the CLI contract instead of reconstructing external API calls yourself.
- Use `jq` only for one-off local reshaping after the CLI has already returned the right record set.
- Keep `rg` for repo and local-file exploration; do not substitute it for a Forge domain command.
- If the same shell shaping keeps recurring, treat that as product feedback to add a narrow Forge primitive rather than normalizing the pipeline in every session.

Adjacent capabilities:

- If the task is about OpenAI product or API guidance rather than a Forge CLI, switch to `openai-docs`.
- If the task is about GitHub issues, pull requests, reviews, Actions, or repository state, prefer the GitHub plugin skills and `gh` CLI instead of forcing the work through Forge.
- Avoid direct native GitHub Codex app flows when an equivalent plugin-skill or `gh` workflow exists, because they are less deterministic and more likely to trigger extra permission prompts.

If the user names a specific Forge binary, switch to that crate skill immediately.

## Inputs

- job-to-be-done + any known system/IDs/URLs

## Output

- next skill to use + first narrow `--json` command

## Checks

- start with a minimal read; expand only if needed
