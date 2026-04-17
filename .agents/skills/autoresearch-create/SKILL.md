---
name: autoresearch-create
description: Set up and run an autonomous experiment loop for any optimization target. Gathers what to optimize, then starts the loop immediately. Use when asked to "run autoresearch", "optimize X in a loop", "set up autoresearch for X", or "start experiments".
---

# Autoresearch

Autonomous experiment loop: try ideas, keep what works, discard what doesn't, never stop.

## Tools

Codex does not have pi's `init_experiment`, `run_experiment`, or `log_experiment` extension tools, so use the Codex harness equivalents directly:

- **git + branch discipline** — isolate the autoresearch session and keep/revert ideas cleanly.
- **`autoresearch.sh`** — runs the workload, times it, captures structured metrics.
- **`autoresearch.md`** — session memory and operating contract for a fresh agent.
- **`autoresearch.jsonl`** — append-only experiment log you maintain from the harness.
- **`autoresearch.ideas.md`** — backlog for promising ideas not yet explored.

## Setup

1. Ask (or infer): **Goal**, **Command**, **Metric** (+ direction), **Files in scope**, **Constraints**.
2. `git checkout -b autoresearch/<goal>-<date>`
3. Read the source files. Understand the workload deeply before writing anything.
4. Write `autoresearch.md` and `autoresearch.sh` (see below). Commit both.
5. Run the baseline via `./autoresearch.sh`, record the result in `autoresearch.jsonl`, then start looping immediately.

### `autoresearch.md`

This is the heart of the session. A fresh agent with no context should be able to read this file and run the loop effectively. Invest time making it excellent.

```markdown
# Autoresearch: <goal>

## Objective
<Specific description of what we're optimizing and the workload.>

## Metrics
- **Primary**: <name> (<unit>, lower/higher is better) — the optimization target
- **Secondary**: <name>, <name>, ... — independent tradeoff monitors

## How to Run
`./autoresearch.sh` — outputs `METRIC name=number` lines.

## Files in Scope
<Every file the agent may modify, with a brief note on what it does.>

## Off Limits
<What must NOT be touched.>

## Constraints
<Hard rules: tests must pass, no new deps, etc.>

## What's Been Tried
<Update this section as experiments accumulate. Note key wins, dead ends,
and architectural insights so the agent doesn't repeat failed approaches.>
```

Update `autoresearch.md` periodically — especially the "What's Been Tried" section — so resuming agents have full context.

### `autoresearch.sh`

Bash script (`set -euo pipefail`) that: pre-checks fast (syntax errors in <1s), runs the benchmark, and outputs structured lines to stdout. Keep the script fast — every second is multiplied by hundreds of runs.

**For fast, noisy benchmarks** (< 5s), run the workload multiple times inside the script and report the median. This produces stable data points and makes the confidence score reliable from the start. Slow workloads (ML training, large builds) don't need this — single runs are fine.

#### Structured output

- `METRIC name=value` — primary metric and any secondary metrics. Parse and log these into `autoresearch.jsonl` after each run.

#### Design the script to inform optimization

The script should output **whatever data helps you make better decisions in the next iteration.** Think about what you'll need to see after each run to know where to focus:

- Phase timings when the workload has distinct stages
- Error counts, failure categories, or test names when checks can fail in different ways
- Memory usage, cache hit rates, or other runtime diagnostics when relevant
- Anything domain-specific that would help localize regressions or identify bottlenecks

The script runs the same code every iteration — but you can **update it during the loop** if you discover you need more signal. Add instrumentation as you learn what matters.

#### Harness-supplied annotations in `autoresearch.jsonl`

Use `autoresearch.jsonl` to annotate each run with **whatever would help the next iteration make a better decision.** Free-form structured fields are fine — you decide what's worth recording. Don't repeat the full raw output; capture what you'd lose after a context reset.

**Annotate failures and crashes heavily.** Discarded and crashed runs are reverted — the code changes are gone. The only record that survives is the description and annotations in `autoresearch.jsonl`. If you don't capture what you tried and why it failed, future iterations will waste time re-discovering the same dead ends.

### `autoresearch.config.json` (optional)

JSON config file that lives in the working directory. Supported fields:

- **`maxIterations`** (number) — maximum experiments before auto-stopping.
- **`workingDir`** (string) — override the directory for all autoresearch operations: file I/O (`autoresearch.jsonl`, `autoresearch.md`, `autoresearch.sh`, `autoresearch.checks.sh`, `autoresearch.ideas.md`), command execution, and git operations. Supports absolute paths or relative paths (resolved against the current working directory). Fails if the directory doesn't exist.

```json
{
  "workingDir": "/path/to/project",
  "maxIterations": 50
}
```

### `autoresearch.checks.sh` (optional)

Bash script (`set -euo pipefail`) for backpressure/correctness checks: tests, types, lint, etc. **Only create this file when the user's constraints require correctness validation** (e.g., "tests must pass", "types must check").

When this file exists:
- Run it automatically after every **passing** benchmark.
- If checks fail, record that clearly in `autoresearch.jsonl` and treat the run as `checks_failed`.
- Its execution time does **NOT** affect the primary metric.
- You cannot keep a result when checks have failed.
- Give it a separate timeout appropriate to the repo.

When this file does **not** exist, everything behaves exactly as before — no changes to the loop.

**Keep output minimal.** Only the tail of checks output should be preserved in context on failure. Suppress verbose progress/success output and let only errors through. This keeps context lean and helps pinpoint what broke.

```bash
#!/bin/bash
set -euo pipefail
# Example: run tests and typecheck — suppress success output, only show errors
pnpm test --run --reporter=dot 2>&1 | tail -50
pnpm typecheck 2>&1 | grep -i error || true
```

## Loop Rules

**LOOP FOREVER.** Never ask "should I continue?" — the user expects autonomous work.

- **Primary metric is king.** Improved → keep. Worse/equal → discard. Secondary metrics rarely affect this.
- **Annotate every run in `autoresearch.jsonl`.** Record what you learned — not what you did. What would help the next iteration or a fresh agent resuming this session?
- **Watch confidence and noise.** If the workload is noisy, re-run enough times to know whether the improvement is real.
- **Simpler is better.** Removing code for equal perf = keep. Ugly complexity for tiny gain = probably discard.
- **Don't thrash.** Repeatedly reverting the same idea? Try something structurally different.
- **Crashes:** fix if trivial, otherwise log and move on. Don't over-invest.
- **Think longer when stuck.** Re-read source files, study profiling data, reason about what the CPU is actually doing. The best ideas come from deep understanding, not random variations.
- **Resuming:** if `autoresearch.md` exists, read it + git log, continue looping.

**NEVER STOP.** The user may be away for hours. Keep going until interrupted.

## Ideas Backlog

When you discover complex but promising optimizations that you won't pursue right now, **append them as bullets to `autoresearch.ideas.md`**. Don't let good ideas get lost.

On resume (context limit, crash), check `autoresearch.ideas.md` — prune stale/tried entries, experiment with the rest. When all paths are exhausted, delete the file and write a final summary.

## User Messages During Experiments

If the user sends a message while an experiment is running, finish the current benchmark + log cycle first, then incorporate their feedback in the next iteration. Don't abandon a running experiment.
