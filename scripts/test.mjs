#!/usr/bin/env node
// Modular test runner. Selects suites or groups, runs them in order, and prints
// a summary that distinguishes passed, failed, and unavailable (missing tool) suites.
//
//   npm run verify                       # default: every suite except opt-in ones
//   npm run verify -- quick              # fast frontend feedback (unit, dom, contracts)
//   npm run verify -- backend rust-clippy
//   npm run verify -- dom -- -t "settings"   # args after `--` go to each suite's test tool
//   npm run verify -- --list
//
// See tests/README.md for the full suite catalog.
import { spawn, spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const tauriDir = join(root, "src-tauri");
const isWindows = process.platform === "win32";

/**
 * @typedef {object} Suite
 * @property {string} description
 * @property {string} tool executable that must be on PATH
 * @property {string[]} args
 * @property {string} [cwd]
 * @property {boolean} [passthrough] forward args given after `--`
 * @property {string[]} [passthroughPrefix] inserted before forwarded args (e.g. cargo's `--`)
 * @property {boolean} [optIn] excluded from the default selection
 */

/** @type {Record<string, Suite>} */
const suites = {
  types: {
    description: "Svelte/TypeScript type check (svelte-check), including test files",
    tool: "npm",
    args: ["run", "check"],
  },
  unit: {
    description: "Frontend unit tests: pure TypeScript logic and SSR rendering (Vitest, node)",
    tool: "npx",
    args: ["vitest", "run", "--project", "unit"],
    passthrough: true,
  },
  dom: {
    description: "Interactive component tests with mocked Tauri IPC (Vitest, jsdom)",
    tool: "npx",
    args: ["vitest", "run", "--project", "dom"],
    passthrough: true,
  },
  contracts: {
    description: "Rust <-> TypeScript contracts: commands, arguments, enums, golden payloads",
    tool: "npx",
    args: ["vitest", "run", "--project", "contracts"],
    passthrough: true,
  },
  build: {
    description: "Static SvelteKit build (vite build)",
    tool: "npm",
    args: ["run", "build"],
  },
  "rust-fmt": {
    description: "Rust formatting (cargo fmt --check)",
    tool: "cargo",
    args: ["fmt", "--check"],
    cwd: tauriDir,
  },
  "rust-unit": {
    description: "Backend unit tests inside src-tauri/src (cargo test --lib)",
    tool: "cargo",
    args: ["test", "--lib"],
    cwd: tauriDir,
    passthrough: true,
    passthroughPrefix: ["--"],
  },
  "rust-integration": {
    description: "Backend integration and payload contract tests in src-tauri/tests",
    tool: "cargo",
    args: ["test", "--test", "*"],
    cwd: tauriDir,
    passthrough: true,
    passthroughPrefix: ["--"],
  },
  "rust-contracts": {
    description: "Only the Rust side of the payload contracts (golden JSON)",
    tool: "cargo",
    args: ["test", "--test", "contracts"],
    cwd: tauriDir,
    passthrough: true,
    passthroughPrefix: ["--"],
    optIn: true,
  },
  "rust-clippy": {
    description: "Rust lints for all targets (cargo clippy -D warnings)",
    tool: "cargo",
    args: ["clippy", "--all-targets", "--", "-D", "warnings"],
    cwd: tauriDir,
    optIn: true,
  },
};

/** @type {Record<string, string[]>} */
const groups = {
  quick: ["unit", "dom", "contracts"],
  frontend: ["types", "unit", "dom", "contracts", "build"],
  backend: ["rust-fmt", "rust-unit", "rust-integration"],
  boundary: ["contracts", "rust-contracts"],
  all: Object.keys(suites).filter((name) => !suites[name].optIn),
};

function usage() {
  const pad = Math.max(...Object.keys({ ...suites, ...groups }).map((name) => name.length)) + 2;
  const lines = [
    "Usage: npm run verify -- [suites|groups...] [options] [-- tool args]",
    "",
    "Suites:",
    ...Object.entries(suites).map(
      ([name, suite]) => `  ${name.padEnd(pad)}${suite.description}${suite.optIn ? " (opt-in)" : ""}`,
    ),
    "",
    "Groups:",
    ...Object.entries(groups).map(([name, members]) => `  ${name.padEnd(pad)}${members.join(", ")}`),
    "",
    "Options:",
    "  --list               Show suites and groups",
    "  --skip a,b           Remove suites or groups from the selection",
    "  --bail               Stop after the first failing suite",
    "  --update-contracts   Regenerate golden payloads (UGL_UPDATE_CONTRACTS=1) before running",
    "  --dry-run            Print the commands without running them",
    "",
    "With no selection, runs the `all` group.",
  ];
  console.log(lines.join("\n"));
}

/** @param {string[]} names */
function expand(names) {
  /** @type {string[]} */
  const result = [];
  for (const name of names) {
    if (groups[name]) result.push(...groups[name]);
    else if (suites[name]) result.push(name);
    else {
      console.error(`Unknown suite or group: ${name}\n`);
      usage();
      process.exit(2);
    }
  }
  return [...new Set(result)];
}

/** @param {string[]} argv */
function parseArgs(argv) {
  const separator = argv.indexOf("--");
  const own = separator === -1 ? argv : argv.slice(0, separator);
  const forwarded = separator === -1 ? [] : argv.slice(separator + 1);
  const options = {
    selection: /** @type {string[]} */ ([]),
    skip: /** @type {string[]} */ ([]),
    bail: false,
    list: false,
    dryRun: false,
    updateContracts: false,
  };
  for (let i = 0; i < own.length; i += 1) {
    const arg = own[i];
    if (arg === "--list" || arg === "--help" || arg === "-h") options.list = true;
    else if (arg === "--bail") options.bail = true;
    else if (arg === "--dry-run") options.dryRun = true;
    else if (arg === "--update-contracts") options.updateContracts = true;
    else if (arg === "--skip") options.skip.push(...(own[++i] ?? "").split(","));
    else if (arg.startsWith("--skip=")) options.skip.push(...arg.slice(7).split(","));
    else if (arg.startsWith("-")) {
      console.error(`Unknown option: ${arg}\n`);
      usage();
      process.exit(2);
    } else options.selection.push(arg);
  }
  return { ...options, forwarded };
}

/** @param {string} tool */
function toolAvailable(tool) {
  const probe = spawnSync(tool, ["--version"], { stdio: "ignore", shell: isWindows });
  return probe.status === 0;
}

/**
 * @param {string} command
 * @param {string[]} args
 * @param {string} cwd
 * @param {Record<string, string>} [env]
 * @returns {Promise<number>}
 */
function run(command, args, cwd, env = {}) {
  return new Promise((resolve) => {
    const child = spawn(command, args, {
      cwd,
      env: { ...process.env, ...env },
      stdio: "inherit",
      shell: isWindows,
    });
    child.on("close", (code) => resolve(code ?? 1));
    child.on("error", () => resolve(127));
  });
}

const options = parseArgs(process.argv.slice(2));
if (options.list) {
  usage();
  process.exit(0);
}

const skipped = new Set(expand(options.skip.filter(Boolean)));
const selected = expand(options.selection.length ? options.selection : ["all"]).filter(
  (name) => !skipped.has(name),
);

/** @type {{ name: string, status: "passed" | "failed" | "unavailable", seconds: number }[]} */
const results = [];
/** @type {Map<string, boolean>} */
const availability = new Map();

if (options.updateContracts) {
  const args = ["test", "--test", "contracts"];
  console.log(`\n▶ Regenerating golden contract payloads: UGL_UPDATE_CONTRACTS=1 cargo ${args.join(" ")}`);
  if (!options.dryRun && (await run("cargo", args, tauriDir, { UGL_UPDATE_CONTRACTS: "1" })) !== 0) {
    console.error("Failed to regenerate contract payloads.");
    process.exit(1);
  }
}

for (const name of selected) {
  const suite = suites[name];
  const args = [...suite.args];
  if (suite.passthrough && options.forwarded.length) {
    args.push(...(suite.passthroughPrefix ?? []), ...options.forwarded);
  }
  const cwd = suite.cwd ?? root;
  console.log(`\n▶ ${name}: ${suite.tool} ${args.join(" ")}${cwd === root ? "" : "  (in src-tauri/)"}`);
  if (options.dryRun) continue;

  if (!availability.has(suite.tool)) availability.set(suite.tool, toolAvailable(suite.tool));
  if (!availability.get(suite.tool)) {
    console.log(`  ${suite.tool} is not available on PATH; suite not run.`);
    results.push({ name, status: "unavailable", seconds: 0 });
    if (options.bail) break;
    continue;
  }

  const started = Date.now();
  const code = await run(suite.tool, args, cwd);
  results.push({ name, status: code === 0 ? "passed" : "failed", seconds: (Date.now() - started) / 1000 });
  if (code !== 0 && options.bail) break;
}

if (options.dryRun) process.exit(0);

const icons = { passed: "✔", failed: "✖", unavailable: "⚠" };
console.log("\nSummary");
for (const { name, status, seconds } of results) {
  console.log(`  ${icons[status]} ${name.padEnd(18)} ${status.padEnd(12)} ${seconds.toFixed(1)}s`);
}
const notRun = selected.filter((name) => !results.some((result) => result.name === name));
if (notRun.length) console.log(`  - not run (--bail): ${notRun.join(", ")}`);

process.exit(results.every(({ status }) => status === "passed") && notRun.length === 0 ? 0 : 1);
