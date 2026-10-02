import { describe, expect, it } from "vitest";
import en from "$lib/i18n/locales/en";
import errors from "./fixtures/errors.json";
import providers from "./fixtures/metadata-providers.json";
import { frontendSource, pascalToCamel, pascalToKebab, rustEnumVariants, stringUnion } from "./sources";

const page = frontendSource("routes/+page.svelte");

/** `errors.*` catalog entries used for frontend-only failures. */
const frontendOnlyErrorMessages = ["unknown", "clientMissing"];

describe("error codes", () => {
  const rustCodes = rustEnumVariants("error.rs", "ErrorCode").map(pascalToCamel);

  it("golden error fixture lists every Rust ErrorCode variant", () => {
    expect(errors.codes).toEqual(rustCodes);
  });

  it("every backend error code has an English message", () => {
    const missing = rustCodes.filter((code) => !(code in en.errors));
    expect(missing).toEqual([]);
  });

  it("the errors catalog has no entries for removed codes", () => {
    const stale = Object.keys(en.errors).filter(
      (code) => !rustCodes.includes(code) && !frontendOnlyErrorMessages.includes(code),
    );
    expect(stale).toEqual([]);
  });

  it("codes the page handles explicitly exist in the backend", () => {
    const handled = new Set<string>();
    for (const [, code] of page.matchAll(/code === "(\w+)"/g)) handled.add(code);
    for (const block of page.matchAll(/\(\{\s*([\s\S]*?)\}\[code\]/g)) {
      for (const [, code] of block[1].matchAll(/(\w+):\s*t\(/g)) handled.add(code);
    }
    for (const [, code] of page.matchAll(/^\s*(\w+): t\("errors\.\w+"\),?$/gm)) handled.add(code);
    expect(handled.size).toBeGreaterThan(5);
    expect([...handled].filter((code) => !rustCodes.includes(code))).toEqual([]);
  });
});

describe("platform identifiers", () => {
  it("Rust Platform variants serialize to the TypeScript union values", () => {
    const rust = rustEnumVariants("models.rs", "Platform").map(pascalToKebab);
    expect(stringUnion("lib/library.ts", "Platform")).toEqual(rust);
  });
});

describe("metadata providers", () => {
  it("golden provider list matches the Rust enum", () => {
    expect(providers.all).toEqual(rustEnumVariants("metadata.rs", "MetadataProvider").map(pascalToCamel));
  });

  it("the settings UI offers exactly the selectable providers, never RAWG", () => {
    const block = /const metadataProviders = \[([\s\S]*?)\] as const;/.exec(page)?.[1] ?? "";
    const offered = [...block.matchAll(/id: "(\w+)"/g)].map(([, id]) => id);
    expect(offered).toEqual(providers.selectable);
    expect(offered).not.toContain("rawg");
  });
});
