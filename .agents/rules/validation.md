# Validation rules

- Validate affected behavior with existing tools; record exact commands, results, and environmental blockers.
- Frontend: `npm run check` and `npm run build` from the repository root. Backend: `cargo test` from `src-tauri/`. A desktop smoke test requires Tauri prerequisites and real client installations.
- Distinguish source inspection from runtime reproduction. For bugs, capture steps, expected/actual behavior, OS/client versions where relevant, and regression coverage.
- Scan changed files for secrets before committing. Document unverified areas rather than presenting them as passed checks.
- For documentation-only edits, check paths, links, and statements against the current tree; application builds are not required.
