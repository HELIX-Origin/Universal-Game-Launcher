# 🎮 Universal Game Launcher

Universal Game Launcher is a desktop application intended to bring games from multiple stores into one library. The Rust/Tauri backend reads locally installed clients and their data, stores user-added games and settings, and delegates supported launch or installation actions to the official client.

Optional metadata enrichment uses **IGDB** in place of the discontinued RAWG API. Browsing locally discovered games does not require a store account or an online metadata service.

> ⚠️ **Project status:** This project is early work in progress. An initial library interface now loads the backend snapshot and offers supported game actions, but it has not yet been smoke-tested in the desktop app with real store clients. Do not treat the current build as release-ready.

## 🧭 Start here

These documents describe the current project, known issues, priorities, and contributor workflow:

- [📍 Roadmap](ROADMAP.md) — planned milestones and priorities.
- [🐛 Bugs](BUGS.md) — issues confirmed from the current source.
- [✅ Next-session checklist](TODO.md) — immediate work items; unchecked items are not shipped features.
- [🤖 Agent guide](AGENTS.md) — architecture, working conventions, and the agent handoff process.

## 🏗️ Architecture at a glance

| Area | Location | Responsibility |
| --- | --- | --- |
| Desktop commands | `src-tauri/src/lib.rs` | Registers Tauri commands and initializes app state. |
| Discovery | `src-tauri/src/stores/`, `src-tauri/src/library.rs` | Reads local store data and assembles library snapshots. |
| Launching and user data | `src-tauri/src/launcher.rs`, `src-tauri/src/persistence.rs` | Validates launch targets and manages custom games and settings. |
| Optional metadata | `src-tauri/src/metadata.rs` | Fetches opt-in metadata from enabled providers, supports IGDB in place of discontinued RAWG, and reports key status without returning secret values. |
| User interface | `src/routes/+page.svelte`, `src/lib/i18n/` | Library with search, filters, supported actions, custom local/cloud games, basic platform preferences, metadata-provider opt-in, and write-only credentials; localization remains incomplete. |

The application uses Tauri 2, Rust, SvelteKit, Svelte 5, TypeScript, and Vite. Store discovery depends on the operating system and installed client. Cloud services are user-added shortcuts, not automatically discovered libraries.

## 🧰 Development setup

Install Node.js/npm, Rust (at least version 1.85), and the [Tauri 2 system prerequisites](https://v2.tauri.app/start/prerequisites/) for your operating system. From the repository root:

```sh
npm ci
npm run check
npm test
npm run build
npm run tauri dev
```

### 🔍 What the checks cover

- `npm run check` checks the frontend types.
- `npm test` runs the frontend library behavior and initial-state regression tests.
- `npm run build` packages the web frontend; it does not package the desktop application.
- `npm run tauri dev` starts the desktop application and requires the Tauri system prerequisites.
- To test the Rust backend, run `cargo test` from `src-tauri/`.
- To package the desktop application, run `npm run tauri build`.

Frontend unit tests do not replace a desktop smoke test. Report commands that could not run instead of treating them as successful.

## 🚀 Continuing development

Start with the [next-session checklist](TODO.md), check confirmed issues in [BUGS.md](BUGS.md), and keep the [roadmap](ROADMAP.md) aligned with verified progress. Follow [AGENTS.md](AGENTS.md) when using the repository's roles, skills, safety rules, and handoff template.
