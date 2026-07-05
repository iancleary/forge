---
name: forge-tools
description: Route Codex to the right Forge CLI or workflow skill for Linear, Slack retrieval or assistant actions, Codex session retrieval, loop design, or local Forge management. Use when a task may involve more than one Forge skill or when the correct narrow skill is not obvious yet.
---

# Forge Tools

Use this skill as the lightweight entry point for the Forge CLI bundle.

Pick the narrowest CLI skill that matches the job:

- `bytefield-diagrams`: author and render bytefield, packet-layout, and memory-layout SVG diagrams through `forge bytefield`.
- `linear-cli`: Linear issue, project, milestone, and viewer workflows.
- `mermaid-diagrams`: text-first Mermaid authoring, diagram-type selection, and rendering through the Forge `mermaid` CLI.
- `typst-documents`: source-first Typst PDF authoring, reusable document templates, and compile/build verification.
- `slack-query-cli`: Slack permalink resolution, search, thread reads, and nearby message context.
- `slack-agent-cli`: assistant-oriented Slack thread replies, reactions, file uploads, DMs, and channel joins.
- `codex-threads-cli`: local Codex session sync, search, thread resolution, and event inspection.
- `forge-cli`: local Forge config, permission checks, self-update commands, global tool updates, and managed-surface deployment.
- `source-driven-development`: verify framework, library, API, CLI, or vendor behavior from primary sources before implementation.
- `debugging-and-error-recovery`: reproduce, localize, reduce, fix, and guard failing tests, broken builds, command errors, or unexpected behavior.
- `api-and-interface-design`: design or review CLI/API/module/JSON contracts before changing a public or durable interface.
- `security-and-hardening`: review concrete trust-boundary risk around input, auth, secrets, filesystem, shell, network, permissions, or persisted state.
- `test-strategy`: choose focused verification for features, bug fixes, refactors, and regressions without forcing ceremony.
- `code-simplification`: simplify working code while preserving behavior and keeping proof intact.
- `documentation-and-adrs`: update durable docs, workflow policy, command contracts, and ADR-style decisions.
- `autoresearch-create`: start an autonomous experiment loop for measurable optimization work.
- `autoresearch-finalize`: turn an autoresearch branch into clean, reviewable branches.
- `effective-loop-writer`: interview the human and scaffold a restartable file-backed loop under `loops/`.
- `autoreview`: run structured Codex-first or optional Claude code review as a closeout check before commit, PR update, or ship.
- `create-release-process`: create, audit, or update a repo-local release workflow that future releases can execute deterministically.
- `cut-release`: execute an existing repo-local release workflow by running the checked-in release runner.
- `chrome-devtools-mcp`: Codex MCP setup and live Chrome debugging with console/network/DOM/trace inspection.
- `design-algorithm`: shaping and scope-reduction workflow for deciding what should exist before building or automating it.

Default operating rules:

- Use `design-algorithm` when the user is shaping a feature, debating command surface, or deciding whether recurring shell work should become a Forge primitive.
- Use `api-and-interface-design` after the need survives `design-algorithm` and the remaining question is the stable contract.
- Use `source-driven-development` before relying on version-sensitive framework, API, CLI, or vendor behavior.
- Use `debugging-and-error-recovery` when a command or test fails and the next step should be root-cause analysis.
- Use `test-strategy` when the main uncertainty is what proof the change needs.
- Use `security-and-hardening` when a concrete trust boundary is in scope.
- Use `code-simplification` only after behavior is known and proof is available or cheap.
- Use `documentation-and-adrs` when the change updates durable workflow, command, or architecture knowledge.
- Use `effective-loop-writer` when the user wants to design or scaffold an unattended loop, generator-evaluator workflow, overnight agent procedure, or reusable loop artifact.
- Use `create-release-process` for release workflow maintenance and `cut-release` for ordinary release execution after the workflow exists.
- Prefer the crate-specific skill once the target CLI is clear.
- Prefer `--json` for all reads because it is the deterministic, low-token contract agents consume directly.
- Fetch a small amount of data first with the tool's `--limit` or narrowest read command.
- For bytefield diagram work, keep authoring in the upstream DSL and execution in `forge bytefield` rather than teaching every session a new shell wrapper.
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
