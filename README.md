# Universal Game Launcher

A desktop launcher intended to show games from multiple stores in one library. The Rust/Tauri backend reads locally installed clients and their data, keeps user-added games and settings, and delegates launching or installation to the official client. Optional metadata providers can enrich entries; discovery itself does not require a store account or online API.

**Current state:** early work in progress. The backend has store scanners and Tauri commands, but the Svelte page is still the starter greeting screen and does not expose the library. Do not treat the current window as a working game launcher.

## Where to start

- [ROADMAP.md](ROADMAP.md) — milestones and priorities.
- [BUGS.md](BUGS.md) — reproducible issues observed in the current tree.
- [TODO.md](TODO.md) — ordered next-session checklist.
- [AGENTS.md](AGENTS.md) — architecture, working conventions, and agent handoffs.

## Architecture

| Area | Location | Responsibility |
| --- | --- | --- |
| Desktop commands | `src-tauri/src/lib.rs` | Tauri command registration and app state |
| Discovery | `src-tauri/src/stores/`, `src-tauri/src/library.rs` | Local store scanning and library snapshot |
| Launching and data | `src-tauri/src/launcher.rs`, `src-tauri/src/persistence.rs` | Safe launch targets, custom games, settings |
| Metadata | `src-tauri/src/metadata.rs` | Optional provider lookups and API-key status |
| UI | `src/routes/+page.svelte`, `src/lib/i18n/` | Starter page and incomplete localization groundwork |

The desktop app uses Tauri 2, Rust, SvelteKit, Svelte 5, TypeScript, and Vite. Store integrations are platform-dependent; cloud services are user-added shortcuts, not automatically discovered libraries.

## Development

Install Node.js/npm, Rust (at least 1.85), and the [Tauri 2 system prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS. Then run from the repository root:

```sh
npm ci
npm run check
npm run build
npm run tauri dev
```

`npm run build` builds only the web frontend. To test the Rust backend, run `cargo test` from `src-tauri/`; a packaged desktop build uses `npm run tauri build`. No frontend test script is currently defined.

## Continuing the work

Start with [TODO.md](TODO.md), verify the issues in [BUGS.md](BUGS.md), and keep the handoff documents aligned with implemented behavior. The agent roles, task-specific skills, rules, and handoff template are indexed in [AGENTS.md](AGENTS.md).