# Skill: store integration

**Use when:** adding or changing discovery of a locally installed store's games.

1. Inspect `src-tauri/src/stores/mod.rs`, the store-specific scanner, `models.rs`, and `library.rs`; identify platform and OS coverage before editing.
2. Read local client data without modifying it. Preserve stable game IDs and delegate launch/install to the appropriate official client.
3. Test parsing and error cases with existing Rust tests and use representative non-sensitive fixtures when needed.
4. Run `cargo test` in `src-tauri/` and record actual OS/client smoke tests separately from unit tests.

Do not put account credentials or real user-library data into fixtures, logs, or handoffs.
