# Algorithm

This document adapts Elon Musk's five-step "algorithm" into a Forge design and implementation workflow.

The point is not hero worship. The point is sequencing.

Do these steps in order:

1. Question every requirement.
2. Delete any part or process you can.
3. Simplify and optimize what remains.
4. Accelerate cycle time.
5. Automate last.

If you reverse the order, you usually automate noise.

## 1. Question Every Requirement

Before adding a command, flag, crate, doc section, or skill rule:

- ask who actually needs it
- identify the single owner of the requirement
- ask whether the requirement is inherited from habit rather than current need
- prefer direct user and agent jobs over abstract completeness

For Forge, that usually means asking:

- what repeated agent task are we trying to remove?
- which next decision should the command make easier?
- does this belong in `docs/`, a skill, or a crate contract?
- is the requirement coming from the user, the agent, or imagined future scope?

## 2. Delete Parts Or Process

Delete before you optimize.

Examples in Forge:

- remove a shell shaping step instead of documenting it forever
- avoid adding a new top-level command when an existing noun/action fits
- cut noisy JSON fields that agents drop every time
- remove duplicated guidance from multiple skills when a shared doc can carry it

Use holistic thinking here:

- have both sides of the interface been considered?
- is the part required by both producer and consumer?
- can the whole interaction be changed so the part disappears?

If a proposed addition cannot survive a deletion pass, it is probably not load-bearing.

## 3. Simplify And Optimize

Only optimize the parts that remain necessary.

In Forge, simplification usually means:

- narrow command contracts
- stable, compact JSON
- explicit verbs
- fewer flags with clearer meanings
- one obvious read path before adding alternate views

Optimize for:

- low token usage
- deterministic outputs
- safe defaults
- minimal follow-up shell shaping

## 4. Accelerate Cycle Time

Once the design is necessary and simple, make the loop faster.

Examples:

- add a narrow subcommand instead of requiring repeated `jq`
- improve help text so the next agent run finds the right command immediately
- add tests around the compact output shape agents rely on
- tighten docs so the spec and implementation stay aligned

Cycle-time improvements should reduce user and agent backtracking, not just execution time.

## 5. Automate Last

Automation is valuable only after the earlier steps are done.

Good automation in Forge:

- a CLI primitive that removes a repeated manual shaping step
- a doctor check that codifies repeated environment debugging
- a skill rule that routes agents to the right binary first

Bad automation in Forge:

- wrapping an unclear workflow in more commands
- encoding duplicated or speculative policy into many places
- adding mutation flows before the safe read path is stable

## Practical Review Checklist

Before landing a Forge change, ask:

- What requirement did we question, and who owns it?
- What did we delete instead of preserving?
- What became simpler for the user or agent?
- What loop became faster?
- Did we automate only after the earlier steps were satisfied?

If those answers are weak, keep deleting and simplifying.
