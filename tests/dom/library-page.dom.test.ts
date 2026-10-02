import { describe, expect, it } from "vitest";
import Page from "../../src/routes/+page.svelte";
import { alerts, click, gameTitles, getButton, queryButtons, render, settle, textOf, typeInto, waitFor } from "../helpers/dom";
import { makeGame, makeSnapshot, sampleLibrary } from "../helpers/fixtures";
import { deferred, mockTauri, reject } from "../helpers/tauri";

async function renderLibrary(games = sampleLibrary()) {
  const tauri = mockTauri({ get_library: () => makeSnapshot(games) });
  render(Page);
  await settle();
  return tauri;
}

describe("library page: loading lifecycle", () => {
  it("requests a non-refreshing snapshot on mount and shows a loading state until it resolves", async () => {
    const pending = deferred();
    const tauri = mockTauri({ get_library: () => pending.promise });
    render(Page);
    await settle();

    expect(textOf()).toContain("Finding your games");
    expect(tauri.callsTo("get_library")).toEqual([{ refresh: false }]);

    pending.resolve(makeSnapshot(sampleLibrary()));
    await settle();
    expect(textOf()).not.toContain("Finding your games");
    expect(gameTitles()).toEqual(["Alpha Quest", "Cloud Runner", "Waiting Room", "Zulu Racer"]);
  });

  it("shows a retryable error when the library cannot be loaded", async () => {
    let attempts = 0;
    const tauri = mockTauri({
      get_library: () => {
        attempts += 1;
        if (attempts === 1) reject("internal");
        return makeSnapshot([makeGame("steam:1", "Recovered")]);
      },
    });
    render(Page);
    await settle();

    expect(alerts().join(" ")).toContain("Library unavailable");
    await click(getButton("Try again"));
    expect(gameTitles()).toEqual(["Recovered"]);
    expect(tauri.callsTo("get_library")).toEqual([{ refresh: false }, { refresh: false }]);
  });

  it("shows the empty-library state and rescans on request", async () => {
    const tauri = await renderLibrary([]);
    expect(textOf()).toContain("Your library is ready for games");

    await click(getButton("Rescan libraries", document.querySelector(".empty-state")!));
    expect(tauri.callsTo("get_library").at(-1)).toEqual({ refresh: true });
  });

  it("warns when a platform scan reported an error", async () => {
    mockTauri({
      get_library: () =>
        makeSnapshot([], {
          platforms: [{ platform: "steam", gameCount: 0, error: "scan failed" }],
        }),
    });
    render(Page);
    await settle();
    expect(textOf()).toContain("Some libraries couldn't be scanned");
  });
});

describe("library page: browsing", () => {
  it("filters by installed, favorites and hidden tabs", async () => {
    await renderLibrary();
    const tab = (name: string) =>
      queryButtons(name).find((button) => button.hasAttribute("aria-pressed"))!;

    await click(tab("Installed"));
    expect(gameTitles()).toEqual(["Alpha Quest", "Zulu Racer"]);
    await click(tab("Favorites"));
    expect(gameTitles()).toEqual(["Alpha Quest"]);
    await click(tab("Hidden"));
    expect(gameTitles()).toEqual(["Hidden Gem"]);
    await click(tab("All games"));
    expect(gameTitles()).toHaveLength(4);
  });

  it("searches by title or platform and offers to clear filters when nothing matches", async () => {
    await renderLibrary();
    const search = document.querySelector<HTMLInputElement>('input[type="search"]')!;

    await typeInto(search, "epic");
    expect(gameTitles()).toEqual(["Alpha Quest"]);

    await typeInto(search, "no such game");
    expect(gameTitles()).toEqual([]);
    expect(textOf()).toContain("No games match");

    await click(getButton(/clear filters/i));
    expect(search.value).toBe("");
    expect(gameTitles()).toHaveLength(4);
  });
});

describe("library page: game actions", () => {
  it("launches installed games and reports the request", async () => {
    const tauri = await renderLibrary();
    tauri.on("launch_game", () => null);

    await click(getButton(/^Launch Zulu Racer/));
    expect(tauri.callsTo("launch_game")).toEqual([{ id: "steam:10" }]);
    expect(textOf()).toContain("Launch request sent for Zulu Racer.");
  });

  it("hands installs to the store client for owned, uninstalled games", async () => {
    const tauri = await renderLibrary();
    tauri.on("install_game", () => null);

    await click(getButton(/^Install Waiting Room/));
    expect(tauri.callsTo("install_game")).toEqual([{ id: "gog:40" }]);
    expect(tauri.callsTo("launch_game")).toEqual([]);
  });

  it("offers no launch or install action for entries without a target", async () => {
    await renderLibrary();
    const labels = queryButtons(/Cloud Runner/).map((button) => button.getAttribute("aria-label") ?? "");
    expect(labels.length).toBeGreaterThan(0);
    expect(labels.some((label) => /^(Launch|Install) /.test(label))).toBe(false);
  });

  it("translates backend error codes for failed launches", async () => {
    const tauri = await renderLibrary();
    tauri.on("launch_game", () => reject("gameNotFound"));

    await click(getButton(/^Launch Zulu Racer/));
    await waitFor(() => expect(alerts().join(" ")).toContain("That game is no longer in your library"));
  });

  it("falls back to a generic message for unknown error codes", async () => {
    const tauri = await renderLibrary();
    tauri.on("launch_game", () => reject("somethingNew"));

    await click(getButton(/^Launch Zulu Racer/));
    expect(alerts().join(" ")).toContain("Something went wrong.");
  });

  it("toggles favorites and persists the change", async () => {
    const tauri = await renderLibrary();
    tauri.on("set_favorite", () => null);

    await click(getButton("Add to favorites: Zulu Racer"));
    expect(tauri.callsTo("set_favorite")).toEqual([{ id: "steam:10", value: true }]);
    expect(getButton("Remove from favorites: Zulu Racer").getAttribute("aria-pressed")).toBe("true");
  });

  it("keeps the favorite state unchanged when saving fails", async () => {
    const tauri = await renderLibrary();
    tauri.on("set_favorite", () => reject("storage"));

    await click(getButton("Add to favorites: Zulu Racer"));
    expect(queryButtons("Add to favorites: Zulu Racer")).toHaveLength(1);
    expect(alerts()).not.toEqual([]);
  });

  it("hides games and moves them to the hidden filter", async () => {
    const tauri = await renderLibrary();
    tauri.on("set_hidden", () => null);

    await click(getButton(/^Hide.*: Zulu Racer$/));
    expect(tauri.callsTo("set_hidden")).toEqual([{ id: "steam:10", value: true }]);
    expect(gameTitles()).not.toContain("Zulu Racer");
  });

  it("opens the install folder only for games that report one", async () => {
    const tauri = await renderLibrary([
      makeGame("steam:1", "Alpha", { installDir: "/synthetic/games/alpha" }),
      makeGame("steam:2", "Beta"),
    ]);
    tauri.on("open_install_dir", () => reject("noInstallDir"));

    expect(queryButtons(/install folder.*Beta/i)).toHaveLength(0);
    await click(getButton(/install folder.*Alpha/i));
    expect(tauri.callsTo("open_install_dir")).toEqual([{ id: "steam:1" }]);
    expect(alerts().join(" ")).toContain("No install folder is available for this game.");
  });
});
