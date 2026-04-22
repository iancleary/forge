# Technical Diagrams

This reference is for system design, software architecture, platform work, data design, and incident analysis.

## `flowchart`

Use for:

- service topology
- request routing
- deployment paths
- business process with technical consequences

Why:

- it is the most flexible general-purpose graph form

Remember:

- `graph` is an alias
- quote labels if parser edge cases appear

## `sequenceDiagram`

Use for:

- API request/response order
- async workflows
- queue handoff timing
- auth/token exchange flows

Why:

- temporal order is explicit

## `classDiagram`

Use for:

- domain model communication
- library design
- inheritance / interface relationships

Avoid when:

- the real question is data storage shape or runtime sequence

## `stateDiagram-v2`

Use for:

- entity lifecycle
- workflow states
- orchestrator transitions
- feature flag rollout logic

Why:

- transition semantics matter more than topology

## `erDiagram`

Use for:

- relational schema design
- cardinality review
- business entity modeling

Why:

- relationship types are first-class

## `architecture-beta`

Use for:

- high-level system context
- cloud/service component communication
- infrastructure views for non-specialists

Why:

- it gives a cleaner structure-first picture than a generic flowchart

## `block-beta`

Use for:

- subsystem decomposition
- hardware/software functional blocks
- clear high-level partitioning

Why:

- it is simple and presentation-friendly

## C4

Use for:

- context, container, component, or code views when a team already thinks in C4

Caution:

- Mermaid marks C4 as warning-level in docs; verify render output before treating it as the stable default

## `packet-beta`

Use for:

- bit or field layout of packets or protocol structures

Why:

- it is far better than faking packet structure in generic boxes

## `gitGraph`

Use for:

- release branching
- merge strategy explanation
- incident or hotfix timeline across branches

## `tree`

Use for:

- config hierarchy
- filesystem-like structure
- menu or route nesting

## `mindmap`

Use for:

- architecture brainstorming before formalizing a design

## `zenuml`

Use for:

- teams that specifically prefer ZenUML sequence syntax

Use sparingly unless the surrounding docs already speak that dialect.

## `requirementDiagram`

Use for:

- technical requirements traceability
- verification and risk metadata

## Recommended technical bundle

For most engineering docs, the highest-value set is:

- `flowchart`
- `sequenceDiagram`
- `stateDiagram-v2`
- `erDiagram`
- `architecture-beta`
- `block-beta`
- `packet-beta`
- `requirementDiagram`
