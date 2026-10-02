/**
 * Raw source access and lightweight parsers used by contract tests to compare
 * the Rust backend with the TypeScript frontend without a running app.
 */

const rustSources = import.meta.glob<string>("/src-tauri/src/**/*.rs", {
  query: "?raw",
  import: "default",
  eager: true,
});

const frontendSources = import.meta.glob<string>("/src/**/*.{ts,svelte}", {
  query: "?raw",
  import: "default",
  eager: true,
});

export function rustSource(path: string): string {
  const source = rustSources[`/src-tauri/src/${path}`];
  if (source === undefined) throw new Error(`Rust source not found: ${path}`);
  return source;
}

export function frontendSource(path: string): string {
  const source = frontendSources[`/src/${path}`];
  if (source === undefined) throw new Error(`Frontend source not found: ${path}`);
  return source;
}

/** Production frontend files (tests excluded). */
export function productionFrontendFiles(): [string, string][] {
  return Object.entries(frontendSources).filter(([path]) => !/\.test\.ts$/.test(path));
}

export const snakeToCamel = (name: string) => name.replace(/_([a-z0-9])/g, (_, c: string) => c.toUpperCase());
export const pascalToCamel = (name: string) => name[0].toLowerCase() + name.slice(1);
export const pascalToKebab = (name: string) =>
  name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase();

/** Strip `//` line comments so commented-out code does not count. */
function stripLineComments(source: string) {
  return source.replace(/^\s*\/\/.*$/gm, "");
}

/** Commands registered in `tauri::generate_handler![...]`. */
export function registeredCommands(): string[] {
  const match = /generate_handler!\[([\s\S]*?)\]/.exec(stripLineComments(rustSource("lib.rs")));
  if (!match) throw new Error("generate_handler! not found in lib.rs");
  return match[1]
    .split(",")
    .map((name) => name.trim())
    .filter(Boolean);
}

export interface RustCommand {
  name: string;
  /** Frontend-visible argument names (camelCase), excluding injected Tauri types. */
  args: string[];
}

const injectedTypes = /^(?:State<|AppHandle\b|Window\b|WebviewWindow\b|tauri::)/;

/** Every `#[tauri::command]` function across the backend. */
export function declaredCommands(): RustCommand[] {
  const commands: RustCommand[] = [];
  for (const source of Object.values(rustSources)) {
    const pattern = /#\[tauri::command\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*\(([^)]*)\)/g;
    for (const [, name, params] of source.matchAll(pattern)) {
      const args = params
        .split(/,(?![^<]*>)/)
        .map((param) => param.trim())
        .filter(Boolean)
        .map((param) => {
          const [argName, ...type] = param.split(":");
          return { argName: argName.replace(/^mut\s+/, "").trim(), type: type.join(":").trim() };
        })
        .filter(({ type }) => !injectedTypes.test(type))
        .map(({ argName }) => snakeToCamel(argName));
      commands.push({ name, args });
    }
  }
  return commands;
}

export interface Invocation {
  file: string;
  /** Literal command names; several when the name comes from a typed union. */
  commands: string[];
  /** Top-level argument keys, or `null` when no argument object is passed. */
  args: string[] | null;
  line: number;
}

/** Union members of an exported string-literal type alias in `src/lib/*.ts`. */
export function stringUnion(file: string, typeName: string): string[] {
  const match = new RegExp(`type\\s+${typeName}\\s*=([^;]+);`).exec(frontendSource(file));
  if (!match) throw new Error(`type ${typeName} not found in ${file}`);
  return [...match[1].matchAll(/"([^"]+)"/g)].map(([, value]) => value);
}

function topLevelKeys(objectLiteral: string): string[] {
  const keys: string[] = [];
  let depth = 0;
  let token = "";
  const flush = () => {
    const key = /^\s*(?:\.\.\.)?["']?([A-Za-z_$][\w$]*)["']?/.exec(token)?.[1];
    if (key && !token.trim().startsWith("...")) keys.push(key);
    token = "";
  };
  for (const char of objectLiteral.slice(1, -1)) {
    if ("{[(".includes(char)) depth += 1;
    if ("}])".includes(char)) depth -= 1;
    if (char === "," && depth === 0) flush();
    else token += char;
  }
  flush();
  return keys;
}

function balancedFrom(source: string, start: number): string {
  let depth = 0;
  for (let i = start; i < source.length; i += 1) {
    if (source[i] === "{") depth += 1;
    if (source[i] === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(start, i + 1);
    }
  }
  throw new Error("unbalanced object literal");
}

/**
 * Finds `invoke(...)` calls in production frontend code. A command passed as a
 * variable is resolved through `commandVariables` (name → possible commands).
 */
export function frontendInvocations(commandVariables: Record<string, string[]> = {}): Invocation[] {
  const invocations: Invocation[] = [];
  for (const [file, source] of productionFrontendFiles()) {
    const pattern = /\binvoke(?:<[^>]*>)?\(\s*(?:"([^"]+)"|(\w+))\s*(,\s*)?/g;
    for (const match of source.matchAll(pattern)) {
      const [whole, literal, variable, hasArgs] = match;
      const commands = literal ? [literal] : (commandVariables[variable] ?? [`<unresolved ${variable}>`]);
      const after = match.index + whole.length;
      let args: string[] | null = null;
      if (hasArgs) {
        const rest = source.slice(after);
        args = rest.startsWith("{")
          ? topLevelKeys(balancedFrom(source, after))
          : [`<non-literal ${/^[\w.]+/.exec(rest)?.[0]}>`];
      }
      invocations.push({ file, commands, args, line: source.slice(0, match.index).split("\n").length });
    }
  }
  return invocations;
}

/** Variants of a Rust enum, in declaration order. */
export function rustEnumVariants(file: string, enumName: string): string[] {
  const match = new RegExp(`pub enum ${enumName}\\s*\\{([\\s\\S]*?)\\n\\}`).exec(rustSource(file));
  if (!match) throw new Error(`enum ${enumName} not found in ${file}`);
  return match[1]
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line && !line.startsWith("//") && !line.startsWith("#"))
    .map((line) => /^([A-Z]\w*)/.exec(line)?.[1])
    .filter((name): name is string => Boolean(name));
}
