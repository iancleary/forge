# Agent Operating Loop

Forge should make an agent better at the whole job, not just faster at one command.

The core product is an operating loop:

```text
observe -> orient -> choose -> act -> verify -> accrete
```

Every Forge surface should improve one step in that loop without making another step less legible.

## System Goal

An agent using Forge should be able to:

- understand the current situation with the smallest reliable read
- find the right skill or command without guessing
- choose a safe next action from explicit state
- execute through a narrow deterministic contract
- verify the result without reconstructing the workflow
- leave behind a better reusable surface when repeated friction appears

This is the difference between a toolbox and a system. A toolbox gives the agent parts. Forge should give the agent a linked set of abstractions where each layer explains when to use the next one down.

## Four Planes

Forge-managed behavior belongs to one of four planes.

### 1. Context Plane

The context plane helps the agent know what is true now.

Examples:

- `forge doctor --json`
- `forge self update-check --json`
- `forge skills status --json`
- `forge codex diff --json`
- `codex-threads --json ...`
- read-only Slack, Linear, and repository query commands

Context commands should be cheap, bounded, and safe. They should return enough state to support the next decision, not every field the source system can provide.

### 2. Routing Plane

The routing plane helps the agent choose the right abstraction.

Examples:

- skill frontmatter descriptions
- router skills such as `forge-tools`
- repo `AGENTS.md`
- command help and examples
- docs that name neighboring boundaries

Routing surfaces should answer: "Use this when, prefer it over what, and stop here when what is true?"

### 3. Execution Plane

The execution plane performs the chosen action.

Examples:

- `forge codex install`
- `forge skills install`
- `forge release run --dry-run|--apply`
- `linear issue create`
- `slack-agent reply send`
- skill-backed artifact commands such as `forge bytefield render`

Execution commands should be explicit about mutation, actor, target, preview behavior, and rollback or recovery when applicable. Reads and writes should not be hidden behind the same ambiguous command path.

### 4. Accretion Plane

The accretion plane turns repeated friction into durable improvement.

Examples:

- managed skills
- command contracts
- docs and ADR-style decisions
- test fixtures around JSON shapes
- release examples and templates
- session search that makes past work reusable

Accretion should not mean saving everything. It means preserving the smallest useful abstraction that makes the next agent run more accurate or cheaper.

## Contract Stack

Forge should maintain a tower of linked contracts:

```text
User job
  -> skill/router trigger
  -> command or workflow spec
  -> compact JSON shape
  -> verification path
  -> durable lesson or deletion decision
```

Each layer should name the next layer it expects the agent to use. A command spec that does not say how an agent should verify it is incomplete. A skill that does not say when to stop or hand off is incomplete. A workflow plan that cannot produce a smaller future contract is probably just temporary coordination, not Forge product surface.

## Agent-Ergonomic Output

Agent-facing JSON should usually include:

- stable identifiers for follow-up commands
- compact status fields that summarize state
- bounded result sets with limits or cursors
- actionable error codes
- preview or dry-run results before mutation
- enough provenance to know which source was read

Avoid:

- raw provider payloads by default
- fields that exist only because the upstream API returned them
- prose that must be parsed for control flow
- unbounded history dumps
- hidden writes during read or status commands

The best output is the smallest truthful state that makes the next action obvious.

## Accretion Rule

When an agent encounters repeated friction, apply this sequence:

1. Name the repeated decision or transformation.
2. Delete any local shell shaping that should not be permanent.
3. Decide whether the remaining job belongs in a skill, doc, test, or CLI.
4. Add the smallest contract that removes the repeated error source.
5. Verify that the next agent can discover and use it from the normal routing path.

Do not accrete noise. Session logs, raw exports, local prompts, and one-off scripts only become Forge material when they make a durable future decision easier.

## Design Review Questions

Before adding or changing a Forge surface, answer:

- Which loop step does this improve?
- Which plane owns it?
- What existing surface can be deleted or simplified because this exists?
- What is the smallest read path before any write path?
- What command, skill, or doc should the agent discover next?
- What verification proves the action worked?
- What lesson should become durable if this friction repeats?

If those answers are weak, do not add more surface yet. Improve the existing loop instead.
