# Mermaid CLI

This document defines the Forge-style `mermaid` binary.

## Goal

Provide a narrow, text-first wrapper around Mermaid rendering so agents can produce reproducible diagrams without depending on a browser editor or ad hoc `pnpm` shell shaping.

Output contract:

- human-readable text by default
- compact JSON envelope with `--json`
- no pretty-printed JSON on the agent path

## Upstream Sources

Primary sources for this surface:

- Mermaid docs: `https://mermaid.js.org/`
- Mermaid syntax reference: `https://mermaid.js.org/intro/syntax-reference.html`
- Mermaid CLI repository: `https://github.com/mermaid-js/mermaid-cli`

The Mermaid docs currently redirect CLI-specific readers to the `mermaid-cli` repository.

## Scope

Implemented in the first Forge version:

- `doctor`
- `tool install`
- `render`

Out of scope for now:

- a live browser editor
- automatic diagram-type inference from prose
- a bespoke diagram DSL compiler
- hidden npm or pnpm mutations outside explicit commands
- direct reliance on the `mermaid-cli` Node API, which upstream documents as not being covered by semver

## Requirements

The wrapper assumes:

- `node` is on `PATH`
- `pnpm` is on `PATH`

The preferred package-manager path is `pnpm`, not `npm` or `npx`.

## Tool Root

Default Forge-managed tool root:

```text
~/.config/forge/mermaid/tool
```

Override with:

- `FORGE_MERMAID_CONFIG_DIR`
- `--tool-root <path>`

The tool root contains a minimal `package.json` plus the locally installed Mermaid CLI package when `mermaid tool install` is used.

## Command Surface

Automatic commands from Clap:

```sh
mermaid --help
mermaid --version
```

Forge-specific commands:

```sh
mermaid --json doctor
mermaid tool install
mermaid render --input diagram.mmd --output diagram.svg
mermaid render --input diagram.mmd --output diagram.png --theme dark --background transparent
mermaid render --input diagram.mmd --output diagram.pdf --pdf-fit
```

## Command Notes

### `doctor`

```sh
mermaid --json doctor
```

Reports:

- resolved config dir
- resolved tool root
- whether `node` is available
- whether `pnpm` is available
- whether a local installed `mmdc` binary is present under the tool root
- the default package and pinned version used by the wrapper

### `tool install`

```sh
mermaid tool install
mermaid tool install --package-version 11.12.0
mermaid tool install --tool-root /tmp/mermaid-tool
```

Behavior:

- creates the tool root if needed
- writes a minimal `package.json` when missing
- installs `@mermaid-js/mermaid-cli` with `pnpm --dir <tool-root> add --save-exact --save-dev ...`
- keeps the package install explicit instead of hiding it behind `render`

### `render`

```sh
mermaid render --input diagram.mmd --output diagram.svg
mermaid render --input diagram.mmd --output diagram.png --theme dark --background transparent
mermaid render --input diagram.mmd --output diagram.svg --config-file mermaid.config.json
mermaid render --input - --output diagram.svg
```

Behavior:

- uses the installed tool root by default when `node_modules/.bin/mmdc` exists
- otherwise falls back to `pnpm dlx @mermaid-js/mermaid-cli@<version> mmdc`
- passes explicit render flags through to `mmdc`
- creates the output parent directory if needed

Supported flags in v1:

- `--input`
- `--output`
- `--config-file`
- `--css-file`
- `--puppeteer-config-file`
- `--background`
- `--theme`
- `--width`
- `--height`
- `--scale`
- `--pdf-fit`
- `--quiet`
- `--tool-mode auto|installed|dlx`
- `--package-version`

## Authoring Guidance

The stable contract should stay:

1. author Mermaid text in `.mmd` or Markdown fenced blocks
2. optionally generate Mermaid text from checked-in Node/TypeScript code
3. render through `mermaid render`

If a higher-level DSL is introduced later, it should compile to Mermaid text files rather than bypassing the Mermaid source layer. That keeps the authored diagram reviewable and makes render failures easier to debug.
