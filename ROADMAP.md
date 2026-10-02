# 🗺️ Roadmap

This roadmap describes intended work; it is **not** a claim that the user interface or integrations are complete. See the [README](README.md) for current project status and the [next-session checklist](TODO.md) for immediate tasks.

## 🎯 Now — deliver a usable first library

- [x] Replace the starter page with a library view powered by `get_library`.
- [x] Add understandable loading, empty, refresh, and command-error states.
- [x] Connect launch and install actions only when an entry supports the action; surface failures.
- [x] Add frontend regression coverage for initial loading state and library filtering/action eligibility.
- [ ] Smoke-test the desktop UI and launch/install handoffs on supported operating systems with locally installed store clients. Record what was actually tested.

## 🧩 Next — personal library and settings

- [x] Surface favorite and hidden-game management in the library using existing backend commands.
- [x] Add/remove local games and cloud shortcuts, open available install folders, and edit enabled libraries/minimize-on-launch settings.
- [x] Let users opt into metadata providers, store credentials write-only, and request game details.
- [x] Retire RAWG network requests and remove legacy RAWG selections from settings while retaining stored enum/key fields for backward-compatible reads.
- [ ] Verify provider requests, key saving/status, and metadata rendering from the running desktop app.
- Keep third-party credentials optional: browsing the local library must not depend on them.
- [x] Connect user-facing library UI copy to the English source catalog and clarify that locale tags are not bundled UI translations.
- [ ] Add translated dictionaries and a language selector together; do not advertise a locale as a supported UI language until its strings can be selected and displayed.

## 📚 Delivered milestones and records

- **Initial library:** replaced starter content with `get_library` loading, refresh, empty/error states, search and filters; connected supported launch/install actions.
- **Personal library:** added favorite and hide/unhide actions; add/remove local executable and cloud shortcut entries; open available install folders.
- **Preferences:** configure enabled libraries and minimize-on-launch; choose opt-in metadata providers and save provider credentials without retrieving secret values.
- **Metadata provider transition:** IGDB is used instead of RAWG. New settings do not offer RAWG, legacy saved RAWG selections are discarded when settings load/sanitize, and a legacy RAWG enum value cannot trigger a network request.
- **Regression checks:** frontend tests cover startup loading, filters, action eligibility, and catalog interpolation (11 tests pass); frontend type checks and build pass. All 62 backend unit tests pass, including provider parsing, key status, and RAWG retirement/migration; Rust formatting passes. Desktop/provider runtime behavior remains unverified; see [TODO.md](TODO.md).

## ✅ Later — prepare for release

- Exercise scanning, persistence, and launch flows against real client data on each target operating system.
- Expand regression tests for settings, metadata provider opt-in, credentials, and important frontend/backend command boundaries.
- Review accessibility, privacy, error handling, installer packaging, and documentation before describing the project as release-ready.

## 🔄 Keeping this roadmap current

Update this roadmap when work is verified. Move confirmed defects to [BUGS.md](BUGS.md), and keep immediate next actions in [TODO.md](TODO.md). Do not mark planned or source-inferred behavior as shipped or runtime-tested.
