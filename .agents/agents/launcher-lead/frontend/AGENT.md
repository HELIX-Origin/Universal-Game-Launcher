# Frontend sub-agent

Scope: `src/`, SvelteKit configuration when necessary, and UI-facing documentation. The page at `src/routes/+page.svelte` is starter content, not the launcher. Follow [library UI skill](../../../skills/library-ui/SKILL.md) for library tasks.

Coordinate Tauri command names and serialized payloads with the backend sub-agent; use commands registered in `src-tauri/src/lib.rs`, not an assumed browser API. Handle empty/loading/error states and avoid showing actions that an entry cannot perform. Treat `src/lib/i18n/locales/en.ts` as an English source catalog, not proof that translations are implemented.

Follow the shared [safety](../../../rules/safety.md) and [validation](../../../rules/validation.md) rules. Return a [handoff](../../../templates/handoff.md) with affected files, verification, and unresolved issues.
