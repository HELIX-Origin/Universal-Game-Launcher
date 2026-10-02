import { describe, expect, it } from "vitest";
import type {
  ApiKeyStatus,
  CommandError,
  GameEntry,
  GameMetadata,
  LaunchTarget,
  LibrarySnapshot,
  PlatformStatus,
  Settings,
} from "$lib/library";
import apiKeyStatus from "./fixtures/api-key-status.json";
import errors from "./fixtures/errors.json";
import snapshot from "./fixtures/library-snapshot.json";
import platforms from "./fixtures/platforms.json";
import providers from "./fixtures/metadata-providers.json";
import settingsCustomized from "./fixtures/settings.customized.json";
import settingsDefault from "./fixtures/settings.default.json";
import requestSettings from "./fixtures/requests/update-settings.json";
import { arrayOf, boolean, extraKeys, literal, nullable, number, object, string, valuesAt, type Check } from "./shape";
import { stringUnion } from "./sources";

const platformIds = stringUnion("lib/library.ts", "Platform");
const platform = literal(...platformIds);

const launchTarget = object<LaunchTarget>({ kind: literal("uri", "executable") });

const gameMetadata = object<GameMetadata>({
  description: nullable(string),
  developer: nullable(string),
  publisher: nullable(string),
  releaseDate: nullable(string),
  genres: arrayOf(string),
  rating: nullable(number),
  coverUrl: nullable(string),
  heroUrl: nullable(string),
  sources: arrayOf(literal(...providers.all)),
  fetchedAt: number,
});

const gameEntry = object<GameEntry>({
  id: string,
  title: string,
  platform,
  installed: boolean,
  install: nullable(launchTarget),
  coverUrl: nullable(string),
  heroUrl: nullable(string),
  installDir: nullable(string),
  metadata: nullable(gameMetadata),
  favorite: boolean,
  hidden: boolean,
  custom: boolean,
});

const platformStatus = object<PlatformStatus>({
  platform,
  gameCount: number,
  error: nullable(string),
});

const librarySnapshot = object<LibrarySnapshot>({
  games: arrayOf(gameEntry),
  platforms: arrayOf(platformStatus),
  scannedAt: number,
});

const settings = object<Settings>({
  disabledPlatforms: arrayOf(platform),
  steamPath: nullable(string),
  theme: string,
  viewMode: string,
  minimizeOnLaunch: boolean,
  locale: nullable(string),
  metadataProviders: arrayOf(literal(...providers.all)),
});

const keyStatus = object<ApiKeyStatus>({
  steamgriddb: boolean,
  igdbClientId: boolean,
  igdbClientSecret: boolean,
  vndb: boolean,
  rawg: boolean,
});

const commandError = object<CommandError>({
  code: literal(...errors.codes),
  detail: nullable(string),
});

/** Keys of GameEntry, typed so new TypeScript fields must be listed here. */
const gameEntryKeys: Record<keyof GameEntry, true> = {
  id: true,
  title: true,
  platform: true,
  installed: true,
  install: true,
  coverUrl: true,
  heroUrl: true,
  installDir: true,
  metadata: true,
  favorite: true,
  hidden: true,
  custom: true,
};

function validate(check: Check, value: unknown) {
  return check(value, "$");
}

describe("Rust response payloads match the TypeScript types", () => {
  it.each<[string, Check, unknown]>([
    ["get_library → LibrarySnapshot", librarySnapshot, snapshot],
    ["get_settings → Settings (defaults)", settings, settingsDefault],
    ["update_settings → Settings (customized)", settings, settingsCustomized],
    ["get_api_key_status → ApiKeyStatus", keyStatus, apiKeyStatus],
    ["command rejection → CommandError", commandError, errors.withoutDetail],
    ["command rejection with detail → CommandError", commandError, errors.withDetail],
  ])("%s", (_name, check, payload) => {
    expect(validate(check, payload)).toEqual([]);
  });

  it("covers both empty and filled optional game fields in the golden snapshot", () => {
    for (const field of ["install", "coverUrl", "heroUrl", "installDir", "metadata"]) {
      const values = valuesAt(snapshot, ["games", field]);
      expect(values, field).toContain(null);
      expect(values.some((value) => value !== null), field).toBe(true);
    }
    expect(valuesAt(snapshot, ["games", "custom"])).toContain(true);
    expect(valuesAt(snapshot, ["platforms", "error"]).some((value) => value !== null)).toBe(true);
  });

  it("records the extra fields Rust sends that the UI does not model yet", () => {
    expect(extraKeys(snapshot.games[0], Object.keys(gameEntryKeys))).toEqual([
      "lastPlayed",
      "launch",
      "playCount",
      "sizeBytes",
      "storeId",
    ]);
    expect(extraKeys(snapshot.platforms[0], ["platform", "gameCount", "error"])).toEqual([
      "canOpen",
      "canOpenStore",
      "enabled",
    ]);
  });

  it("the update_settings request fixture is a complete Settings value", () => {
    expect(validate(settings, requestSettings)).toEqual([]);
    expect(Object.keys(requestSettings).sort()).toEqual(Object.keys(settingsDefault).sort());
  });

  it("never returns secret values in key status", () => {
    expect(Object.values(apiKeyStatus).every((value) => typeof value === "boolean")).toBe(true);
  });
});

describe("shared identifiers", () => {
  it("TypeScript Platform union matches Rust platform keys in order", () => {
    expect(platformIds).toEqual(platforms.map(({ key }) => key));
    expect(platforms.every(({ id, key }) => id === key)).toBe(true);
  });

  it("cloud and user-managed platforms match the Add game options", () => {
    const userManaged = platforms.filter(({ userManaged }) => userManaged).map(({ key }) => key);
    expect(userManaged.sort()).toEqual(["geforce-now", "local", "xcloud"]);
  });
});
