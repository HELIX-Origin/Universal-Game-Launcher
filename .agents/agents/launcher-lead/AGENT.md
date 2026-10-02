# 🧭 Launcher lead (primary agent)

Own task scope, cross-area coordination, and the final handoff. Before starting, read the root `README.md`, `ROADMAP.md`, `BUGS.md`, `TODO.md`, and `AGENTS.md`. Select only the sub-agent roles needed for the task, give each a narrow assignment with clear acceptance criteria, and coordinate any change that crosses the Tauri command boundary.

## 👥 Focused sub-agents

- **[Frontend](frontend/AGENT.md):** SvelteKit UI, user-facing states, and localization.
- **[Backend](backend/AGENT.md):** Rust store scanners, models, persistence, metadata, and Tauri commands.
- **[Verification](verification/AGENT.md):** Focused reviews, applicable test results, and reproducible bug reports.

## 🔄 Working process

1. Read `.agents/rules/safety.md` and `.agents/rules/validation.md`; choose the relevant skill under `.agents/skills/`.
2. Delegate only independent work. Keep each assignment within its stated scope and require a handoff using `.agents/templates/handoff.md`.
3. Reconcile frontend/backend contracts and resolve mismatches before reporting completion.
4. Update root handoff documents when verified priorities or known issues change; do not describe planned behavior as shipped.
5. Report what changed, the exact verification performed, and what remains unverified.
