# Linear CLI

This document defines the Forge-style `linear`.

## Goal

Provide a narrow, JSON-first Linear wrapper in Rust that gives Codex stable access to the Linear GraphQL API for common issue, project, and milestone workflows.

This is not a full port of `schpet/linear-cli`. It borrows the useful command shape from the upstream CLI, but trims the scope down to explicit agent-friendly primitives.

References:

- Upstream README: `https://github.com/schpet/linear-cli`
- Linear GraphQL docs: `https://linear.app/developers/graphql`
- Linear API key settings: `https://linear.app/settings/account/security`

## Relationship To Upstream

The upstream `linear` CLI includes VCS-aware issue workflows, interactive prompts, app/browser launching, project-local config, and a broader command surface.

Forge's `linear` intentionally keeps a smaller contract:

- JSON-first output
- explicit read and write verbs
- no hidden Git or `gh` side effects
- no browser/app launching
- no interactive prompts in v1

That means some upstream commands are mirrored directly, some are adapted, and some remain out of scope.

## Scope

Implemented in the first Forge version:

- `viewer`
- `auth login`
- `config`
- `completions`
- `team list`
- `project list`
- `project view`
- `issue list`
- `issue read`
- `issue create`
- `issue update`
- `milestone list`
- `milestone view`
- `milestone create`
- `milestone update`
- `milestone delete`
- `teams ...` alias for `team ...`
- `issues ...` alias for `issue ...`
- `m ...` alias for `milestone ...`

Out of scope for now:

- project-local repo config such as `./linear.toml`
- git and `jj` issue detection
- branch creation or issue-start workflows
- PR creation
- comments, documents, and team member management
- app/browser launching

## Auth And Setup

Upstream setup starts with:

1. Create an API key at `https://linear.app/settings/account/security`
2. Authenticate the upstream CLI with `linear auth login`
3. Configure the project with `linear config`

Forge adapts that flow for a local, non-interactive CLI:

1. Create a personal Linear API key at `https://linear.app/settings/account/security`
2. Initialize the Forge config directory:

```sh
linear config
```

3. Save the API key with:

```sh
linear auth login
```

4. Or place the raw API key in:

```text
~/.config/forge/linear/token
```

5. Optionally edit:

```text
~/.config/forge/linear/config.toml
```

Example config:

```toml
token_file = "~/.config/forge/linear/token"
team_id = "YOUR_TEAM_ID"
```

Supported lookup order:

1. `LINEAR_API_KEY`
2. `~/.config/forge/linear/config.toml`
3. `~/.config/forge/linear/token`

Optional override:

- `FORGE_LINEAR_CLI_CONFIG_DIR`

`LINEAR_API_KEY` is useful for ad hoc use and CI. The config dir is the preferred local setup.

## Upstream Workflow Notes

The upstream CLI works with both Git and `jj`:

- Git works best when your branch names include a Linear issue ID such as `eng-123-my-feature`
- `jj` workflows can use `Linear-issue` trailers, for example:

```sh
jj describe "$(linear issue describe ABC-123)"
```

Forge does not implement those VCS-aware helpers in `linear` v1. If those workflows matter later, they should be added as explicit repo-aware commands rather than hidden side effects on read/write API calls.

## Command Surface

Automatic commands from Clap:

```sh
linear --help
linear --version
```

Forge-specific setup commands:

```sh
linear auth login
linear config
linear completions zsh
```

Current API commands:

```sh
linear --json viewer
linear team list
linear --json project list --limit 20
linear --json project view <project-id>
linear --json issue list --team-id <id> --limit 20
linear --json issue read ENG-123
linear --json issue create --team-id <id> --title "Fix auth bug" --description-file issue.md
linear --json issue update ENG-123 --state-id <workflow-state-id>
linear --json milestone list --project <project-id>
linear --json milestone view <milestone-id>
linear --json milestone create --project <project-id> --name "Q1 Goals" --target-date "2026-03-31"
linear --json milestone update <milestone-id> --name "New Name"
linear --json milestone delete <milestone-id> --force
linear --json m list --project <project-id>
linear --json m view <milestone-id>
linear --json m create --project <project-id> --name "Q1 Goals"
linear --json m update <milestone-id> --target-date "2026-04-15"
linear --json m delete <milestone-id> --force
```

