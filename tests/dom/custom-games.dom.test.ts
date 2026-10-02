import { describe, expect, it } from "vitest";
import Page from "../../src/routes/+page.svelte";
import type { GameEntry } from "$lib/library";
import { alerts, click, gameCard, gameTitles, getButton, getDialog, getField, render, settle, submit, textOf, typeInto } from "../helpers/dom";
import { makeGame, makeSnapshot, sampleLibrary } from "../helpers/fixtures";
import { mockTauri, reject } from "../helpers/tauri";
// Request fixtures are also deserialized by `src-tauri/tests/contracts.rs`.
import localRequest from "../contracts/fixtures/requests/add-custom-game.local.json";
import cloudRequest from "../contracts/fixtures/requests/add-custom-game.cloud.json";

interface AddInput {
  input: {
    platform: string;
    title: string;
    executable: string | null;
    args: string[];
    url: string | null;
    coverUrl: string | null;
  };
}

async function openAddGame() {
  const tauri = mockTauri({ get_library: () => makeSnapshot(sampleLibrary()) });
  render(Page);
  await settle();
  await click(getButton("Add game"));
  return { tauri, dialog: getDialog("Add a game") };
}

describe("add game dialog", () => {
  it("adds a local executable chosen through the file dialog", async () => {
    const { tauri, dialog } = await openAddGame();
    tauri.on("plugin:dialog|open", () => "/synthetic/bin/game");
    tauri.on("add_custom_game", ({ input }) =>
      makeGame("local:new", (input as AddInput["input"]).title, { platform: "local", custom: true }),
    );

    await typeInto(getField("Name", dialog), "Homebrew");
    await click(getButton("Browse…", dialog));
    await typeInto(getField(/^Launch arguments/, dialog), "  --windowed   --fps 60 ");
    await submit(dialog.querySelector("form")!);

    expect(tauri.callsTo("add_custom_game")).toEqual([{ input: localRequest }]);
    expect(document.querySelectorAll("dialog")).toHaveLength(0);
    expect(gameTitles()).toContain("Homebrew");
    expect(textOf()).toContain("Homebrew was added to your library.");
  });

  it("sends cloud shortcuts as URLs without executable data", async () => {
    const { tauri, dialog } = await openAddGame();
    tauri.on("add_custom_game", ({ input }) =>
      makeGame("xcloud:new", (input as AddInput["input"]).title, { platform: "xcloud", custom: true }),
    );

    await typeInto(dialog.querySelector("select")!, "xcloud");
    await typeInto(getField("Name", dialog), "Streamed");
    await typeInto(getField("Game link", dialog), "https://www.xbox.com/play/synthetic");
    await typeInto(getField(/^Cover image URL/, dialog), "https://example.com/cover.png");
    await submit(dialog.querySelector("form")!);

    const [{ input }] = tauri.callsTo("add_custom_game") as unknown as AddInput[];
    expect(input).toEqual(cloudRequest);
  });

  it.each([
    ["titleRequired", "Please enter a name."],
    ["urlHostNotAllowed", "This link does not belong to the selected service."],
    ["executableNotFound", "The executable could not be found."],
    ["somethingUnexpected", "The game couldn't be added."],
  ])("maps backend validation error %s to a dialog message", async (code, message) => {
    const { tauri, dialog } = await openAddGame();
    tauri.on("add_custom_game", () => reject(code));
    await typeInto(getField("Name", dialog), "Anything");
    await submit(dialog.querySelector("form")!);

    expect(getDialog("Add a game")).toBeDefined();
    expect(alerts().join(" ")).toContain(message);
  });
});

describe("remove custom game", () => {
  async function renderWithCustomGame(removeResult: () => unknown) {
    const custom: GameEntry = makeGame("local:mine", "Mine", { platform: "local", custom: true });
    const tauri = mockTauri({
      get_library: () => makeSnapshot([custom, makeGame("steam:1", "Store Game")]),
      remove_custom_game: removeResult,
    });
    render(Page);
    await settle();
    return tauri;
  }

  it("only offers removal for custom entries", async () => {
    await renderWithCustomGame(() => true);
    expect(() => getButton("Remove Store Game from library")).toThrow();
    expect(getButton("Remove Mine from library", gameCard("Mine"))).toBeDefined();
  });

  it("confirms before removing and updates the library", async () => {
    const tauri = await renderWithCustomGame(() => true);
    await click(getButton("Remove Mine from library"));
    const dialog = document.querySelector<HTMLElement>('dialog[role="alertdialog"]')!;
    expect(textOf(dialog)).toContain("Files on disk are not deleted.");

    await click(getButton("Remove game", dialog));
    expect(tauri.callsTo("remove_custom_game")).toEqual([{ id: "local:mine" }]);
    expect(gameTitles()).toEqual(["Store Game"]);
    expect(textOf()).toContain("Mine was removed from your library.");
  });

  it("does not call the backend when removal is cancelled", async () => {
    const tauri = await renderWithCustomGame(() => true);
    await click(getButton("Remove Mine from library"));
    await click(getButton("Cancel", document.querySelector('dialog[role="alertdialog"]')!));
    expect(tauri.callsTo("remove_custom_game")).toEqual([]);
    expect(gameTitles()).toContain("Mine");
  });

  it("reports an error when the backend did not remove the entry", async () => {
    await renderWithCustomGame(() => false);
    await click(getButton("Remove Mine from library"));
    await click(getButton("Remove game"));
    expect(alerts().join(" ")).toContain("The game couldn't be removed.");
    expect(gameTitles()).toContain("Mine");
  });
});
