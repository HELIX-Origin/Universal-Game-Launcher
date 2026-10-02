# ✅ Next-session checklist

Use the [roadmap](ROADMAP.md) for broader milestones and [BUGS.md](BUGS.md) for confirmed issues. Check an item only after verifying the result; an unchecked item is planned work, not a delivered feature.

## 🎮 First library experience

- [ ] Replace the starter UI with a screen backed by `get_library`.
- [ ] Provide clear loading, empty, refresh, and command-error states; resolve B-01 and B-02 in [BUGS.md](BUGS.md).
- [ ] Connect launch and install actions only when the game entry supports them, and verify the handoff to the official client.
- [ ] Add regression coverage for the frontend/backend command boundary and initial library state.

## ⚙️ Personal library and settings

- [ ] Surface custom games, favorites, hidden entries, and settings through the existing backend commands.
- [ ] Reconcile the locale registry and English message catalog with the UI before claiming additional languages are supported.

## 🧪 Verification and documentation

- [ ] Validate scanners and launch actions on actual supported operating-system/client combinations; record reproducible issues in [BUGS.md](BUGS.md).
- [ ] Keep [README.md](README.md), [ROADMAP.md](ROADMAP.md), [BUGS.md](BUGS.md), and this checklist aligned with verified behavior.
