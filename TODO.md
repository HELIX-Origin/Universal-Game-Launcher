# ✅ Next-session checklist

Use the [roadmap](ROADMAP.md) for broader milestones and [BUGS.md](BUGS.md) for confirmed issues. Check an item only after verifying the result; an unchecked item is planned work, not a delivered feature.

## 🎮 First library experience

- [x] Replace the starter UI with a screen backed by `get_library`.
- [x] Provide loading, empty, refresh, and command-error states; resolve B-01 and B-02 in [BUGS.md](BUGS.md).
- [x] Connect launch and install actions only when the game entry supports them.
- [x] Add frontend regression coverage for the initial loading state, library filters, and action eligibility.
- [ ] Smoke-test Tauri command invocation and launch/install handoffs to official clients on supported operating systems.

## ⚙️ Personal library and settings

- [x] Expose favorites and hidden-game management through the existing backend commands.
- [ ] Add UI flows for creating/removing custom games and editing launcher settings.
- [ ] Reconcile the locale registry and English message catalog with the UI before claiming additional languages are supported.

## 🧪 Verification and documentation

- [ ] Validate scanners and launch actions on actual supported operating-system/client combinations; record reproducible issues in [BUGS.md](BUGS.md).
- [ ] Keep [README.md](README.md), [ROADMAP.md](ROADMAP.md), [BUGS.md](BUGS.md), and this checklist aligned with verified behavior.
