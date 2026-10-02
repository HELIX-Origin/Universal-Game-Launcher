# 🗺️ Roadmap

This roadmap describes intended work; it is **not** a claim that the user interface or integrations are complete. See the [README](README.md) for current project status and the [next-session checklist](TODO.md) for immediate tasks.

## 🎯 Now — deliver a usable first library

- Replace the starter page with a library view powered by `get_library`.
- Make loading, empty, refresh, and command-error states understandable to the user.
- Connect `launch_game`, `install_game`, and `open_client` only when an entry supports the corresponding action. Surface failures instead of silently ignoring them.
- Verify discovery and launch handoffs on supported operating systems with locally installed store clients. Record what was actually tested.

## 🧩 Next — personal library and settings

- Expose custom games, favorites, hidden entries, and install folders through the existing backend commands.
- Provide settings for enabled platforms, display choices, and optional metadata providers.
- Keep third-party credentials optional: browsing the local library must not depend on them.
- Connect the existing English message catalog to the UI. Add translated dictionaries only when the UI can select and use them.

## ✅ Later — prepare for release

- Exercise scanning, persistence, and launch flows against real client data on each target operating system.
- Add regression tests for reported defects and important frontend/backend command boundaries.
- Review accessibility, privacy, error handling, installer packaging, and documentation before describing the project as release-ready.

## 🔄 Keeping this roadmap current

Update this roadmap when work is verified. Move confirmed defects to [BUGS.md](BUGS.md), and keep immediate next actions in [TODO.md](TODO.md). Do not mark planned or source-inferred behavior as shipped or runtime-tested.
