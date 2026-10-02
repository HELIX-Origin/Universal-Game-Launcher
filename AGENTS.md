# Agent guide

Start at [README.md](README.md), then read [ROADMAP.md](ROADMAP.md), [BUGS.md](BUGS.md), and [TODO.md](TODO.md). The first priority is a usable library UI, not new store integrations. Do not describe the starter page as a working launcher.

## Code map and contracts

- `src-tauri/src/lib.rs` registers Tauri commands and initializes `AppState`; `src-tauri/src/library.rs` assembles library snapshots.
- `src-tauri/src/stores/` scans local client data; `models.rs` defines platform IDs and launch targets. Preserve stable serialized IDs when touching scanners.
- `src-tauri/src/launcher.rs` handles launch targets; `persistence.rs` manages user data and settings; `metadata.rs` handles opt-in enrichment and keys. Do not log or commit credentials or user library data.
- `src/routes/+page.svelte` is currently a starter page; `src/lib/i18n/` has an English catalog and locale registry, not a complete translated UI.
- The frontend is a static SvelteKit SPA (`src/routes/+layout.ts`). Tauri commands are not browser-server endpoints. Keep the TypeScript/Rust command names and serialized payloads consistent.

## Agent ecosystem

The primary agent coordinates tasks and assigns narrow changes to sub-agents; these are repository guidance files, not automatically executed services.

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

Select only the relevant role and skill for a task. The lead owns cross-boundary coordination and merges sub-agent handoffs using `.agents/templates/handoff.md`. Apply `.agents/rules/` to every change.

## Verification

From the repository root, run `npm ci` if dependencies are missing, `npm run check` for frontend types, and `npm run build` for frontend packaging. Run `cargo test` in `src-tauri/` for backend changes; `npm run tauri build` packages the desktop app and needs Tauri's OS prerequisites. Report commands that could not run, rather than claiming unverified success. Do not introduce new dependencies without need.
