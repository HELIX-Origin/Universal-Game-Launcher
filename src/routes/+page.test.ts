import { render } from "svelte/server";
import { describe, expect, it } from "vitest";
import Page from "./+page.svelte";

describe("library page initial state", () => {
  it("shows a loading state while the first library snapshot is pending", () => {
    const { body } = render(Page);

    expect(body).toContain("Finding your games");
    expect(body).toContain("Checking your enabled libraries.");
    expect(body).not.toContain("Your library is ready for games");
  });
});
