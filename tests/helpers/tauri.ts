/**
 * In-memory Tauri IPC double. Every `invoke` from the code under test is
 * recorded and routed to a handler; unhandled commands reject like the real
 * backend does (`{ code, detail }`) so missing mocks surface as test failures.
 */
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import type { CommandError } from "$lib/library";

export type InvokeArgs = Record<string, unknown>;
export type CommandHandler = (args: InvokeArgs) => unknown;

export interface RecordedCall {
  command: string;
  args: InvokeArgs;
}

export class CommandRejection extends Error {
  constructor(public readonly payload: CommandError) {
    super(`${payload.code}${payload.detail ? `: ${payload.detail}` : ""}`);
  }
}

/** Throw from a handler to make `invoke` reject with a backend-style error. */
export function reject(code: string, detail: string | null = null): never {
  throw new CommandRejection({ code, detail });
}

export interface TauriMock {
  readonly calls: RecordedCall[];
  /** Arguments of every call to `command`, in order. */
  callsTo(command: string): InvokeArgs[];
  /** Replace or add a handler while a test is running. */
  on(command: string, handler: CommandHandler): void;
  /** Commands invoked without a handler (should normally stay empty). */
  readonly unhandled: string[];
  reset(): void;
}

export function mockTauri(handlers: Record<string, CommandHandler> = {}): TauriMock {
  const routes = new Map(Object.entries(handlers));
  const calls: RecordedCall[] = [];
  const unhandled: string[] = [];

  mockIPC(async (command, payload) => {
    const args = (payload ?? {}) as InvokeArgs;
    calls.push({ command, args });
    const handler = routes.get(command);
    if (!handler) {
      unhandled.push(command);
      throw { code: "internal", detail: `no mock for ${command}` } satisfies CommandError;
    }
    try {
      return await handler(args);
    } catch (error) {
      if (error instanceof CommandRejection) throw error.payload;
      throw error;
    }
  });

  return {
    calls,
    unhandled,
    callsTo: (command) => calls.filter((call) => call.command === command).map((call) => call.args),
    on: (command, handler) => void routes.set(command, handler),
    reset: () => {
      calls.length = 0;
      unhandled.length = 0;
    },
  };
}

export { clearMocks };

/** A promise whose settlement is controlled by the test (for pending-state checks). */
export function deferred<T = unknown>() {
  let resolve!: (value: T) => void;
  let rejectPromise!: (reason?: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    rejectPromise = rej;
  });
  return { promise, resolve, reject: rejectPromise };
}
