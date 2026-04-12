# Agent-Friendly CLIs

This file turns the OpenAI "agent-friendly CLI" idea into a working spec for this repo.

Source material:

- OpenAI use case: `https://developers.openai.com/codex/use-cases/agent-friendly-clis`
- Referenced skill: `https://github.com/openai/skills/tree/main/skills/.curated/cli-creator`
- Original examples discussed here: `codex-threads`, `slack-query`, `typefully-cli`

## Why Build These

When Codex keeps needing the same external system, raw connectors and copied docs are often too noisy. A small CLI gives the agent something it can already use well:

- Exact commands
- Stable flags
- `--help`
- Machine-readable JSON
- Predictable errors
- Easy retry and composition

The goal is not "replace APIs" or "replace MCP/connectors". The goal is to wrap the few operations we actually use into commands that are easy for an agent to discover and chain together.

## Design Sequence

Apply the repo algorithm from `docs/algorithm.md` when deciding whether a pain point should become CLI surface:

1. Question the requirement.
2. Delete any part or process that is not load-bearing.
3. Simplify and optimize the remaining contract.
4. Accelerate the loop.
5. Automate last.

For agent-friendly CLIs, this usually means deleting repeated shell shaping and unnecessary output before adding more commands or flags.

When this workflow should travel with Forge installs rather than stay repo-local, ship it as a managed skill in addition to documenting it here.

## What Good Looks Like

An agent-friendly CLI should have:

- A clear noun/verb command shape
- `--json` output for every read command
- Stable field names across versions
- Narrow subcommands for common tasks
- Good `--help` text
- Predictable exit codes
- Errors written to stderr
- Support for large result sets via paging or limits
- Safe defaults for write actions

Examples:

```sh
slack-query search "app server auth" --all-pages --max-pages 3 --json
slack-query resolve-permalink "https://openai.slack.com/archives/..."
slack-query read-thread L143 123522523239.633199 --json
slack-query context R152 25723525099.626199 --before 5 --after 5 --json
```

```sh
codex-threads --json sync
codex-threads --json messages search "build a CLI" --limit 20
codex-threads --json threads resolve "tweet idea"
codex-threads --json threads read <session-id>
codex-threads --json events read <session-id> --limit 50
```

```sh
typefully-cli --json drafts list --social-set <id> --limit 20
typefully-cli --json drafts read --social-set <id> <draft-id>
typefully-cli --json drafts create --social-set <id> --body-file draft.json
typefully-cli --json media upload --social-set <id> ./image.png
typefully-cli --json queue schedule-read --social-set <id>
```

## CLI Contract

Every CLI we build in this repo should try to follow this contract.

### Command design

- Binary name should be explicit: `<tool>-cli`
- Prefer `resource action` subcommands
- Avoid overloaded flags that mean different things in different subcommands
- Keep the first version small and task-shaped

Preferred pattern:

```sh
<tool>-cli [global-options] <resource> <action> [args] [flags]
```

Examples:

```sh
github-notes-cli issues search "rate limit"
notion-cli pages read <page-id>
linear-lite-cli issues list --team ENG --limit 25
```

### Output design

- Reads should support `--json`
- JSON should be compact, stable, and documented
- Human-readable mode can exist, but JSON is the primary interface for agents
- Include IDs the next command can reuse

For Forge-style CLIs, "low-token, deterministic" means:

- the command returns only the fields needed for the next decision or follow-up command
- field names and shapes stay stable across runs
- the command supports narrow reads with `--limit`, paging, or a specific read target
- errors are shaped and actionable instead of forcing the agent to parse prose
- the CLI does the repeated normalization work once so every future agent run does not rebuild the same shell pipeline

Preferred top-level JSON shape:

```json
{
  "ok": true,
  "data": [],
  "next_cursor": null
}
```

Preferred error shape:

```json
{
  "ok": false,
  "error": {
    "code": "not_found",
    "message": "Draft not found",
    "details": {
      "draft_id": "dr_123"
    }
  }
}
```

### Error behavior

- Exit `0` on success
- Exit non-zero on failure
- Print friendly summaries to stderr
- If `--json` is present, emit structured JSON errors
- Use distinct error codes where practical: `auth_error`, `rate_limited`, `not_found`, `validation_error`

### Paging and limits

- Large reads must support `--limit`
- If the source system pages, expose a cursor or page token
- Support "fetch a little first" workflows
- Do not dump huge payloads by default

## Forge CLI-First Policy

When Forge already has a CLI for the target system, prefer the Forge command first and fall back to shell tools only when the job is still exploratory or one-off.

Use the Forge CLI first when:

- the operation is part of the stable product contract
- the same post-processing keeps showing up across sessions
- the agent needs reusable IDs, normalized fields, or predictable error codes
- the output will likely be chained into another command
- the agent would otherwise need the same `jq` transformation repeatedly

Use `jq` as an acceptable fallback when:

- the CLI already returned the right record set and only a one-off local projection is needed
- you are validating or exploring a new output shape before deciding whether Forge should absorb it
- the transformation is local presentation, not part of the CLI contract

