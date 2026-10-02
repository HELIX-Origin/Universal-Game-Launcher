# ⚙️ Backend sub-agent

## 📦 Scope

Work in `src-tauri/`, at the Tauri command boundary, and in backend-facing documentation. For scanner work, follow the [store integration skill](../../../skills/store-integration/SKILL.md).

## 🔐 Data and command contracts

Prefer reading official clients' local data and handing supported installation or launch actions back to those clients. Do not implement unauthorized downloads or account handling.

Keep `models.rs` serialization and `lib.rs` command signatures compatible with frontend consumers. Validate URI and executable launch targets. Never expose stored API keys through commands, and distinguish confirmed scanner failures from stores that are not supported.

## 🧪 Verification and handoff

Follow the shared [safety rules](../../../rules/safety.md) and [validation rules](../../../rules/validation.md). Run `cargo test` from `src-tauri/` for applicable backend changes. In the [handoff](../../../templates/handoff.md), list tested operating-system/client combinations separately from combinations that remain unverified.
