---
name: forge-cli-admin
description: Use the Forge `forge` CLI for local Forge permissions, config, and self-update management. See `forge-tools` first if the right Forge CLI is not obvious.
---

# Forge CLI Admin

This skill covers the top-level `forge` binary. If the task may belong to another Forge CLI, check `forge-tools` first and then use this skill when the request is clearly about local Forge management.

Use these commands first:

- `forge doctor --json`
- `forge permissions check --json`
- `forge permissions fix --json`
- `forge self update-check --json`
- `forge self update --json`
- `forge skills status --json`

Working rules:

- Prefer `--json` for agent reads because it is the deterministic, low-token contract.
- Prefer `forge doctor --json` when you need an environment snapshot before using other Forge CLIs.
- Prefer `permissions check` before `permissions fix`.
- Prefer `self update-check` before `self update`.
- `skills status` defaults to `mainline` targets; use `--scope all` when debugging development installs.
- Use `--force` on `self update-check` only when a fresh remote check is required.
- Use `--repo-path` only when the repo is not in the expected configured location.
- Non-user targets such as `path:/...` and `forge_repo` default to `development`; use `--target-role mainline` when they are part of the primary managed install set.
- Route out of this skill when the request is actually about OpenAI documentation (`openai-docs`) or GitHub issues, PRs, reviews, and CI. Prefer GitHub plugin skills and `gh` workflows over direct native GitHub Codex app usage when both are viable.

Safety:

- `permissions fix` changes local file modes.
- `self update` changes the local Forge checkout.
- Do not run modifying commands unless the user asked for them or clearly approved the local mutation.