Do not treat repeated `jq` pipelines as the steady state. If the same projection keeps recurring, that is product feedback that the CLI probably needs a narrower flag, subcommand, or output mode.

Use `rg` as the right tool when:

- the task is repository or local-file exploration rather than an external system contract
- the search space is unstructured text and the user is still discovering where the target lives
- you are looking for implementation references, docs, tests, or config snippets

Do not pull `rg`-style exploration into Forge unless the search target is part of a stable domain-specific contract and the CLI can return meaningfully normalized results instead of raw text matches.

## Folding Pain Into Forge

Use this pattern to decide when recurring agent pain should become a Forge primitive:

1. Notice repetition. The same shell pipeline, copied query, or cleanup step appears across multiple sessions.
2. Confirm the target is stable. The agent is not just exploring; it is trying to perform the same real task each time.
3. Isolate the narrow job. Name the exact thing the agent wants, such as "list projects with IDs and status" rather than "make the JSON nicer."
4. Define the minimal contract. Pick the fields, flags, limits, and error codes that remove the repeated post-processing.
5. Keep the first cut small. Add one narrow flag, view, or subcommand instead of a generic query language.
6. Leave exploration outside. Keep `jq` and `rg` available for ad hoc shaping and repo discovery.

Heuristics for opening a follow-up Forge change instead of writing another shell pipeline:

- the same `jq` expression has appeared three or more times
- the agent repeatedly drops the same noisy fields before it can reason
- the same IDs or summaries are needed for the next command every time
- mistakes are coming from inconsistent local shaping rather than the source API itself
- the desired output can be described as a stable command example in the docs

### Safety model

Write commands should default to the least risky behavior.

- Reads should be safe by default
- Creates should avoid irreversible side effects unless explicitly requested
- Publish, delete, overwrite, or schedule actions should require explicit verbs or flags
- Prefer dry-run or preview modes where possible

Examples:

- Create a draft, not a published post
- Read queue state, do not mutate queue state unless asked
- Upload media without auto-attaching or auto-publishing

## Skill Pairing

The binary is only half the solution. The agent should also have a skill or operating note that says:

- Which commands to try first
- Always use `--json` unless a human-readable view is requested
- How much data to fetch on first pass
- Which write actions need explicit confirmation
- When to use a temp body file instead of shell quoting

Example policy for a content tool:

- Use `drafts create`, not publish
- Use `--body-file` for long content
- Never schedule, publish, overwrite, or delete unless explicitly requested

### Recommended packaging for Forge-style bundles

When a repo ships multiple agent-friendly CLIs, prefer:

- one narrow consumer skill per binary
- one lightweight router skill for discovery across the bundle

For this repo, that means:

- `linear-cli`
- `slack-query-research`
- `codex-threads-cli`
- `forge-cli-admin`
- `forge-tools` as a short entry point that routes Codex to the correct crate skill

Keep the router skill short. It should help Codex choose the right CLI, not duplicate every command rule from every crate skill.

## Build Workflow For Each Tool

Use this sequence when turning one of your recurring tools into a CLI.

1. Identify repeated tasks
2. Ignore the full API surface
3. Pick the 3-7 operations you actually use
4. Design small subcommands around those operations
5. Define the JSON output before implementation
6. Add help text and examples
7. Add auth handling and predictable errors
8. Add paging and limits
9. Wrap the CLI in a skill or repo doc with safe usage guidance

## Tool Selection Rubric

A tool is a good candidate when:

- You keep handing Codex the same docs, exports, or logs
- The raw source is too noisy for direct use
- You only need a narrow part of a much larger API
- You want repeatable commands, not repeated explanation
- You care about safe defaults for write actions

Bad candidates:

- One-off tasks
- Systems where raw files are already clean and local
- APIs where you genuinely need broad exploratory access every time

## Starter Template

Use this as the starting shape for a new CLI:

```text
Goal:
- Give Codex stable access to <tool> for <primary jobs>

Users:
- Me
- Codex acting on my behalf

Core commands:
- <tool>-cli <resource> list
- <tool>-cli <resource> read <id>
- <tool>-cli <resource> search <query>
- <tool>-cli <resource> create ...

Global flags:
- --json
- --limit <n>
- --cursor <token>
- --profile <name>
- --verbose

Safety rules:
- Reads are always allowed
- Creates default to draft or preview mode
- Publish/delete/overwrite require explicit commands or flags

JSON output:
- ok
- data
- next_cursor
- error.code
- error.message

Docs to include:
- install
- auth
- examples
- common failure modes
- safe usage notes for agents
```

## Concrete CLIs To Build Next

If we use this repo to build your actual tool wrappers, the first pass should focus on tools you touch often and repeatedly ask Codex about.

Good likely candidates:

- Internal notes or docs search
- Export readers for logs or analytics
- Slack or chat history research
- Scheduling or publishing workflows with safe draft defaults
- Past-agent-session search and retrieval

For each candidate, define:

- The source system
- The 3-7 high-value commands
- Which commands are read-only
- Which commands mutate state
- The minimum JSON schema needed for chaining

## Practical Rule

If you keep explaining the same thing to Codex, stop explaining it and give it a command.
