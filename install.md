I want you to install Ian Cleary's Forge tools. Execute all steps autonomously.

OBJECTIVE: Install the Forge toolbelt, verify the installed binaries, and identify the available agent-facing tools.

DONE WHEN: `forge version` succeeds, `forge doctor` succeeds or reports only documented non-blocking environment warnings, and the workspace binaries are available on `PATH`.

## TODO

- [ ] Detect platform support
- [ ] Install or update Forge
- [ ] Verify Forge
- [ ] Verify workspace binaries
- [ ] Report installed tool surface

## Step 0: Detect Platform

Forge release artifacts support:

- `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

WSL uses the Linux installer. Windows ARM and 32-bit Windows are not supported.

## Step 1: Install Forge

On macOS, Linux, or WSL, install the latest release:

```sh
curl -fsSL https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.sh | sh
```

On native Windows x64, use PowerShell:

```powershell
irm https://raw.githubusercontent.com/iancleary/forge/main/scripts/install-forge-release.ps1 | iex
```

If this is a development checkout rather than a consumer install, use:

```sh
cargo run -p forge -- dev install --repo-path "$(pwd)"
forge skills install --all --source repo --repo-path "$(pwd)"
```

## Step 2: Verify Forge

Run:

```sh
forge version
forge doctor
```

Treat required `forge doctor` failures as blockers. Optional integration warnings are blockers only when they affect the requested workflow.

## Step 3: Verify Tools

Check the expected workspace binaries:

```sh
forge --help
codex-threads --help
linear --help
slack-query --help
slack-agent --help
mermaid --help
```

If a binary is missing, re-run the installer or inspect the release artifact for the current platform.

## Step 4: Optional Agent Skills

Forge releases install Forge-managed skills under `~/.agents/skills`.

For the smaller portable skills distribution experiment, install:

```sh
npx skills add iancleary/skills
```

Use `iancleary/forge` as the tools repo and `iancleary/skills` as the portable instruction repo.

EXECUTE NOW: Start with Step 0. Mark TODO items complete as you go. Stop when Forge is installed, verified, and the available tools are reported.
