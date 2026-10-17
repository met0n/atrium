<p align="center">
  <img src="src/assets/brand/atrium-app-icon.svg" width="128" height="128" alt="Atrium" />
</p>

<h1 align="center">Atrium</h1>

<p align="center">
  A desktop workspace for running, organizing, and resuming coding agents and shells side by side.
</p>

Atrium gives every project a set of real terminals (PTYs) arranged in split panes, with sub-tabs
for Claude Code, Codex, OpenCode, Cursor, Copilot, Antigravity, Kiro and plain shells. Layouts,
scrollback and agent sessions survive restarts, and an orchestrator lets a lead agent hand work
to parallel Codex or Claude workers in isolated git worktrees.

Atrium is a fork of [Alethe](https://github.com/Kc1t/alethe-agents) by Kauã Miguel, focused on
being a first-class **macOS** app while staying cross-platform.

## Features

- **Workspace** — groups, projects, split-pane grids, a layout designer, tabs and history, isolated
  local profiles.
- **Terminals** — real PTYs with persisted scrollback, suspend/priority controls and process trees.
- **Agent sessions** — list, title and resume past Claude Code, Codex, OpenCode, Cursor and
  Antigravity sessions.
- **Usage and cost** — live quota per provider, model pricing and cost per session.
- **Orchestrator** — an MCP server that lets a lead agent delegate, steer and collect parallel
  workers, each optionally in its own git worktree.
- **Git** — status, staging, commits, branch graph, cherry-pick/revert, worktrees and pull requests.
- **MCP and skills** — scan, sync and health-check MCP servers across every installed agent.
- **More** — embedded browser pane, plugins, remote control from a phone, local speech-to-text,
  backups.

See [`docs/FEATURES.md`](docs/FEATURES.md) for the full list.

## Privacy and network

“Local-first” describes workspace persistence, not an internet-free guarantee. Provider usage
polling is on by default, update checks are off until Atrium ships signed releases, and other
network features are optional or action-triggered. Manual GitHub Gist Sync is already available.

- Handing a conversation from one agent to another goes through a locally redacted context packet.
  Redaction is best effort, so review the packet before starting the target agent.
- Remote Control is off by default and uses unencrypted HTTP/WebSocket transport on the LAN, so
  enable it only on a trusted network. Clean profiles are read-only; answering agents requires a
  separate input opt-in.

See [`docs/PRIVACY.md`](docs/PRIVACY.md) for network defaults, credentials and retention.

## Run from source

Requirements: Node.js 18+, Rust stable, and the Xcode Command Line Tools on macOS (Visual Studio
Build Tools on Windows, the Tauri system packages on Linux).

```sh
git clone https://github.com/met0n/atrium.git
cd atrium
npm install
npm run app          # desktop app with hot reload
```

```sh
npm run dev          # frontend only (no Tauri IPC)
npm run build        # typecheck + build the frontend
npm test             # unit tests
npm run tauri build  # app bundle → src-tauri/target/release/bundle/
```

## Terminal command

Install the `atrium` command from **Settings ▸ Integrations ▸ Terminal command**:

```bash
atrium                # opens the current folder as a project
atrium ~/some/project # opens the given folder
```

## Releases and updates

Atrium does not publish signed releases yet, so the in-app updater is off. To turn it on, generate
a signing key (`npx tauri signer generate`), put its public key in
`src-tauri/tauri.conf.json` → `plugins.updater.pubkey`, and build releases with
`VITE_ATRIUM_UPDATER=1` and `bundle.createUpdaterArtifacts` enabled.

Cloud sync needs a GitHub OAuth app of your own: set `ATRIUM_SYNC_GITHUB_CLIENT_ID` at build time.

## License and attribution

Atrium is free software under the **GNU Affero General Public License v3.0 or later**; see
[`LICENSE`](LICENSE). It is based on Alethe, Copyright (C) 2026 Kauã Miguel and Alethe
contributors; the original notices are preserved in [`NOTICE`](NOTICE). Atrium is an independent
project and is not affiliated with or endorsed by the Alethe project. See
[`TRADEMARK.md`](TRADEMARK.md).
