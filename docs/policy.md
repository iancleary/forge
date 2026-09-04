# Forge Policy

This document defines the planned Forge policy manager for machine and profile-specific agent capabilities.

The policy manager belongs in Forge because it reconciles installable tools, skills, and product skill packs on a machine. It should not live in `iancleary/skills`, because a skills repo should describe agent behavior, not own machine state.

## Goal

Let a machine declare which agent capabilities should be installed, pinned, verified, and reported.

The first supported capability type is an installable source repository:

- tools repo, such as `iancleary/forge`
- pure skills repo, such as `iancleary/skills`
- product skill pack, such as `American-Embedded/kistack`
- product CLI with bundled skills, such as `basecamp/basecamp-cli`

Policy answers this question:

```text
What should this machine have available for agents?
```

Each source repository still owns its own installation contract, usually through `install.md`.

## Non-Goals

Forge policy is not a general package manager.

It must not manage:

- language toolchains
- operating-system packages
- editor settings
- unrelated dotfiles
- secrets or account credentials
- arbitrary shell bootstrap scripts

Nix and Home Manager remain the owner for machine package state on managed systems. Forge policy should be able to read policy rendered by Nix, but it should not replace Nix.

## Policy File

The portable policy format is TOML.

Default locations, in precedence order:

```text
--policy <path>
./forge-policy.toml
~/.config/forge/policy.toml
```

Example:

```toml
version = 1
profile = "electronics"

[[sources]]
id = "forge"
kind = "tools"
repo = "iancleary/forge"
install = "install.md"
scope = "user"
reason = "Forge toolbelt and managed local assets"

[[sources]]
id = "ian-skills"
kind = "skills"
repo = "iancleary/skills"
install = "install.md"
scope = "user"
reason = "Ian's portable agent workflow skills"

[[sources]]
id = "kistack"
kind = "product-skills"
repo = "American-Embedded/kistack"
install = "install.md"
profile = "electronics"
scope = "user"
reason = "KiCad skills for electronics workstations"

[lock."American-Embedded/kistack"]
ref = "main"
commit = "REVIEWED_COMMIT_SHA"
```

## Source Kinds

`kind` is descriptive routing metadata, not a permission bypass.

Supported kinds:

- `tools`: installs executable tooling and may install managed assets
- `skills`: installs portable agent skills
- `product-skills`: installs domain-specific skills owned by another product repo
- `product-cli`: installs a product CLI and its bundled agent setup

Unknown kinds must fail validation.

## Profiles

Profiles are filters over sources.

Rules:

- sources without `profile` apply to every profile
- sources with a matching `profile` apply only to that profile
- a machine can pass one profile at a time
- omitted profile means apply only unprofiled sources

Examples:

```sh
forge policy status --profile electronics --json
forge policy install --profile electronics --json
```

## Commands

The planned top-level command is:

```sh
forge policy <command>
```

### `forge policy validate`

```sh
forge policy validate [--policy <path>] [--profile <name>] [--json]
```

Checks that:

- the policy file parses
- `version` is supported
- source IDs are unique
- source kinds are known
- repositories use `owner/repo` form
- install paths are relative
- lock entries refer to declared repositories

Validation does not fetch the network and does not install anything.

### `forge policy status`

```sh
forge policy status [--policy <path>] [--profile <name>] [--json]
```

Reports desired sources, installed state, lock state, and drift.

This is the default inspect-first command. It must not mutate state.

### `forge policy diff`

```sh
forge policy diff [--policy <path>] [--profile <name>] [--json]
```

Shows what `install` or `update --lock` would change.

This command may fetch remote metadata when needed to compare a lock against upstream, but it must not install or update anything.

### `forge policy install`

```sh
forge policy install [--policy <path>] [--profile <name>] [--json]
```

Installs missing sources and repairs out-of-date installed policy state.

Rules:

- run validation first
- run each source's declared install contract
- prefer pinned commits when a lock entry exists
- fail before mutation when an install contract is missing
- do not run arbitrary undocumented commands from the policy file

### `forge policy update --lock`

```sh
forge policy update --lock [--policy <path>] [--profile <name>] [--json]
```

Refreshes reviewed source pins in the lock section.

This is explicit because moving a third-party skill source to a new revision changes the instruction surface agents may load.

## JSON Contract

Policy JSON should use compact stable envelopes.

Example status shape:

```json
{
  "policy": "/home/user/.config/forge/policy.toml",
  "profile": "electronics",
  "summary": {
    "desired": 3,
    "installed": 2,
    "missing": 1,
    "drifted": 0
  },
  "sources": [
    {
      "id": "kistack",
      "kind": "product-skills",
      "repo": "American-Embedded/kistack",
      "profile": "electronics",
      "desired": true,
      "installed": false,
      "locked_commit": "REVIEWED_COMMIT_SHA",
      "state": "missing"
    }
  ]
}
```

Valid states:

- `installed`
- `missing`
- `drifted`
- `invalid`
- `unsupported`

## Safety

Policy installs change the local agent capability surface. Treat that as a controlled mutation.

Required safety properties:

- status and validate are read-only
- install is explicit
- lock updates are explicit
- third-party sources are pinned before unattended rollout
- private or account-specific data stays out of policy
- source install contracts are read from repositories, not embedded in machine policy

## Relationship To Skills

Forge policy chooses which skill sources a machine should install.

Individual skills still decide when an agent should use them through their `description` frontmatter and body instructions.

Do not copy third-party skills into Forge or `iancleary/skills` just to install them on a machine. Add them to policy, pin them, and let the source repo remain canonical.
