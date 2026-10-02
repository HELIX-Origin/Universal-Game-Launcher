import { describe, expect, it } from "vitest";
import {
  declaredCommands,
  frontendInvocations,
  registeredCommands,
  stringUnion,
} from "./sources";

/**
 * Registered commands the UI does not call yet. Adding a command here is a
 * deliberate decision; remove it once the frontend uses it.
 */
const backendOnlyCommands = ["open_client", "clear_metadata", "get_app_info", "open_data_dir"];

/** Plugin commands invoked through official plugin wrappers, not app commands. */
const isPluginCommand = (command: string) => command.startsWith("plugin:");

const invocations = frontendInvocations({
  command: stringUnion("lib/library.ts", "GameActionCommand"),
});
const declared = new Map(declaredCommands().map((command) => [command.name, command.args]));
const registered = registeredCommands();

describe("Tauri command registry", () => {
  it("parses the backend and frontend sources", () => {
    expect(registered.length).toBeGreaterThan(0);
    expect(declared.size).toBeGreaterThan(0);
    expect(invocations.length).toBeGreaterThan(0);
  });

  it("registers every #[tauri::command] exactly once", () => {
    expect([...registered].sort()).toEqual([...declared.keys()].sort());
    expect(new Set(registered).size).toBe(registered.length);
  });

  it("only invokes commands that the backend registers", () => {
    const unknown = invocations.flatMap(({ file, line, commands }) =>
      commands
        .filter((command) => !isPluginCommand(command) && !registered.includes(command))
        .map((command) => `${file}:${line} → ${command}`),
    );
    expect(unknown).toEqual([]);
  });

  it("uses every registered command or lists it as backend-only", () => {
    const used = new Set(invocations.flatMap(({ commands }) => commands));
    const unused = registered.filter((command) => !used.has(command));
    expect(unused.sort()).toEqual([...backendOnlyCommands].sort());
  });

  it("keeps backend-only commands registered", () => {
    for (const command of backendOnlyCommands) expect(registered).toContain(command);
  });
});

describe("Tauri command arguments", () => {
  for (const invocation of invocations) {
    for (const command of invocation.commands.filter((name) => declared.has(name))) {
      it(`${command} (${invocation.file}:${invocation.line}) passes exactly the Rust parameters`, () => {
        const expected = [...declared.get(command)!].sort();
        expect([...(invocation.args ?? [])].sort()).toEqual(expected);
      });
    }
  }
});
