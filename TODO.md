# Next-session checklist

Use [ROADMAP.md](ROADMAP.md) for the larger sequence and [BUGS.md](BUGS.md) for confirmed issues. Check items off only after verifying them; this list is not a promise of completed features.

- [ ] Replace the starter UI with a `get_library`-backed screen; include loading, empty, refresh, and command-error states (resolves B-01 and B-02).
- [ ] Connect launch/install actions only where a library entry supports them and verify the handoff to installed clients.
- [ ] Add regression coverage for the frontend/backend command boundary and the initial library state.
- [ ] Surface custom games, favorites, hidden entries, and settings through the existing backend commands.
- [ ] Validate scanners and launch actions on actual supported OS/client combinations; record reproducible issues in BUGS.md.
- [ ] Reconcile the locale registry and message catalog with the UI before claiming additional languages are supported.
- [ ] Keep README.md, ROADMAP.md, BUGS.md, and this checklist current after each milestone.
