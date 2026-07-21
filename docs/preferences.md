# Portable User Preferences

This document defines Forge's narrow desired-state contract for user preference files that are partly owned by another application or by the user.

The goal is not to mirror complete dotfiles. Forge owns named invariants and preserves every setting outside those invariants.

## Commands

```sh
forge preference check windows-terminal [options] [--json]
forge preference diff windows-terminal [options] [--json]
forge preference apply windows-terminal [options] [--json]
```

Common options:

```text
--settings <absolute-path>
--theme <name>
--font-face <name>
--git-bash-commandline <commandline>
```

Defaults:

- theme: `dark`
- font face: `Cascadia Mono`
- Git Bash command line: `"%PROGRAMFILES%\Git\bin\bash.exe" -li`

On Windows, Forge discovers the stable Store, Preview Store, or unpackaged Windows Terminal `settings.json`. If zero or multiple candidates exist, the command stops and asks for an explicit `--settings` path. An explicit path is also the supported testing path on macOS and Linux.

`check` and `diff` are read-only. `apply` is the only write command.

## Windows Terminal Ownership

Forge ensures these settings:

- root `theme`
- root `defaultProfile`, set to the Git Bash profile GUID
- `profiles.defaults.font.face`
- one visible Git Bash entry in `profiles.list`
- the Git Bash entry's GUID, name, and command line

The Git Bash profile identity is the fragment GUID documented by Windows Terminal:

```text
{2ece5bfe-50ed-5f3a-ab87-5cd4baafed2b}
```

Primary sources for this contract:

- [Windows Terminal JSON fragment extensions](https://learn.microsoft.com/en-us/windows/terminal/json-fragment-extensions) defines the Git Bash GUID and profile update identity.
- [Windows Terminal startup settings](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/startup) defines root `defaultProfile` behavior.
- [Windows Terminal profile appearance](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/profile-appearance) defines `profiles.defaults.font.face`.
- [Windows Terminal application appearance](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/appearance) defines the root `theme` setting.

Forge matches this GUID first and accepts an existing case-insensitive `Git Bash` name as a migration path. Multiple matches are rejected instead of guessed at.

Windows Terminal uses JSON with comments and trailing commas. Forge edits its concrete syntax tree so unrelated settings, comments, array ordering, and formatting remain intact. It does not replace the complete file or normalize it through ordinary JSON serialization.

## Current Boundary

This first adapter intentionally does not:

- install Windows Terminal, Git for Windows, or fonts
- manage the Windows OS-wide default terminal application
- reorder existing terminal profiles
- own `newTabMenu`, actions, color schemes, or custom themes
- apply preferences during `forge self update`
- provide a generic arbitrary JSON-path merge language

When a custom `newTabMenu` omits remaining profiles, the Git Bash profile can exist in `profiles.list` without appearing in that custom menu. Menu reconciliation should be added only with an explicit ordering and ownership contract.

## Adding Other Applications

New preference targets should follow the same lifecycle and choose one explicit ownership model:

- structured ensure for application-owned JSON, JSONC, TOML, or YAML
- a delimited managed block for line-oriented files
- whole-file ownership only when the user explicitly delegates the entire file

Each structured adapter owns its identity and merge rules. Arrays must be matched by a stable application-specific identity rather than by generic position.
