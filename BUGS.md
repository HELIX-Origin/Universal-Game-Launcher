# Bugs

Issues below are confirmed by the current source, not reports of tested behavior on every OS. Add reproduction steps and environment details for newly discovered runtime issues; remove entries once fixed and verified.

## Open

### B-01 — Starter page calls an unregistered command

- **Where:** `src/routes/+page.svelte` invokes `greet` on form submission; `src-tauri/src/lib.rs` registers commands in `tauri::generate_handler!` but does not register `greet`.
- **Reproduce:** Run the desktop app, enter a name, and submit the form.
- **Expected:** A supported action completes, or the obsolete starter form is removed.
- **Actual:** The invocation rejects because the command is not registered; the page has no error handling for the rejection.
- **Next:** Replace the starter page with a library view and cover command failures in the UI.

### B-02 — Starter page references missing logo assets

- **Where:** `src/routes/+page.svelte` references `/vite.svg`, `/tauri.svg`, and `/svelte.svg`; none of these files exist in `static/`.
- **Reproduce:** Open the starter page and inspect its three logo requests.
- **Expected:** Logo images load or are not requested.
- **Actual:** The image URLs have no corresponding static files.
- **Next:** Remove the starter links and logos when building the library view.

## Reporting a new issue

Record OS and client versions, steps to reproduce, expected and actual behavior, the relevant source area, and whether the issue was reproduced or only inferred from code. Never paste API keys, personal library data, or credentials.
