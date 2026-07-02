---
name: effective-loop-writer
description: Create file-backed agent loop specs through an interactive interview. Use when the user asks to design, write, scaffold, or improve an unattended restartable loop, generator-evaluator workflow, autonomous coding run, overnight agent procedure, or loop artifact under loops/. Produces loop.md, contract.md, evaluator-rubric.md, durable state files, and trace policy; do not use to execute a finished loop unless the user asks to revise its design.
---

# Effective Loop Writer

Use this skill to turn a fuzzy automation idea into a small, restartable, file-backed loop that future agents can run, inspect, and improve.

Do not begin by asking the human for a perfect prompt. Interview for the missing loop pieces, write the loop artifact, and leave unknowns as explicit TODOs when they are not blocking.

## Operating Rules

- Keep the loop small enough to express as `gather, reason, act, verify, repeat`.
- Separate planner, generator, and evaluator contexts. The generator must not grade its own work.
- Make the evaluator assume the output is broken until the contract is proven.
- Negotiate `contract.md` before implementation work begins.
- Write durable state to disk so the loop can resume after session loss.
- Allow restart and deletion of bad work when the contract is still valid.
- Insert a human when the contract is wrong, not merely when the build is broken.
- Make subjective quality explicit with a weighted rubric and examples when possible.
- Save raw agent transcripts under `traces/` so failures can be debugged from evidence.
- Name the current bottleneck the loop is meant to expose.

## Tooling Rules

- If the loop workflow needs Python, use `uv` rather than relying on ambient Python environments.
- If the loop workflow needs YAML or PyYAML, run the needed tool through `uvx` so the dependency is explicit. For example, use `uvx --with pyyaml python <script> ...` for a Python validator that imports `yaml`, or `uvx --from <package> <command> ...` when a package provides a CLI.
- Record any `uv` or `uvx` command required for execution or validation in `loop.md` and any dependency assumptions in `contract.md`.

## Interview Flow

Ask only the questions needed to produce useful files. If the user already supplied enough context, draft the files and mark uncertain values as TODO.

1. Repeatable work
   - What work should run unattended?
   - Can it be stated as short repeated verbs: gather, reason, act, verify, repeat?
   - What should the loop refuse to do?
2. Role split
   - What does the planner own?
   - What does the generator own?
   - What does the evaluator own?
   - Which context or files should each role see?
3. Contract
   - What assertions can be tested before the evaluator accepts the work?
   - Which requirements come from the planner spec and must not drift?
   - What does the generator need the evaluator to agree to before coding starts?
4. State
   - Which files let the loop resume?
   - What is append-only?
   - Which files may be regenerated or deleted?
5. Restart policy
   - When may the loop delete bad work and start over?
   - When must it stop for a human?
6. Quality rubric
   - What objective checks are mandatory?
   - What subjective axes matter, and what weights should they carry?
   - Are there examples of good and bad output to calibrate against?
7. Traces
   - Where will raw transcripts be saved?
   - What divergence should a human look for when debugging the loop?
8. Harness deletion
   - Which scaffolding exists only because current models need it?
   - When should it be removed or simplified?
9. Bottleneck
   - What bottleneck should this loop reveal next: planning, verification, taste, source capture, review, or something else?

## Artifact Contract

Create or update this directory shape unless the project already has a stronger loop convention:

```text
loops/<loop-name>/
  loop.md
  contract.md
  evaluator-rubric.md
  state/
    feature_list.json
    progress.md
    log.md
  traces/
```

Use a lowercase hyphen-case `<loop-name>`. If the user gives no name, derive one from the repeatable work.

Because Git does not track empty directories, add `traces/.gitkeep` when the loop artifact should be committed before the first trace exists.

## File Templates

Use these section contracts, adapting labels only when the project already has a local convention.

### `loop.md`

Include:

- Purpose: one paragraph describing the repeatable work.
- Non-goals: work the loop must not absorb.
- Procedure: the short `gather, reason, act, verify, repeat` cycle.
- Roles: planner, generator, evaluator, with context boundaries.
- Contract negotiation: how `contract.md` is proposed, challenged, and accepted before implementation.
- State files: each file, owner, mutability, and resume rule.
- Restart policy: allowed restarts, deletion policy, and human-intervention triggers.
- Trace policy: where transcripts are written and how they are named.
- Harness deletion criteria: what to revisit as models improve.
- Current bottleneck: the bottleneck this loop is designed to expose.
- Runbook: the exact first unattended iteration.

### `contract.md`

Include:

- Planner boundary: what the planner asked for and what is out of scope.
- Preconditions: files, credentials, tools, or fixtures required before a run.
- Testable assertions: objective statements the evaluator can prove or falsify.
- Negotiation notes: generator/evaluator disagreements and final decisions.
- Acceptance rule: the threshold for proceeding, retrying, restarting, or stopping.

Use checkboxes for assertions when that makes evaluation easier.

### `evaluator-rubric.md`

Include:

- Evaluator stance: assume broken, inspect evidence, and never accept self-graded generator output.
- Objective checks: tests, diffs, screenshots, command output, lint, or review steps.
- Subjective scoring: weighted axes such as design, originality, craft, functionality, correctness, maintainability, or evidence quality.
- Calibration examples: known-good and known-bad examples when available.
- Scoring threshold: what score passes and what score triggers retry or restart.
- Report format: evidence-backed finding, score, decision, and next bottleneck.

If the loop has no subjective quality surface, say that explicitly and use an objective-only rubric.

### `state/feature_list.json`

Start with a tiny durable schema:

```json
{
  "version": 1,
  "items": []
}
```

Rename `feature_list.json` only when another state noun fits the loop better.

### `state/progress.md`

Track the active iteration, last completed step, current blocker, and resume instructions.

### `state/log.md`

Make this append-only. Use entries shaped like:

```markdown
## [YYYY-MM-DD] op | title

- role:
- action:
- evidence:
- next:
```

## Output Contract

When using this skill:

1. Create or patch the loop files when filesystem access is available.
2. Report the created paths and any TODOs that still need human judgment.
3. Explain the first invocation command or prompt a future agent should use.
4. Do not claim the loop is automated until it has been run and evaluated at least once.