## Command Notes

### `auth login`

```sh
linear auth login
linear auth login --api-key lin_api_xxx --force
```

Prompts for a Linear personal API key and writes it to `~/.config/forge/linear/token`. Use `--api-key` for non-interactive setup and `--force` to overwrite an existing token file.

### `config`

```sh
linear config
```

Creates `~/.config/forge/linear/config.toml` if it does not already exist and returns the config and token paths as JSON.

### `completions`

```sh
linear completions zsh
```

Prints shell completion scripts to stdout.

### `viewer`

```sh
linear --json viewer
```

Returns the authenticated viewer.

### `team list`

```sh
linear --json team list
```

Returns accessible teams.

### `project list`

```sh
linear --json project list [--limit <n>]
```

Returns accessible projects.

### `project view`

```sh
linear --json project view <project-id>
```

Reads a single project by UUID.

### `issue list`

```sh
linear --json issue list [--team-id <id>] [--assigned-to-me] [--limit <n>]
```

Returns issues for a team, with optional `assignedToMe` filtering.

Implementation note:

- `--assigned-to-me` currently fetches the team issue list and filters it client-side by the authenticated viewer ID
- this is intentional so the CLI, not the LLM, does the heavy lifting
- it also avoids depending on a more brittle server-side relation filter shape

### `issue read`

```sh
linear --json issue read <issue-id>
```

Reads a single issue by Linear issue identifier such as `ENG-123`.

### `issue create`

```sh
linear --json issue create --team-id <id> --title <title> [--description <text>] [--description-file <path>] [--state-id <id>]
```

Creates a new issue using the GraphQL `issueCreate` mutation.

### `issue update`

```sh
linear --json issue update <issue-id> [--title <title>] [--description <text>] [--description-file <path>] [--state-id <id>]
```

Updates an issue using the GraphQL `issueUpdate` mutation.

### `milestone list`

```sh
linear --json milestone list --project <project-id> [--limit <n>]
```

Returns milestones for a project.

### `milestone view`

```sh
linear --json milestone view <milestone-id>
```

Reads a single project milestone by UUID.

### `milestone create`

```sh
linear --json milestone create --project <project-id> --name <name> [--description <text>] [--description-file <path>] [--target-date <yyyy-mm-dd>]
```

Creates a project milestone. Unlike upstream `linear`, Forge does not support an interactive create flow in v1.

### `milestone update`

```sh
linear --json milestone update <milestone-id> [--name <name>] [--description <text>] [--description-file <path>] [--target-date <yyyy-mm-dd>]
```

Updates a project milestone.

### `milestone delete`

```sh
linear --json milestone delete <milestone-id> --force
```

Deletes a project milestone. `--force` is required because this is destructive.

## Safety Rules

- reads are safe by default
- writes are explicit verbs
- destructive milestone deletion requires `--force`
- no hidden VCS side effects
- no hidden PR creation
- `auth login` writes only to the documented config dir token file

## Install And Run

Run from source:

```sh
cargo run -p linear -- --json viewer
cargo run -p linear -- auth login
cargo run -p linear -- config
cargo run -p linear -- completions zsh
```

Install locally:

```sh
cargo install --path crates/linear
```

## API Notes

Linear's docs say:

- use `Authorization: <API_KEY>` for personal API keys
- use `Authorization: Bearer <ACCESS_TOKEN>` for OAuth
- check GraphQL `errors` even when HTTP status is `200`

## Future Extensions

- issue comments
- documents
- team members and workflow states
- repo-aware Git and `jj` helpers
- `gh` PR helpers
