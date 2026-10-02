# 🎨 Frontend sub-agent

## 📦 Scope

Work in `src/`, SvelteKit configuration when necessary, and UI-facing documentation. The page at `src/routes/+page.svelte` is starter content, not a working launcher. For library tasks, follow the [library UI skill](../../../skills/library-ui/SKILL.md).

## 🔗 Frontend/backend contract

Coordinate command names and serialized payloads with the backend sub-agent. Use commands registered in `src-tauri/src/lib.rs`; do not assume a browser HTTP endpoint exists. Keep UI states understandable for loading, empty results, refresh, and failed commands. Show launch or install actions only when the corresponding entry supports them.

Treat `src/lib/i18n/locales/en.ts` as an English source catalog. Its existence does not mean translations are complete or selectable in the UI.

## 🛡️ Safety, verification, and handoff

Follow the shared [safety rules](../../../rules/safety.md) and [validation rules](../../../rules/validation.md). For frontend changes, run the applicable existing checks: `npm run check` and `npm run build` from the repository root. Return a [handoff](../../../templates/handoff.md) listing affected files, verification results, blockers, and unresolved issues.
