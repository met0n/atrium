# AGENTS.md

This file provides guidance to coding agents (Codex, OpenCode, Claude Code and others) working in this repository.

> Identical in content to [`CLAUDE.md`](CLAUDE.md) in this directory. Keep both in sync.
> Contributing from outside? Start with [`CONTRIBUTING.md`](CONTRIBUTING.md).

## What it is

**Atrium** is a macOS-first, cross-platform Tauri 2 desktop app (Rust backend, React 18 +
TypeScript + Vite frontend, Zustand state, xterm.js) for running coding agents (Claude Code,
Codex, OpenCode, Cursor, Copilot, Antigravity, Kiro…) and shells side by side in real PTYs, with
persistent layouts, session resume, usage/cost tracking, an agent orchestrator, git/worktree tools,
MCP management, plugins and LAN remote control. Bundle identifier `com.met0n.atrium`.

It is a fork of [Alethe](https://github.com/Kc1t/alethe-agents) (AGPL-3.0-or-later). Keep the
upstream copyright and attribution in `NOTICE`; never reintroduce the Alethe name, logo or icon
(`TRADEMARK.md`).

## Commands

```sh
npm install
npm run app            # tauri dev with hot reload — the normal way to run the app
npm run dev            # Vite frontend only on 127.0.0.1:1422 (no Tauri IPC: terminals won't work)
npm run build          # tsc (typechecks and validates i18n) + vite build
npm run lint           # eslint
npm run format:check   # prettier (npm run format to fix)
npm test               # vitest, jsdom, src/**/*.{test,spec}.{ts,tsx}
npx vitest run src/lib/platform.test.ts          # one file
npx vitest run -t "hasPrimaryModifier"           # by test name
npm run test:rust                                 # cd src-tauri && cargo test --lib
cd src-tauri && cargo test --lib quit_request     # Rust tests matching a path/name
cd src-tauri && cargo test                        # also runs tests/orchestrator.rs
cd src-tauri && cargo clippy --all-targets
npm run tauri build    # bundle → src-tauri/target/release/bundle/ (macOS: Atrium.app + .dmg)
```

CI (`.github/workflows/ci.yml`) runs lint, format:check, `npm test` and `npm run build` on Ubuntu,
then `cargo check`, `cargo clippy --all-targets` and `cargo test` on Windows, Linux and macOS.

- E2E tests (`npm run test:e2e:build` then `npm run test:e2e`) drive the debug binary through
  `tauri-driver`, which only exists for Linux and Windows — they cannot run on macOS.
- A release build needs `node_modules`: `build.rs` copies the xterm files for the remote-control
  page out of it and panics in release if they are missing.
- Windows installers need the MSVC environment: run `npm run tauri build` from a `vcvars64` shell.
- The in-app updater is off unless the build sets `VITE_ATRIUM_UPDATER=1` and
  `plugins.updater.pubkey` holds Atrium's own key; `bundle.createUpdaterArtifacts` stays false
  until then, so local builds need no signing key. Cloud sync needs `ATRIUM_SYNC_GITHUB_CLIENT_ID`.
- The Tauri CLI refuses to build when an `@tauri-apps/*` npm package and its Rust crate differ in
  major/minor version. Bump them together; `@tauri-apps/api` and the notification and updater
  plugins are pinned with `~` in `package.json` for this reason.

## Non-negotiable rules

1. **Do not stop or restart the app or the dev server** if the owner is running it; changes land
   through HMR.
2. **Do not commit, push, tag or release without explicit permission at that moment.**
3. **Design system:** no gradients, no generic template UI, real data only (no mocks in widgets).
   One `.module.css` per component; colors and spacing only through the tokens in
   `src/styles/theme.css` (`--bg`, `--fg`, `--accent`, `--agent-*`, `--status-*`…).
4. **i18n:** every visible string goes through `t()`. Add the key to
   `src/lib/i18n/messages/en.ts` (source of truth) and to `pt-BR.ts` and `zh-CN.ts`; they are typed
   against `en.ts`, so `npm run build` fails on a missing translation.
5. **Changelog:** every user-facing addition, change or removal updates `docs/CHANGELOG.md` in the
   same task, under the `Atrium [Unreleased]` section at the top.

## Architecture

**Frontend → backend.** Components never call `invoke` directly: `src/lib/tauri/*.ts` holds one
typed wrapper module per backend area (pty, git, mcp, orchestrator, usage, plugins…), re-exported
from `src/lib/tauri/index.ts`. A new backend command needs a `#[tauri::command]` in its Rust
module, an entry in the `generate_handler![…]` list in `src-tauri/src/lib.rs`, and a wrapper here.
Streaming comes back as Tauri events, e.g. `pty://data/{id}` and `pty://exit/{id}`.

**State.** `src/stores/projectsStore.ts` is the persisted workspace (groups, projects, terminals,
tabs, layouts, preferences), split into `projectsStore.{project,terminal,workspace}Slices.ts`. It
is saved to `profiles/<profileId>/projects.json` through `save_projects` (debounced, atomic tmp →
rename). The file is versioned: any shape change needs a step in
`projectsStore.migrations.ts`. Runtime-only PTY state lives in `terminalsStore`, transient UI
(modals, toasts) in `uiStore`. Use narrow selectors; broad ones re-render the whole workspace.

**Rust backend (`src-tauri/src`).** Flat modules per domain: `pty.rs` + `process_tree.rs`
(PTYs, scrollback, kill trees), `*_sessions.rs` / `*_usage.rs` / `agent_cost.rs` (reading each
agent's local history and quota), `git_control.rs` / `worktrees.rs` / `github_pr.rs`,
`mcp_*.rs` / `skills.rs`, `plugins*.rs`, `remote/` (LAN HTTP + WebSocket server for the
`src-tauri/remote/` phone page), `speech*.rs` (local sherpa-onnx dictation), `event_bus.rs`
(internal broadcast feeding `telemetry.rs`).

**Agent integration.** `agent_events.rs` runs a token-protected loopback HTTP listener on the
first free port in 127.0.0.1:9123–9143. Agents report back through it: Claude Code gets HTTP hooks
and an HTTP MCP server via a generated settings file; Codex gets `.codex/config.toml` entries
pointing at bridge scripts (PowerShell on Windows, `sh` + `curl` elsewhere) because it only speaks
stdio MCP and command hooks; OpenCode gets a bridge plugin in `~/.config/opencode/plugin/`.

**Orchestrator.** `orchestrator_core.rs` is the delegation engine (job queue, Codex/Claude
workers, the `atrium_*` MCP tools). It must stay free of Tauri and of other crate modules: it is
compiled directly into the standalone stdio server `src/bin/atrium-orchestrator-mcp.rs` and into
`tests/orchestrator.rs`. `orchestrator.rs` is the app layer that hosts it over the listener's
`/mcp` route; `scheduler.rs` holds the task DAG.

**Plugins.** The host, registry, permissions, trust and storage are in `src/lib/plugins/`;
bundled plugins (`git-control`, `todos`, `theme-pack`) in `src/plugins/`. Plugin assets are
served over the `atrium-plugin://` protocol registered in `lib.rs`, which the CSP allowlists.
See `docs/PLUGINS.md`.

**Platform specifics.**
- macOS window: `src-tauri/tauri.macos.conf.json` (merged over `tauri.conf.json`) keeps native
  traffic lights over the title bar; `useNativeWindowControls` gives the title bar room for them.
  `window_style.rs` rounds corners only for an undecorated window.
- `quit_request.rs` replaces the stock macOS Quit item: AppKit's terminates before Tauri sees it,
  skipping the frontend close flow (`useCloseConfirmation` → `quit_app`) that saves the workspace.
- `shell_path.rs` adopts the login shell's PATH when launched from Finder (launchd's minimal PATH).
- App shortcuts use ⌘ on macOS and Ctrl elsewhere through `hasPrimaryModifier()` in
  `src/lib/platform.ts`; never test `ctrlKey || metaKey` for a new shortcut, because Ctrl must
  reach the terminal on macOS.

## Gotchas and security

- `tauri.conf.json` ships a strict CSP (`script-src 'self'`, `worker-src 'none'`) asserted by
  `src/securityPolicy.test.ts`; changing it is a deliberate, reviewed decision. The webview has
  full IPC access, so treat rendered content as untrusted.
- `spawn_pty` runs commands and args supplied by the frontend; validate them before spawning.
- Secrets (GitHub sync token, Spotify credentials) go through `secure_store.rs`, the OS keyring
  under service `com.met0n.atrium`. Never write a new secret into `projects.json` or another JSON
  file. Agent credentials stay in each agent's own files and are only read.
- Local data: `~/Library/Application Support/com.met0n.atrium/` on macOS, the app-data folder
  elsewhere (`profiles.json`, `profiles/<id>/projects.json`, `scrollback/*.bin`, `spawn.log`,
  root `logs/`).
- Many Rust warnings on macOS come from Windows-only code; don't "fix" them by deleting it.

## Language and comments

- English for all versioned content: comments, docs, changelog, UI strings, commits and PRs.
  Locale files are the only exception. Translate non-English comments in files you touch.
- Comment only non-obvious behavior, constraints or decisions.

## Going deeper

`docs/OVERVIEW.md` (domain model: Group → Project → Container → Pane → Terminal → Sub-tab → PTY,
persistence), `docs/FEATURES.md`, `docs/PLUGINS.md`, `docs/THEMES.md`, `docs/PRIVACY.md` (every
network surface and stored file), `docs/ROADMAP.md` and `docs/ORCHESTRATOR_ROADMAP.md`,
`docs/DIAGNOSTICO_MATURIDADE_TECNICA.md` (known debt: the large `projectsStore`, thin frontend
test coverage, duplicated polling).

The `graphify-query` skill (`.claude/skills/`) works only after a Graphify knowledge graph has
been generated into `graphify-out/`, which is not checked in.
