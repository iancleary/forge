---
name: mermaid-diagrams
description: Use the Forge `mermaid` CLI plus Mermaid syntax to create reproducible business and technical diagrams, choose the right diagram type, and author diagrams as plain Mermaid, generated Node code, or a thin DSL that compiles to Mermaid. Prefer this over WYSIWYG diagram editors when text-first artifacts are acceptable.
---

# Mermaid Diagrams

Use this skill when the job is to author, revise, select, or render Mermaid diagrams.

This skill assumes the local contract is the Forge `mermaid` CLI, not raw `pnpm` shell shaping.

Start here:

- `mermaid --json doctor`
- `mermaid render --input diagram.mmd --output diagram.svg`

Use plain Mermaid first. Move to generated Node code only when many diagrams share the same schema. Introduce a DSL only when the domain vocabulary is stable and repeated enough that plain Mermaid or Node string templates stay noisy.

Load only the reference file that matches the task:

- syntax, frontmatter, themes, layout, accessibility, and diagram breakers:
  `references/foundations.md`
- Forge render/install flow and upstream CLI notes:
  `references/rendering.md`
- choosing the right diagram family for the job:
  `references/diagram-selection.md`
- startup, product, GTM, and executive communication diagrams:
  `references/business-diagrams.md`
- architecture, software, data, and systems diagrams:
  `references/technical-diagrams.md`
- plain Mermaid vs generated Node code vs a thin DSL:
  `references/authoring-models.md`

Working rules:

- Keep the Mermaid source file checked in. The rendered SVG/PNG/PDF is a build artifact, not the canonical source.
- Prefer reviewable `.mmd` or Markdown fenced Mermaid blocks over opaque binary slides.
- Use Mermaid frontmatter for theme, look, and layout before reaching for custom CSS.
- Add accessible title and description when the diagram will be published or reused.
- When the user asks for a new diagram language, use `design-algorithm` thinking first and keep the output compiling down to Mermaid text.

Do not use this skill when:

- the user needs a pixel-perfect branded illustration better handled in design tools
- the task is a live browser debugging job; use `chrome-devtools-mcp`
- a static table, short list, or prose explanation would communicate better than a diagram

## Inputs

- problem to communicate
- audience
- output format (`.mmd`, `.svg`, `.png`, `.pdf`, or Markdown)

## Output

- chosen Mermaid diagram type
- checked-in Mermaid source
- `mermaid ...` render command when an artifact is needed

## Checks

- prefer the smallest diagram that answers the question
- keep source text reviewable
- render before finalizing when syntax or layout risk is non-trivial
