---
name: bytefield-diagrams
description: Use when a user wants bytefield, packet-layout, register-layout, or memory-layout diagrams in SVG. Prefer the upstream `bytefield-svg` DSL and run it through `forge bytefield` instead of invoking `pnpm`, `pnpx`, or `bytefield-svg` directly.
---

# Bytefield Diagrams

This skill covers authoring and rendering bytefield diagrams with `bytefield-svg` through Forge's wrapper contract.

Use these commands first:

- `forge bytefield install --json`
- `forge bytefield render --source <file> --output <file> --json`
- `forge bytefield render --source <file> --output <file> --embedded --json`

Working rules:

- Prefer the native `bytefield-svg` DSL. Do not invent a second Python layer or bespoke DSL unless the user explicitly asks to build a translator.
- Treat `forge bytefield ...` as the execution contract. Do not call `pnpm`, `pnpx`, or `bytefield-svg` directly unless you are debugging the Forge wrapper itself.
- Keep source in a checked-in file, usually something like `docs/diagrams/<name>.bf.clj` or another repo-local path that fits the project.
- Default output is SVG. Use `--embedded` when the result will be dropped into HTML or docs tooling that wants only the `<svg>` element.
- Treat consumer layout sizing as a downstream concern. Labels like `normal`, `wide`, or Typst-specific width choices belong in a future web/Typst wrapper, not in the raw bytefield DSL itself.
- Start from the protocol or memory layout, choose `boxes-per-row`, then draw left-to-right and top-to-bottom.
- Use helper defs only after the layout is already clear; do not hide the structure too early.
- If a gap ends the diagram or is followed by only a partial row, call `draw-bottom`.

Planned follow-up:

- a Typst-facing wrapper is expected later to map rendered SVG and metadata into Typst-friendly width and placement controls
- that Typst wrapper is planned work, but not part of the current `forge bytefield` execution contract

## Upstream Model

`bytefield-svg` uses a Clojure-based DSL. The main mental model is:

1. Set optional globals with `def`.
2. Draw headers with `draw-column-headers`.
3. Draw fields with `draw-box`, `draw-related-boxes`, `draw-gap`, and related helpers.
4. Adjust styling with attribute expressions and `defattrs`.

Common predefined values:

- `left-margin`
- `right-margin`
- `bottom-margin`
- `box-width`
- `boxes-per-row`
- `column-labels`
- `row-height`
- `svg-attrs`
- `row-header-fn`

Common attribute-expression forms:

- raw map: `{:span 4}`
- named attribute keyword: `:plain`
- merged vector: `[:plain {:font-weight "bold"}]`

Useful predefined attributes:

- `:hex`
- `:plain`
- `:math`
- `:bold`
- `:dotted`
- `:box-first`
- `:box-related`
- `:box-last`
- `:box-above`
- `:box-below`

Core functions to know:

- `draw-column-headers`
- `draw-box`
- `draw-boxes`
- `draw-related-boxes`
- `draw-gap`
- `draw-gap-inline`
- `draw-bottom`
- `draw-padding`
- `defattrs`
- `text`
- `number-as-hex`
- `number-as-bits`
- `char->int`

## Common Patterns

Single-row or compact structure:

```clojure
(def left-margin 1)
(def boxes-per-row 8)
(draw-column-headers)
(draw-box "Type" {:span 1})
(draw-box "Len" {:span 1})
(draw-box "Value" {:span 6})
```

Protocol header with a variable-length payload:

```clojure
(draw-column-headers)
(draw-box "Address" {:span 4})
(draw-box "Size" {:span 2})
(draw-box 0 {:span 2})
(draw-gap "Payload")
(draw-bottom)
```

32-bit rows with two-digit column labels:

```clojure
(def column-labels (mapv #(number-as-hex % 2) (range 32)))
(def boxes-per-row 32)
(draw-column-headers)
```

Vertical labels:

```clojure
(defattrs :vertical [:plain {:writing-mode "vertical-rl"}])
(draw-box (text "ACK" :vertical))
```

Open box spanning multiple rows:

```clojure
(draw-box "Source address" [{:span 16} :box-above])
(draw-box nil [{:span 16} :box-below])
```

## Suggested Workflow

1. Sketch the fields in plain English first.
2. Choose row width and column labels.
3. Write the minimal `draw-box` sequence.
4. Add styling only where the layout needs it.
5. Render with `forge bytefield render`.
6. Inspect the SVG and iterate.

## Inputs

- protocol or memory layout
- desired row width
- output file path

## Output

- bytefield source file + `forge bytefield render ...` command

## Checks

- use the upstream DSL directly
- call `draw-bottom` after terminal gaps
- keep execution on the Forge wrapper
- prefer checked-in source files over inline shell snippets

References:

- https://github.com/Deep-Symmetry/bytefield-svg
- https://bytefield-svg.deepsymmetry.org/bytefield-svg/1.11.0/intro.html
- https://bytefield-svg.deepsymmetry.org/bytefield-svg/1.11.0/funcs.html
- https://bytefield-svg.deepsymmetry.org/bytefield-svg/1.11.0/attrs.html
- https://bytefield-svg.deepsymmetry.org/bytefield-svg/1.11.0/examples.html
