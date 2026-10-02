# 🧪 Validation rules

Choose checks that match the files and behavior changed. Record exact commands, outcomes, and environmental blockers.

## 🔧 Existing project checks

- **Frontend:** Run `npm run check` and `npm run build` from the repository root.
- **Backend:** Run `cargo test` from `src-tauri/`.
- **Desktop behavior:** A desktop smoke test requires Tauri system prerequisites and relevant real client installations.
- **Documentation only:** Check referenced paths, links, and claims against the current tree; application builds are not required.

## 📋 Reporting evidence

Distinguish source inspection from runtime reproduction. For bugs, record reproduction steps, expected and actual behavior, relevant operating-system/client versions, and regression coverage where applicable. Scan changed files for secrets before committing. Document anything not verified rather than presenting it as a passed check.
