# Rendering

Official sources:

- Mermaid CLI docs page: `https://mermaid.js.org/config/mermaidCLI.html`
- Mermaid CLI repo: `https://github.com/mermaid-js/mermaid-cli`

The Mermaid docs now point CLI users to the `mermaid-cli` repository. Forge wraps that CLI so the repeatable contract stays local and text-first.

## Preferred contract

Use the Forge wrapper:

```sh
mermaid --json doctor
mermaid tool install
mermaid render --input diagram.mmd --output diagram.svg
```

Why:

- the agent-facing contract is stable
- the package-manager details stay narrow
- the source file remains explicit and reviewable

## Tool modes

The Forge wrapper has three modes:

- `auto`: use local installed tool root when present, otherwise `pnpm dlx`
- `installed`: require the local tool root install
- `dlx`: use `pnpm dlx @mermaid-js/mermaid-cli@<version> mmdc`

Prefer `installed` or `auto` for repeated local work. Use `dlx` when you need a one-off render and do not want to mutate the tool root first.

## Recommended render patterns

Default SVG:

```sh
mermaid render --input diagrams/context.mmd --output build/context.svg
```

Dark PNG with transparent background:

```sh
mermaid render \
  --input diagrams/sequence.mmd \
  --output build/sequence.png \
  --theme dark \
  --background transparent
```

PDF intended for printing:

```sh
mermaid render \
  --input diagrams/roadmap.mmd \
  --output build/roadmap.pdf \
  --pdf-fit
```

Config-driven render:

```sh
mermaid render \
  --input diagrams/architecture.mmd \
  --output build/architecture.svg \
  --config-file mermaid.config.json
```

## Config strategy

Use a checked-in config file when a repo needs consistent output.

Typical config concerns:

- theme
- look
- layout
- font family
- theme variables

Prefer config and frontmatter before custom CSS. Use CSS only when you need an SVG-only treatment that Mermaid theme variables cannot express cleanly.

## Markdown transforms

The upstream `mmdc` can also transform Markdown files containing Mermaid fences into Markdown with rendered image references. Use that only when the product really is a Markdown transformation flow. For normal diagram authoring, keep `.mmd` as the source layer.

## Node authoring boundary

The upstream repository notes that the `mermaid-cli` Node API is not covered by semver. For that reason:

- do not treat the Node API as the stable contract
- prefer generating Mermaid text in Node and rendering through `mermaid render`

That keeps the integration boundary stable even when upstream library APIs change.
