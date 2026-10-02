# 🐛 Repository bug ledger

This is the canonical record of bugs tracked in this repository. Record each confirmed source defect or reproducible runtime defect here, whether open or resolved. A planned feature or an untested platform scenario is not automatically a bug.

## 🟢 Open bugs

No open, confirmed bugs are currently recorded.

Desktop runtime behavior, metadata network calls, and store/client handoffs have not been smoke-tested in this environment. These are verification gaps, not confirmed defects; track the required checks in [TODO.md](TODO.md).

## ✅ Resolved bugs

### B-01 — Starter page called a command that was not registered

- **Originally observed:** `src/routes/+page.svelte` called `greet` on form submission, but `src-tauri/src/lib.rs` did not register that command.
- **Resolution:** Replaced the starter form with the library UI, which calls registered commands and presents command failures.
- **Verification:** Source inspection and frontend checks confirmed the stale `greet` invocation is gone. No desktop smoke test has been performed.

### B-02 — Starter page requested logo files that were missing

- **Originally observed:** The starter page requested `/vite.svg`, `/tauri.svg`, and `/svelte.svg`; no corresponding files existed in `static/`.
- **Resolution:** Removed the starter links and image requests with the starter page.
- **Verification:** Source inspection confirmed those references were removed. No desktop smoke test has been performed.

### B-03 — Discontinued RAWG metadata provider could still be selected and queried

- **Originally observed:** Metadata settings exposed RAWG and the backend sent requests to `api.rawg.io`, although the project had moved to IGDB.
- **Resolution:** Removed RAWG from selectable providers and outbound requests. Retained the legacy enum/key fields so older user data can still be deserialized; settings loading drops RAWG from the active provider list.
- **Verification:** Rust regression tests cover rejecting legacy RAWG requests and preserving other settings while dropping the legacy provider. `cargo fmt --check` passes; `cargo test` could not compile because this environment lacks the GTK/GIO development libraries (`gio-2.0`, `glib-2.0`, and `gobject-2.0`). The tests remain pending; see [TODO.md](TODO.md).

## 📝 Adding and closing bug records

When reporting a bug, include the affected source area, exact reproduction steps, expected and actual behavior, and whether it was reproduced or inferred from code. For runtime issues, include relevant OS/client versions. When fixing a bug, preserve its ID and record the resolution and verification result here.

Never include API keys, credentials, private machine paths, or personal game-library data in this ledger.
