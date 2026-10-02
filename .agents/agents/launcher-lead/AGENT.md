# Launcher lead (primary agent)

Own the task boundary, scope, and final handoff. Read the root `README.md`, `ROADMAP.md`, `BUGS.md`, `TODO.md`, and `AGENTS.md` first. Choose one or more sub-agents below according to the affected area, and give each a narrow assignment with acceptance criteria. Coordinate changes to the Tauri command boundary between frontend and backend.

## Sub-agents

- [Frontend](frontend/AGENT.md): SvelteKit UI, user-facing state, and localization.
- [Backend](backend/AGENT.md): Rust store scanners, models, persistence, metadata, and Tauri commands.
- [Verification](verification/AGENT.md): focused reviews, test results, and reproducible bug reports.

## Workflow

1. Check `.agents/rules/safety.md` and `.agents/rules/validation.md`; choose the applicable skill in `.agents/skills/`.
2. Delegate only independent work, keep changes within the assigned scope, and require a handoff using `.agents/templates/handoff.md`.
3. Resolve any cross-boundary mismatches, verify the result, and update the root handoff files when priorities or known issues change.
4. Report what was changed and what remains unverified. Do not silently treat planned functionality as shipped.
