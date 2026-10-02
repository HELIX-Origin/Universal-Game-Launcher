import { describe, expect, it } from "vitest";
import { t } from "./index";

describe("English message catalog", () => {
  it("resolves nested messages and interpolates values", () => {
    expect(t("page.addedNotice", { title: "Example" })).toBe(
      "Example was added to your library.",
    );
  });

  it("keeps unresolved placeholders visible for callers to diagnose", () => {
    expect(t("page.launchRequest")).toBe("Launch request sent for {title}.");
  });

  it("resolves existing catalog namespaces", () => {
    expect(t("nav.settings")).toBe("Settings");
    expect(t("errors.metadataNotFound")).toBe("No metadata was found for this game.");
  });
});
