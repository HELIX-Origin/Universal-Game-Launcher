# 🤖 Agent guide

Start with [README.md](README.md), then read [ROADMAP.md](ROADMAP.md), [BUGS.md](BUGS.md), and [TODO.md](TODO.md). The first priority is a usable library UI, not new store integrations. An initial library UI is in place; verify desktop behavior before describing it as a working launcher.

## 🧭 Code map and contracts

- **Tauri commands and state:** `src-tauri/src/lib.rs` registers commands and initializes `AppState`; `src-tauri/src/library.rs` assembles library snapshots.
- **Store discovery and models:** `src-tauri/src/stores/` reads local client data. `models.rs` defines platform IDs and launch targets; preserve stable serialized IDs when changing scanners.
- **Launching, persistence, and metadata:** `src-tauri/src/launcher.rs` handles launch targets; `persistence.rs` manages user data and settings; `metadata.rs` provides opt-in enrichment and key status. Never log or commit credentials or private library data.
- **Frontend and localization:** `src/routes/+page.svelte` contains the initial library interface. `src/lib/i18n/` contains an English catalog and locale registry, not a complete translated UI.
- **Application boundary:** The frontend is a static SvelteKit SPA configured in `src/routes/+layout.ts`. Tauri commands are not browser-server endpoints. Keep TypeScript command names and serialized payloads consistent with Rust.

## 🧩 Agent ecosystem

The primary agent owns task scope and cross-boundary coordination. Sub-agents provide focused work and handoffs; these Markdown files are repository guidance, not automatically running services.

```text
.agents/
  agents/launcher-lead/AGENT.md
  agents/launcher-lead/frontend/AGENT.md
  agents/launcher-lead/backend/AGENT.md
  agents/launcher-lead/verification/AGENT.md
  skills/library-ui/SKILL.md
  skills/store-integration/SKILL.md
  rules/safety.md
  rules/validation.md
  templates/handoff.md
```

Choose only the role and skill relevant to the task. The lead coordinates frontend/backend work and integrates sub-agent findings using `.agents/templates/handoff.md`. Apply `.agents/rules/` to every change.

## 🔍 Verification

From the repository root, run `npm ci` if dependencies are missing. Then run `npm run verify -- <suites|groups>` to choose test layers (`--list` shows the catalog). Use `quick` for fast frontend feedback, `frontend` for types, unit, DOM, contract, and build checks, `backend` for Rust formatting, unit, and integration tests, and `boundary` for both sides of the Rust ↔ TypeScript contracts. The individual commands still work: `npm run check`, `npm test`, `npm run build`, and `cargo test` from `src-tauri/`. [tests/README.md](tests/README.md) describes the layout, helpers, contract workflow, and debugging tips. A packaged desktop build uses `npm run tauri build` and requires the Tauri prerequisites for the operating system.

When a command name, argument, or serialized payload changes, update both sides and run `npm run verify -- boundary`. Regenerate golden fixtures with `--update-contracts` only for intentional payload changes.

Run only checks relevant to the change. State exact commands and outcomes, distinguish source inspection from runtime testing, and report environmental blockers rather than claiming unverified success. Do not add dependencies without need.
