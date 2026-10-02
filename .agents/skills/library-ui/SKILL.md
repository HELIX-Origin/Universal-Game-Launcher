# 🎮 Skill: library UI

## 📌 When to use

Use this skill when implementing or reviewing the Svelte library view and its game actions. The initial view exists; distinguish type/build checks from a desktop smoke test.

## 🧭 Implementation and review checklist

1. Inspect the current UI in `src/routes/+page.svelte`, commands registered in `src-tauri/src/lib.rs`, and serialized models in `src-tauri/src/models.rs`.
2. Cover the important view states: loading, empty, populated, refreshing, and failed commands. Show launch/install controls only when an entry supports the action.
3. Reuse the English catalog in `src/lib/i18n/locales/en.ts` when appropriate. Do not claim the locale registry represents shipped translations.
4. Keep action labels and error behavior consistent with the backend. Hand off missing command contracts to the lead instead of inventing them in the UI.
5. Verify applicable changes with `npm run check` and `npm run build` from the repository root. Arrange a desktop smoke test when behavior depends on Tauri IPC.
