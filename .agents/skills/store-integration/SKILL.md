# 🏬 Skill: store integration

## 📌 When to use

Use this skill when adding or changing discovery of games from a locally installed store client.

## 🧭 Discovery and implementation checklist

1. Inspect `src-tauri/src/stores/mod.rs`, the store-specific scanner, `models.rs`, and `library.rs`. Identify the operating-system and platform coverage before editing.
2. Read local client data without modifying it. Preserve stable game IDs and delegate supported launch/install actions to the appropriate official client.
3. Use existing Rust tests for parsing and error cases. If fixtures are needed, keep them representative and non-sensitive.
4. Run `cargo test` from `src-tauri/`; report unit-test results separately from actual operating-system/client smoke tests.
5. Never put account credentials or real user-library data in fixtures, logs, or handoffs.
