---
name: design-algorithm
description: Apply Forge's design sequence when shaping features, reviewing proposed command surfaces, deciding whether repeated shell work belongs in Forge, or challenging unclear requirements. Use for planning, scope reduction, spec review, and CLI contract design.
---

# Design Algorithm

Apply this sequence in order:

1. Question every requirement.
2. Delete any part or process you can.
3. Simplify and optimize what remains.
4. Accelerate cycle time.
5. Automate last.

Use it to shape work before adding CLI surface, workflow policy, or documentation.

## Working Rules

- Ask who owns each requirement and which concrete user or agent job it serves.
- Delete repeated shell shaping, duplicated policy, or unnecessary interface parts before improving them.
- Prefer the narrowest stable contract that makes the next decision easier.
- Improve loop speed only after the contract is necessary and clean.
- Automate only when the first four steps are satisfied.

## Forge Interpretation

In Forge, this usually means:

- do not add a command, flag, or crate just because a workflow exists
- remove noisy JSON fields agents repeatedly ignore
- avoid duplicating guidance across many skills when one shared rule can carry it
- turn recurring, stable manual shaping into a small primitive only after confirming it should exist

## Quick Checklist

Before finalizing a design, answer:

- What requirement did you question?
- What did you delete?
- What became simpler?
- What loop became faster?
- Why is automation justified now instead of earlier?
