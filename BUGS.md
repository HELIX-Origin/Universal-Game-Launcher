# 🐛 Confirmed bugs

The issues below are confirmed by inspecting the current source. They are **not** reports of behavior reproduced on every operating system. Add environment details when a runtime issue is reproduced, and remove an entry only after fixing and verifying it.

## 🔴 Open issues

### B-01 — Starter page calls a command that is not registered

- **Where:** `src/routes/+page.svelte` invokes `greet` when its form is submitted. The command handler in `src-tauri/src/lib.rs` does not register `greet`.
- **How to reproduce:** Run the desktop application, enter a name on the starter page, and submit the form.
- **Expected:** A supported action completes, or the obsolete starter form is removed.
- **Actual from source inspection:** The invocation rejects because the command is not registered, and the page does not handle the rejection.
- **Suggested next step:** Replace the starter page with the library view and handle command failures in the UI.

### B-02 — Starter page requests logo files that are missing

- **Where:** `src/routes/+page.svelte` references `/vite.svg`, `/tauri.svg`, and `/svelte.svg`; none of these files exist in `static/`.
- **How to reproduce:** Open the starter page and inspect the requests for its three logo images.
- **Expected:** Referenced images load, or the page does not request them.
- **Actual from source inspection:** The referenced URLs have no corresponding files in `static/`.
- **Suggested next step:** Remove the starter links and logos when building the library view.

## 📝 Reporting another issue

Include the operating system and relevant client versions, exact reproduction steps, expected and actual behavior, the source area involved, and whether the issue was reproduced or inferred from code. Never include API keys, credentials, private machine paths, or personal game-library data.
