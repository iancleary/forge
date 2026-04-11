---
name: forge-tools
description: Route Codex to the right Forge CLI skill for Linear, Slack research, Codex session retrieval, or local Forge management. Use when a task may involve more than one Forge CLI or when the correct CLI is not obvious yet.
---

# Forge Tools

Use this skill as the lightweight entry point for the Forge CLI bundle.

Pick the narrowest CLI skill that matches the job:

- `linear-cli`: Linear issue, project, milestone, and viewer workflows.
- `slack-cli-research`: Slack permalink resolution, search, thread reads, and nearby message context.
- `codex-threads-cli`: local Codex session sync, search, thread resolution, and event inspection.
- `forge-cli-admin`: local Forge config, permission checks, and self-update commands.

Default operating rules:

- Prefer the crate-specific skill once the target CLI is clear.
- Prefer `--json` for all reads and for any output that will be chained into another command.
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
