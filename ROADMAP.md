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
- [ ] Add settings for optional metadata providers and API-key status.
- Keep third-party credentials optional: browsing the local library must not depend on them.
- Connect the existing English message catalog to the UI. Add translated dictionaries only when the UI can select and use them.

## ✅ Later — prepare for release

- Exercise scanning, persistence, and launch flows against real client data on each target operating system.
- Add regression tests for reported defects and important frontend/backend command boundaries.
- Review accessibility, privacy, error handling, installer packaging, and documentation before describing the project as release-ready.

## 🔄 Keeping this roadmap current

Update this roadmap when work is verified. Move confirmed defects to [BUGS.md](BUGS.md), and keep immediate next actions in [TODO.md](TODO.md). Do not mark planned or source-inferred behavior as shipped or runtime-tested.
