# Diagram Selection

Choose the smallest diagram family that answers the audience's next question.

## Fast chooser

Use `flowchart` when:

- you need process, routing, decision paths, or service/data movement

Use `sequenceDiagram` when:

- order of interaction over time matters more than topology

Use `architecture-beta`, `block-beta`, or C4 when:

- the audience needs a structural system view, not execution order

Use `erDiagram` when:

- the primary question is data shape and cardinality

Use `stateDiagram-v2` when:

- lifecycle and transition rules matter

Use `gantt`, `timeline`, or `kanban` when:

- the topic is delivery planning, sequencing, or work status

Use `journey`, `quadrantChart`, `sankey-beta`, `pie`, `radar-beta`, `xychart-beta`, or `treemap-beta` when:

- the topic is product, GTM, portfolio, or executive communication

Use `requirementDiagram` when:

- requirements, validation method, or dependency traceability is the point

Use `mindmap`, `tree`, `venn`, or `ishikawa` when:

- the job is idea structure, taxonomy, overlap, or root-cause framing

## Anti-patterns

Do not use a graph when a table answers faster.

Do not use `pie` when precise comparison across many categories matters. Use `xychart-beta`, `treemap-beta`, or `radar-beta` instead.

Do not use `sequenceDiagram` for infrastructure topology. Use `flowchart`, `architecture-beta`, `block-beta`, or C4.

Do not use `flowchart` for state machines with real transition semantics. Use `stateDiagram-v2`.

Do not introduce a DSL when plain Mermaid plus a small checked-in generator script would stay easier to review.

## Audience-first defaults

Executive / board:

- `quadrantChart`
- `gantt`
- `timeline`
- `sankey-beta`
- `treemap-beta`
- `radar-beta`

Product / design / GTM:

- `journey`
- `mindmap`
- `kanban`
- `xychart-beta`
- `quadrantChart`
- `requirementDiagram`

Engineering / platform / architecture:

- `flowchart`
- `sequenceDiagram`
- `stateDiagram-v2`
- `erDiagram`
- `architecture-beta`
- `block-beta`
- `packet-beta`

Cross-functional RFCs:

- one structural diagram
- one lifecycle or interaction diagram
- one delivery or requirement diagram if needed

Most RFCs get worse when they contain five diagram families where two would do.
