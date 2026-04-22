# Examples

Official sources used for this reference:

- Mermaid examples page: `https://mermaid.js.org/syntax/examples.html`
- Mermaid sequence diagram docs: `https://mermaid.js.org/syntax/sequenceDiagram.html`
- Mermaid README examples: `https://github.com/mermaid-js/mermaid#examples`

Use this reference when the user wants a concrete starting point rather than a diagram-family recommendation.

The checked-in `.mmd` files live under `examples/`.

## Sequence diagrams

These get the heaviest coverage because they are the most common engineering ask and the easiest place to make syntax mistakes.

- `examples/sequence-basic.mmd`
  README-style greeting flow with `loop` and `Note`.
- `examples/sequence-loops-alt-opt.mmd`
  Control-flow example using `loop`, `alt`, and `opt`.
- `examples/sequence-self-loop.mmd`
  Request flow with self-messages and polling.
- `examples/sequence-blogging-app.mmd`
  Service-to-service example for a realistic web app path.
- `examples/sequence-par-critical-break.mmd`
  Parallel work, critical region handling, and exception stop.
- `examples/sequence-boxes-activation.mmd`
  Grouped actors, async calls, notes, and activation shortcuts.

## Core README-style examples

These mirror the main families highlighted in Mermaid's public README and examples page.

- `examples/flowchart-basic.mmd`
- `examples/flowchart-styled.mmd`
- `examples/gantt-basic.mmd`
- `examples/class-basic.mmd`
- `examples/state-basic.mmd`
- `examples/pie-basic.mmd`
- `examples/gitgraph-basic.mmd`
- `examples/journey-basic.mmd`
- `examples/c4-context-basic.mmd`

## Usage pattern

When authoring a diagram for a user:

1. start from the closest file in `examples/`
2. rename participants, nodes, or sections to match the problem
3. keep the syntax family the same unless the audience question changed
4. render before finalizing when you used advanced sequence control flow

## Version-sensitive notes

The examples in this directory stay biased toward documented syntax that is stable in the current Mermaid docs surface.

Sequence diagrams have the fastest-moving syntax. Be careful with:

- new arrow variants
- actor creation and destruction rules
- central lifeline connections

If you need those features, verify against the current sequence docs before finalizing.
