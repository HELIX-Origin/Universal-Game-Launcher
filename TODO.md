# ✅ Next-session checklist

Use the [roadmap](ROADMAP.md) for broader milestones and [BUGS.md](BUGS.md) for confirmed issues. Check an item only after verifying the result; an unchecked item is planned work, not a delivered feature.

## 🎮 First library experience — implementation complete; desktop verification open

- [x] Replace the starter UI with a screen backed by `get_library`.
- [x] Provide loading, empty, refresh, and command-error states; resolve B-01 and B-02 in [BUGS.md](BUGS.md).
- [x] Connect launch and install actions only when the game entry supports them.
- [x] Add frontend regression coverage for the initial loading state, library filters, and action eligibility.
- [ ] Smoke-test Tauri command invocation and launch/install handoffs to official clients on supported operating systems.

## ⚙️ Personal library and settings — core controls implemented

- [x] Expose favorites and hidden-game management through the existing backend commands.
- [x] Add/remove local games and GeForce NOW/Xbox Cloud shortcuts; edit enabled libraries and minimize-on-launch preference.
- [x] Open a game's install folder when the backend provides a valid location.
- [x] Select optional metadata providers and save/check write-only provider credentials.
- [x] Request and display metadata for a selected game.
- [x] Remove RAWG from selectable providers and prevent legacy settings from making RAWG network requests; keep legacy fields readable for old user data.
- [x] Wire the English source catalog through the library UI and clarify that locale tags do not mean translated UI dictionaries are bundled.
- [ ] Add translated dictionaries and a language selector together before claiming additional UI languages are supported.

## 🧪 Verification still required

- [x] Run frontend `npm test` (3 files, 11 tests passed), `npm run check` (0 errors, 0 warnings), and `npm run build` after catalog integration.
- [x] Run `cargo test` from `src-tauri/` (62 passed, 0 failed) after installing the Tauri GTK/WebKit system prerequisites.
- [x] Run `cargo fmt --check` from `src-tauri/`.
- [x] Run `npm run tauri build`; Linux `.deb`, `.rpm`, and `.AppImage` bundles were produced. Installer installation and runtime smoke tests remain pending.
- [ ] Verify metadata providers and credentials in the desktop app without using real credentials in logs or fixtures.
- [x] Record successful execution of the RAWG-retirement and legacy-settings migration tests in [BUGS.md](BUGS.md).
- [ ] Validate scanner behavior with synthetic or explicitly authorized data, and launch handoffs on supported operating-system/client combinations; record reproducible issues in [BUGS.md](BUGS.md).
- [x] Keep [README.md](README.md), [ROADMAP.md](ROADMAP.md), [BUGS.md](BUGS.md), and this checklist aligned with verified behavior.

## 🗂️ Implemented work record

- Initial library UI and capability-aware launch/install.
- Search, installed/favorite/hidden filters, refresh, scan warnings, and command error states.
- Favorite and hidden-state persistence; local/cloud custom-game add/remove; platform enablement; minimize-on-launch; install-folder opening.
- Optional metadata-provider selection, write-only key management, and per-game metadata fetch/rendering.
- RAWG API calls retired; IGDB retained as the replacement. Legacy RAWG configuration is not selectable or used for requests.
- English UI messages now resolve through a typed catalog helper with placeholder interpolation; no translated UI dictionaries or locale selector are provided yet.
- Backend unit tests passed, including RAWG request rejection and settings migration; tests used isolated fixtures and did not inspect real user store data.
- Linux desktop packaging succeeds on Ubuntu 24.04 and produces `.deb`, `.rpm`, and `.AppImage` bundles; package install/launch behavior has not been tested.

Only mark a verification item complete after recording its exact command or desktop environment/result. Desktop validation is still blocked: this environment has no graphical session (`DISPLAY`/`WAYLAND_DISPLAY`) or installed Steam, Lutris, Heroic, or Flatpak clients. Windows/macOS packaging must be checked in native environments. No real store data was scanned.

## 📦 Release readiness still open

- [ ] Add translated UI catalogs together with a selector; no non-English UI is currently shipped.
- [ ] Add frontend coverage for interactive settings and Tauri command/payload boundaries beyond the existing library and catalog tests.
- [ ] Install and smoke-test Linux packages in a graphical environment; test Windows and macOS packages in their native environments.
- [ ] Verify metadata key saving/status and live provider responses using disposable credentials supplied for testing.
- [ ] Review accessibility, privacy, credential/error handling, dependency advisories, and packaging behavior before describing the app as release-ready.
