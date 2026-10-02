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
- [ ] Reconcile the locale registry and English message catalog with the UI before claiming additional languages are supported.

## 🧪 Verification still required

- [ ] Run frontend `npm test`, `npm run check`, and `npm run build`, plus backend `cargo test`, after metadata integration.
- [ ] Verify metadata providers and credentials in the desktop app without using real credentials in logs or fixtures.
- [ ] Validate scanners and launch actions on actual supported operating-system/client combinations; record reproducible issues in [BUGS.md](BUGS.md).
- [ ] Keep [README.md](README.md), [ROADMAP.md](ROADMAP.md), [BUGS.md](BUGS.md), and this checklist aligned with verified behavior.

## 🗂️ Implemented work record

- Initial library UI and capability-aware launch/install.
- Search, installed/favorite/hidden filters, refresh, scan warnings, and command error states.
- Favorite and hidden-state persistence; local/cloud custom-game add/remove; platform enablement; minimize-on-launch; install-folder opening.
- Optional metadata-provider selection, write-only key management, and per-game metadata fetch/rendering.
- RAWG API calls retired; IGDB retained as the replacement. Legacy RAWG configuration is not selectable or used for requests.

Only mark a verification item complete after recording its exact command or desktop environment/result.
