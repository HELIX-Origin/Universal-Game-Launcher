# Roadmap

This is a direction of travel, not a claim that the UI or integrations are finished. See [README.md](README.md) for the project overview and [TODO.md](TODO.md) for the next concrete steps.

## Now: usable first library

- Replace the starter page with a library view powered by `get_library`, including refresh, empty/loading/error states.
- Connect game actions (`launch_game`, `install_game`, `open_client`) to the entry's available capabilities and report failures clearly.
- Verify discovery and launching on supported operating systems using locally installed store clients.

## Next: personal library and settings

- Expose custom games, favorites, hidden entries, and install folders through the existing backend commands.
- Add settings for enabled platforms, display choices, and optional metadata sources; do not require third-party credentials to browse the local library.
- Wire up the existing English message catalog; add translated dictionaries only when the UI can select and use them.

## Later: release readiness

- Exercise scanner, persistence, and launch flows against real client data on each target OS, with regression tests for reported bugs.
- Review accessibility, privacy, error handling, installer packaging, and documentation before advertising a public release.

Update this file as work ships; move confirmed issues to [BUGS.md](BUGS.md) and track the immediate next actions in [TODO.md](TODO.md).
