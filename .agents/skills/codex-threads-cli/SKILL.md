---
name: codex-threads-cli
description: Use the Forge `codex-threads` CLI to sync, search, resolve, and read local Codex session archives. See `forge-tools` first if the correct Forge CLI is not obvious.
---

# Codex Threads CLI

This skill covers the Forge `codex-threads` binary. If the user may need another Forge CLI, check `forge-tools` first and come back here once local Codex session retrieval is the right fit.

If the task is about designing new Forge behavior rather than reading local session archives, route to `design-algorithm` first.

Start narrow:

- `codex-threads --json sync`
- `codex-threads --json messages search "<query>" --limit 5`
- `codex-threads --json threads resolve "<query>" --limit 5`
- `codex-threads --json threads read <session-id>`
- `codex-threads --json events read <session-id> --limit 20`

Working rules:

- Use `--json` for reads because the skill is consumed by agents and should stay deterministic and low-token.
- Run `sync` before search if the index may be stale or missing.
- Prefer `messages search` for finding a phrase and `threads resolve` for finding the likely session.
- Prefer `threads read` for normalized history.
- Only use `events read` when normalized thread output is insufficient or raw event inspection is needed.
- Keep limits small first because raw event output gets noisy quickly.

Safety:

- This CLI is local and read-focused.
- `sync` mutates only the local index; it is safe to run when needed.

Common flow:

1. Ensure the index exists with `sync`.
2. Search or resolve the target thread.
3. Read the normalized thread.
4. Fall back to `events read` only for debugging or deeper inspection.

## Inputs

- a query phrase, session id, or timeframe hint
- whether normalized thread output is sufficient vs raw events needed

## Output

- the narrowest `codex-threads --json ...` command(s) needed
- a short extraction of the specific messages/events relevant to the user’s question

## Checks

- keep `--limit` small first, especially for `events read`
