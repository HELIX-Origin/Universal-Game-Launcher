/**
 * Tiny structural validator. `object<T>()` requires a check for every key of
 * the TypeScript type, so `npm run check` fails when a type gains a field the
 * contract does not cover, and the runtime check fails when Rust stops
 * sending a field the frontend reads.
 */
export type Check = (value: unknown, path: string) => string[];

const describe = (value: unknown) => (value === null ? "null" : Array.isArray(value) ? "array" : typeof value);

const primitive =
  (type: "string" | "number" | "boolean"): Check =>
  (value, path) =>
    typeof value === type ? [] : [`${path}: expected ${type}, got ${describe(value)}`];

export const string = primitive("string");
export const number = primitive("number");
export const boolean = primitive("boolean");

export const literal =
  (...allowed: readonly string[]): Check =>
  (value, path) =>
    typeof value === "string" && allowed.includes(value)
      ? []
      : [`${path}: expected one of ${allowed.join(", ")}, got ${JSON.stringify(value)}`];

export const nullable =
  (check: Check): Check =>
  (value, path) =>
    value === null ? [] : check(value, path);

export const arrayOf =
  (check: Check): Check =>
  (value, path) =>
    Array.isArray(value)
      ? value.flatMap((item, index) => check(item, `${path}[${index}]`))
      : [`${path}: expected array, got ${describe(value)}`];

export function object<T>(spec: { [K in keyof T]-?: Check }): Check {
  return (value, path) => {
    if (typeof value !== "object" || value === null || Array.isArray(value)) {
      return [`${path}: expected object, got ${describe(value)}`];
    }
    const record = value as Record<string, unknown>;
    return Object.entries<Check>(spec).flatMap(([key, check]) =>
      key in record ? check(record[key], `${path}.${key}`) : [`${path}.${key}: missing (Rust does not send it)`],
    );
  };
}

/** Keys Rust sends that the TypeScript type does not declare (informational). */
export function extraKeys(value: unknown, declared: readonly string[]): string[] {
  if (typeof value !== "object" || value === null) return [];
  return Object.keys(value).filter((key) => !declared.includes(key)).sort();
}

/** Collect every value reachable at `path` segments (arrays are flattened). */
export function valuesAt(value: unknown, path: string[]): unknown[] {
  if (path.length === 0) return [value];
  if (Array.isArray(value)) return value.flatMap((item) => valuesAt(item, path));
  if (typeof value !== "object" || value === null) return [];
  const [head, ...rest] = path;
  return head in value ? valuesAt((value as Record<string, unknown>)[head], rest) : [];
}
