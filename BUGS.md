# 🐛 BUGS

> [!IMPORTANT]
> All known bugs are listed here. Keep in mind, that if a bug is missing, it may not have been discovered or reported yet.
>
> The repository maintainers (and contributors) actively search for new bugs and update this document accordingly. In some cases, a bug will be spotted and fixed without this page being immediately updated. This page is primarily a living index and may not always reflect the most current state of the codebase.
>
> AI agents are strongly advised to update this page first and push it to the remote before working on any new bug fixes or features. This way the remote repository always has the most up-to-date list of known issues.

## 📖 Legend

### 🚦 Status

- ⚠️ **open** — reproducible, needs fixing *(Detailed lists with possible fixes encouraged. Attempt to include steps to reproduce, expected behavior, and actual behavior. An estimate of how long it might take to fix is also helpful.)*
- 🚧 **investigating** — repro/root-cause in progress *(List of issues currently being worked on. Used for tracking active work. must reference an existing bug from the open section.)*
- 🚫 **wontfix** — accepted limitations *(features that can't be fixed at this time without significant changes or trade-offs)*
- ✅ **resolved** — verified and fixed *(moved to closed with the corresponding release version or commit)*

### 🚨 Severity

- 🔴 **Critical**: *Bugs that cause crashes or major functionality loss.*
- 🟠 **High**: *Bugs that significantly impact usability but do not crash the app.*
- 🟡 **Medium**: *Bugs that affect certain features or have minor usability issues.*
- 🟢 **Low**: *Minor bugs or visual glitches that do not significantly impact the user experience.*

## 🚫 Known quirks & external limitations (wontfix bucket)

- **Verification Gaps vs Confirmed Bugs:** Desktop runtime behavior, metadata network calls, and store/client handoffs have not been smoke-tested in all local desktop environments. These are verification gaps, not confirmed defects; track the required checks in [TODO.md](TODO.md).

## 💡 Explicitly not bugs

- A planned feature or an untested platform scenario is not automatically a bug; record confirmed source defects or reproducible runtime failures here.

## ⚠️ Open

### B-04 — Settings load failure shows no message

- **Severity**: 🟡 Medium (UI Error Reporting)
- **Status**: ⚠️ open
- **Affected area**: `openSettings` in `src/routes/+page.svelte`.
- **Reproduction**: Open Settings while `get_settings` or `get_api_key_status` rejects. Reproduced with mocked IPC in `tests/dom/settings-dialog.dom.test.ts` (`it.fails` case "reports an error when settings cannot be loaded (B-04)").
- **Expected**: A visible error says that settings couldn't be loaded.
- **Actual**: The handler stores `page.settingsLoadError` in `dialogError` and then closes the dialog. `dialogError` is rendered only inside dialogs, so the user sees nothing.
- **Next steps**: When fixed, change the `it.fails` test to `it`.

## ✅ Closed

### B-01 — Starter page called a command that was not registered

- **Severity**: 🟡 Medium (Frontend / IPC)
- **Status**: ✅ resolved
- **Originally observed**: `src/routes/+page.svelte` called `greet` on form submission, but `src-tauri/src/lib.rs` did not register that command.
- **Resolution**: Replaced the starter form with the library UI, which calls registered commands and presents command failures.
- **Verification**: Source inspection and frontend checks confirmed the stale `greet` invocation is gone. No desktop smoke test has been performed.

### B-02 — Starter page requested logo files that were missing

- **Severity**: 🟢 Low (Static Assets)
- **Status**: ✅ resolved
- **Originally observed**: The starter page requested `/vite.svg`, `/tauri.svg`, and `/svelte.svg`; no corresponding files existed in `static/`.
- **Resolution**: Removed the starter links and image requests with the starter page.
- **Verification**: Source inspection confirmed those references were removed. No desktop smoke test has been performed.

### B-03 — Discontinued RAWG metadata provider could still be selected and queried

- **Severity**: 🟠 High (Provider Deprecation & Settings Migration)
- **Status**: ✅ resolved
- **Originally observed**: Metadata settings exposed RAWG and the backend sent requests to `api.rawg.io`, although the project had moved to IGDB.
- **Resolution**: Removed RAWG from selectable providers and outbound requests. Retained the legacy enum/key fields so older user data can still be deserialized; settings loading drops RAWG from the active provider list.
- **Verification**: `cargo test` from `src-tauri/` passes (62 tests, 0 failures), including legacy RAWG request rejection and settings migration; `cargo fmt --check` from `src-tauri/` passes. Tests use isolated fixtures and did not inspect real user store data. Desktop/provider runtime behavior remains unverified; see [TODO.md](TODO.md).

## 📝 Adding and closing bug records

When reporting a bug, include the affected source area, exact reproduction steps, expected and actual behavior, and whether it was reproduced or inferred from code. For runtime issues, include relevant OS/client versions. When fixing a bug, preserve its ID and record the resolution and verification result here.

Never include API keys, credentials, private machine paths, or personal game-library data in this ledger.
