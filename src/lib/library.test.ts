import { describe, expect, it } from "vitest";
import { filterLibraryGames, gameActionCommand, type GameEntry, type Platform } from "./library";

const platformNames: Record<Platform, string> = {
  steam: "Steam",
  epic: "Epic Games",
  gog: "GOG",
  humble: "Humble",
  itch: "itch.io",
  ubisoft: "Ubisoft Connect",
  ea: "EA",
  origin: "Origin",
  xbox: "Xbox",
  amazon: "Amazon Games",
  "battle-net": "Battle.net",
  lutris: "Lutris",
  "geforce-now": "GeForce NOW",
  xcloud: "Xbox Cloud Gaming",
  local: "Local games",
};

function game(
  id: string,
  title: string,
  options: Partial<GameEntry> = {},
): GameEntry {
  return {
    id,
    title,
    platform: "steam",
    installed: true,
    install: null,
    coverUrl: null,
    heroUrl: null,
    installDir: null,
    metadata: null,
    favorite: false,
    hidden: false,
    custom: false,
    ...options,
  };
}

describe("filterLibraryGames", () => {
  const games = [
    game("steam:1", "Zulu"),
    game("epic:2", "Alpha", { platform: "epic", favorite: true }),
    game("steam:3", "Hidden", { hidden: true, favorite: true }),
    game("gog:4", "Waiting", {
      platform: "gog",
      installed: false,
      install: { kind: "uri" },
    }),
  ];

  it("returns visible games sorted by title by default", () => {
    expect(filterLibraryGames(games, "", "all", platformNames).map(({ title }) => title)).toEqual([
      "Alpha",
      "Waiting",
      "Zulu",
    ]);
  });

  it("filters installed games and favorites without showing hidden entries", () => {
    expect(
      filterLibraryGames(games, "", "installed", platformNames).map(({ title }) => title),
    ).toEqual(["Alpha", "Zulu"]);
    expect(
      filterLibraryGames(games, "", "favorites", platformNames).map(({ title }) => title),
    ).toEqual(["Alpha"]);
  });

  it("shows only hidden games in the hidden filter", () => {
    expect(
      filterLibraryGames(games, "", "hidden", platformNames).map(({ title }) => title),
    ).toEqual(["Hidden"]);
  });

  it("searches titles and platform names without case sensitivity", () => {
    expect(
      filterLibraryGames(games, "  EPIC ", "all", platformNames).map(({ title }) => title),
    ).toEqual(["Alpha"]);
    expect(
      filterLibraryGames(games, "gog", "all", platformNames).map(({ title }) => title),
    ).toEqual(["Waiting"]);
  });
});

describe("gameActionCommand", () => {
  it("launches installed games", () => {
    expect(gameActionCommand(game("steam:1", "Installed"))).toBe("launch_game");
  });

  it("installs owned uninstalled games when an install target is available", () => {
    expect(
      gameActionCommand(
        game("steam:2", "Ready to install", {
          installed: false,
          install: { kind: "uri" },
        }),
      ),
    ).toBe("install_game");
  });

  it("offers no action when an uninstalled game has no install target", () => {
    expect(gameActionCommand(game("steam:3", "Unavailable", { installed: false }))).toBeNull();
  });
});
