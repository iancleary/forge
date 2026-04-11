---
name: forge-cli-admin
description: Use the Forge `forge` CLI for local Forge permissions, config, and self-update management. See `forge-tools` first if the right Forge CLI is not obvious.
---

# Forge CLI Admin

This skill covers the top-level `forge` binary. If the task may belong to another Forge CLI, check `forge-tools` first and then use this skill when the request is clearly about local Forge management.

Use these commands first:

- `forge permissions check --json`
- `forge permissions fix --json`
- `forge self update-check --json`
- `forge self update --json`

Working rules:

- Prefer `permissions check` before `permissions fix`.
- Prefer `self update-check` before `self update`.
- Use `--force` on `self update-check` only when a fresh remote check is required.
- Use `--repo-path` only when the repo is not in the expected configured location.

Safety:

- `permissions fix` changes local file modes.
- `self update` changes the local Forge checkout.
- Do not run modifying commands unless the user asked for them or clearly approved the local mutation.
