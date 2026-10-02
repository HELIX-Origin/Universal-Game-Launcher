/**
 * Synthetic data builders shared by every frontend test project.
 *
 * Never put real library contents, machine paths, or credentials in fixtures.
 */
import type {
  ApiKeyStatus,
  GameEntry,
  GameMetadata,
  LibrarySnapshot,
  Platform,
  PlatformStatus,
  Settings,
} from "$lib/library";

export const platformNames: Record<Platform, string> = {
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

export const allPlatforms = Object.keys(platformNames) as Platform[];

export function makeGame(id: string, title: string, options: Partial<GameEntry> = {}): GameEntry {
  const platform = options.platform ?? (id.slice(0, id.lastIndexOf(":")) as Platform);
  return {
    id,
    title,
    platform: allPlatforms.includes(platform) ? platform : "steam",
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

export function makeMetadata(options: Partial<GameMetadata> = {}): GameMetadata {
  return {
    description: "A synthetic game used in tests.",
    developer: "Test Developer",
    publisher: "Test Publisher",
    releaseDate: "2020-01-01",
    genres: ["Puzzle"],
    rating: 80,
    coverUrl: null,
    heroUrl: null,
    sources: ["steamStore"],
    fetchedAt: 1_700_000_000,
    ...options,
  };
}

export function makePlatformStatus(
  platform: Platform,
  options: Partial<PlatformStatus> = {},
): PlatformStatus {
  return { platform, gameCount: 0, error: null, ...options };
}

/** Builds a snapshot whose platform counts match the supplied games. */
export function makeSnapshot(
  games: GameEntry[] = [],
  options: Partial<Omit<LibrarySnapshot, "games">> = {},
): LibrarySnapshot {
  return {
    games,
    platforms:
      options.platforms ??
      allPlatforms.map((platform) =>
        makePlatformStatus(platform, {
          gameCount: games.filter((game) => game.platform === platform).length,
        }),
      ),
    scannedAt: options.scannedAt ?? 1_700_000_000,
  };
}

export function makeSettings(options: Partial<Settings> = {}): Settings {
  return {
    disabledPlatforms: [],
    steamPath: null,
    theme: "dark",
    viewMode: "grid",
    minimizeOnLaunch: false,
    locale: null,
    metadataProviders: [],
    ...options,
  };
}

export function makeApiKeyStatus(options: Partial<ApiKeyStatus> = {}): ApiKeyStatus {
  return {
    steamgriddb: false,
    igdbClientId: false,
    igdbClientSecret: false,
    vndb: false,
    rawg: false,
    ...options,
  };
}

/** A small mixed library covering installed, uninstalled, custom, hidden and favorite entries. */
export function sampleLibrary(): GameEntry[] {
  return [
    makeGame("steam:10", "Zulu Racer"),
    makeGame("epic:alpha", "Alpha Quest", { platform: "epic", favorite: true }),
    makeGame("steam:30", "Hidden Gem", { hidden: true }),
    makeGame("gog:40", "Waiting Room", {
      platform: "gog",
      installed: false,
      install: { kind: "uri" },
    }),
    makeGame("geforce-now:cloud-1", "Cloud Runner", {
      platform: "geforce-now",
      installed: false,
      custom: true,
    }),
  ];
}
