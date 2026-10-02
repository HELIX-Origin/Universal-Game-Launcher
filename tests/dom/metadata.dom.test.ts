import { describe, expect, it } from "vitest";
import Page from "../../src/routes/+page.svelte";
import { alerts, click, gameCard, getButton, render, settle, textOf } from "../helpers/dom";
import { makeGame, makeMetadata, makeSnapshot } from "../helpers/fixtures";
import { deferred, mockTauri, reject } from "../helpers/tauri";

async function renderOneGame() {
  const tauri = mockTauri({
    get_library: () => makeSnapshot([makeGame("steam:1", "Solo"), makeGame("steam:2", "Duo")]),
  });
  render(Page);
  await settle();
  return tauri;
}

describe("game metadata", () => {
  it("fetches metadata with the default locale and renders it on the card", async () => {
    const tauri = await renderOneGame();
    tauri.on("fetch_metadata", () => makeMetadata({ developer: "Synthetic Studio", genres: ["Puzzle", "Indie"] }));

    await click(getButton("Fetch details", gameCard("Solo")));
    expect(tauri.callsTo("fetch_metadata")).toEqual([{ id: "steam:1", locale: "en" }]);
    expect(textOf(gameCard("Solo"))).toContain("Synthetic Studio");
    expect(textOf(gameCard("Solo"))).toContain("Puzzle · Indie");
    expect(textOf()).toContain("Metadata updated for Solo.");
  });

  it("allows only one metadata request at a time", async () => {
    const tauri = await renderOneGame();
    const pending = deferred();
    tauri.on("fetch_metadata", () => pending.promise);

    await click(getButton("Fetch details", gameCard("Solo")));
    expect(getButton(/Fetch/, gameCard("Duo")).disabled).toBe(true);
    pending.resolve(makeMetadata());
    await settle();
    expect(tauri.callsTo("fetch_metadata")).toHaveLength(1);
  });

  it.each([
    ["metadataNoProviders", "Enable a metadata provider in Settings first."],
    ["metadataNotFound", "No metadata was found for this game."],
    ["metadataRequestFailed", "The metadata service could not be reached."],
    ["unmapped", "Game metadata couldn't be fetched."],
  ])("maps %s failures to a readable message", async (code, message) => {
    const tauri = await renderOneGame();
    tauri.on("fetch_metadata", () => reject(code));
    await click(getButton("Fetch details", gameCard("Solo")));
    expect(alerts().join(" ")).toContain(message);
  });
});
