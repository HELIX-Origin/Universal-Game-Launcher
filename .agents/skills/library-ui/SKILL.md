# Skill: library UI

**Use when:** implementing or reviewing the Svelte library view and game actions.

1. Inspect `src/routes/+page.svelte`, the registered commands in `src-tauri/src/lib.rs`, and the serialized models in `src-tauri/src/models.rs`.
2. Plan the library's loading, empty, populated, refresh, and failed-command states; show launch/install controls only for supported entries.
3. Reuse the English catalog in `src/lib/i18n/locales/en.ts` where appropriate. Do not claim the locale registry represents shipped translations.
4. Verify with `npm run check` and `npm run build`; arrange a desktop smoke test if the change depends on Tauri IPC.

Keep action labels and error behavior consistent with the backend. Hand off missing command contracts to the lead rather than inventing them in the UI.
