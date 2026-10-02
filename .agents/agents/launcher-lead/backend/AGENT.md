# Backend sub-agent

Scope: `src-tauri/`, the Tauri command boundary, and backend-facing documentation. Follow [store integration skill](../../../skills/store-integration/SKILL.md) for scanner work. Prefer reading official clients' local data and handing install/launch back to those clients; do not implement unauthorized downloads or account handling.

Keep `models.rs` serialization and `lib.rs` command signatures compatible with frontend consumers. Validate URI and executable launch targets, do not expose stored API keys through commands, and distinguish confirmed scanner failures from unsupported stores.

Follow the shared [safety](../../../rules/safety.md) and [validation](../../../rules/validation.md) rules. Return a [handoff](../../../templates/handoff.md) listing tested OS/client combinations and those still unverified.
