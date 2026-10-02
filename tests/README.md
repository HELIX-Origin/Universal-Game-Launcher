# 🧪 Test suite

The suite has separate layers, so you can run only the checks that match a change. Every layer uses synthetic data. Tests never read real store clients, user libraries, or credentials, and never make network requests.

## 🚀 Running tests

`npm run verify` runs every default suite and prints a pass/fail summary. If a required tool is missing, the summary marks that suite **unavailable** and the command exits with a non-zero status.

```sh
npm run verify                         # all default suites
npm run verify -- quick                # unit + dom + contracts
npm run verify -- frontend             # types, unit, dom, contracts, build
npm run verify -- backend rust-clippy  # Rust fmt, unit, integration, clippy
npm run verify -- all --skip build --bail
npm run verify -- dom -- -t "settings" # args after `--` go verbatim to each selected tool
npm run verify -- --update-contracts boundary
npm run verify -- --list               # catalog of suites and groups
```

| Suite | Command | Covers |
| --- | --- | --- |
| `types` | `npm run check` | svelte-check over `src/` and `tests/`, including contract shape coverage |
| `unit` | `npm run test:unit` | Pure TypeScript logic, i18n, and SSR rendering (`src/**/*.test.ts`, `tests/unit/`) |
| `dom` | `npm run test:dom` | Interactive library page in jsdom with mocked Tauri IPC (`tests/dom/`) |
| `contracts` | `npm run test:contracts` | Frontend side of the Rust ↔ TypeScript boundary (`tests/contracts/`) |
| `build` | `npm run build` | Static SvelteKit build |
| `rust-fmt` | `cargo fmt --check` | Rust formatting |
| `rust-unit` | `cargo test --lib` | Module tests inside `src-tauri/src/` |
| `rust-integration` | `cargo test --test '*'` | `src-tauri/tests/`, including golden payload checks |
| `rust-contracts` *(opt-in)* | `cargo test --test contracts` | Only the Rust side of the payload contracts |
| `rust-clippy` *(opt-in)* | `cargo clippy --all-targets -- -D warnings` | Rust lints |

`npm test` runs every Vitest project, and `npm run test:watch` runs them in watch mode. `npm run test:rust` runs every backend test from the repository root. The Rust suites need the Tauri system prerequisites, such as WebKitGTK on Linux. CI runs the `frontend` group and `backend rust-clippy` from `.github/workflows/tests.yml`.

## 🗂️ Layout

```text
tests/
  helpers/            shared building blocks (import these, do not copy them)
    fixtures.ts       makeGame, makeSnapshot, makeSettings, makeApiKeyStatus, sampleLibrary, platformNames
    tauri.ts          mockTauri(): recorded IPC double; reject(code) for backend-style errors; deferred()
    dom.ts            render/cleanup, settle/waitFor, accessible-name queries, click/typeInto/toggle/submit
    dom-setup.ts      per-test cleanup for the dom project
  dom/                *.dom.test.ts component behavior tests
  contracts/
    sources.ts        raw-source parsers (registered commands, Rust params, invoke calls, enums)
    shape.ts          structural validator tied to TypeScript types
    *.test.ts         command, enum, and payload contracts
    fixtures/         golden JSON written by Rust; requests/ holds UI-shaped request payloads
  unit/               additional node-environment unit tests (optional)
src-tauri/tests/
  common/mod.rs       temp AppState, synthetic games, golden compare/update helpers
  contracts.rs        writes/compares golden payloads; deserializes request fixtures
  library_state.rs    AppState, persistence, settings, and credential integration flows
```

## 🔗 Contract tests

The contract tests catch Rust/TypeScript drift without starting the desktop app:

- **Commands:** Each `#[tauri::command]` is registered once. The frontend invokes only registered commands, and its argument keys must match the Rust parameter names in camelCase. Each registered command must either be called by the UI or appear in `backendOnlyCommands` in `commands.test.ts`.
- **Identifiers:** The `Platform` union, `ErrorCode` variants, `MetadataProvider` IDs, and the English `errors.*` catalog stay aligned. Every backend error code must have a message, and every code that the page handles must exist in Rust.
- **Responses:** Rust serializes representative payloads to `tests/contracts/fixtures/*.json`. `payloads.test.ts` validates them with `object<T>()` specs. Adding a field to a TypeScript type without updating the spec fails `npm run check`. Removing a field from Rust that the UI reads fails the contract test.
- **Requests:** `fixtures/requests/*.json` are the exact payloads that the DOM tests assert the page sends. `src-tauri/tests/contracts.rs` deserializes them into the Rust argument types.

If a payload change is intentional, run `npm run verify -- --update-contracts boundary`. Then update `src/lib/library.ts` and review the fixture diff before committing.

## ✍️ Writing tests

- **Pure logic:** Put tests next to the source as `src/lib/<name>.test.ts`.
- **UI behavior:** Add `tests/dom/<area>.dom.test.ts`. Use `mockTauri({ command: handler })` and `render(Page)`, then query by accessible name with `getButton`, `getField`, or `getDialog`. Use `tauri.callsTo(...)` to assert IPC calls. A command without a handler rejects and is recorded in `tauri.unhandled`.
- **Backend behavior:** Put module-private behavior in `#[cfg(test)]` modules. Put cross-module flows in `src-tauri/tests/*.rs` with `mod common;`.
- **New Tauri command:** Register the command. The contract tests then require a frontend call with matching arguments or a `backendOnlyCommands` entry.

## 🐞 Debugging

- `npx vitest run --project dom tests/dom/settings-dialog.dom.test.ts -t "credentials"` runs one file or test.
- `npx vitest --project dom` reruns tests on save. Add `--inspect-brk --no-file-parallelism` to attach a debugger.
- `getButton` failures list every accessible button name. Add `console.log(document.body.innerHTML)` to inspect the rendered DOM.
- `cargo test --test library_state -- --nocapture` shows Rust `println!` output. `RUST_BACKTRACE=1` adds panic backtraces.
- Golden mismatches print the differing JSON and the command to regenerate fixtures.

## ⚠️ Not covered

Desktop smoke tests, real store client scanning, launch or install handoffs to official clients, and live metadata provider responses need real environments. These checks remain manual; see `TODO.md`.
