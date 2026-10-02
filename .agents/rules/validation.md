# 🧪 Validation rules

Choose checks that match the files and behavior changed. Record exact commands, outcomes, and environmental blockers.

## 🔧 Existing project checks

- **Runner:** `npm run verify -- <suites|groups>` runs selected layers and prints a summary. A suite whose tool is missing is reported as unavailable, not passed. See [tests/README.md](../../tests/README.md).
- **Frontend:** Run `npm run verify -- frontend`. This runs `npm run check`, the unit, DOM, and contract Vitest projects, and `npm run build`.
- **Backend:** Run `npm run verify -- backend`. This runs `cargo fmt --check`, `cargo test --lib`, and `cargo test --test '*'` in `src-tauri/`. Add `rust-clippy` for lints.
- **Command or payload boundary:** Run `npm run verify -- boundary`. Regenerate golden payloads with `--update-contracts` only for intentional changes, and review the fixture diff.
- **Desktop behavior:** A desktop smoke test requires Tauri system prerequisites and relevant real client installations.
- **Documentation only:** Check referenced paths, links, and claims against the current tree; application builds are not required.

## 📋 Reporting evidence

Distinguish source inspection from runtime reproduction. For bugs, record reproduction steps, expected and actual behavior, relevant operating-system/client versions, and regression coverage where applicable. Scan changed files for secrets before committing. Document anything not verified rather than presenting it as a passed check.
