import { describe, expect, it } from "vitest";
import Page from "../../src/routes/+page.svelte";
import type { Settings } from "$lib/library";
import { alerts, click, getButton, getDialog, getField, render, settle, submit, textOf, toggle, typeInto } from "../helpers/dom";
import { makeApiKeyStatus, makeSettings, makeSnapshot, sampleLibrary } from "../helpers/fixtures";
import { mockTauri, reject, type CommandHandler } from "../helpers/tauri";
// Request fixtures are also deserialized by `src-tauri/tests/contracts.rs`.
import apiKeysRequest from "../contracts/fixtures/requests/set-api-keys.json";
import settingsRequest from "../contracts/fixtures/requests/update-settings.json";

async function openSettings(overrides: Record<string, CommandHandler> = {}) {
  const tauri = mockTauri({
    get_library: () => makeSnapshot(sampleLibrary()),
    get_settings: () => makeSettings({ locale: "en", metadataProviders: ["steamStore", "rawg"] }),
    get_api_key_status: () => makeApiKeyStatus({ steamgriddb: true }),
    update_settings: ({ settings }) => settings,
    ...overrides,
  });
  render(Page);
  await settle();
  await click(getButton("Settings"));
  return { tauri, dialog: document.querySelector("dialog") as HTMLDialogElement };
}

function checkbox(dialog: HTMLElement, label: string) {
  return getField(label, dialog);
}

describe("settings dialog", () => {
  it("loads settings and key status together and never shows the retired RAWG provider as selected", async () => {
    const { tauri, dialog } = await openSettings();

    expect(tauri.callsTo("get_settings")).toHaveLength(1);
    expect(tauri.callsTo("get_api_key_status")).toHaveLength(1);
    expect(checkbox(dialog, "Steam Store").checked).toBe(true);
    expect(textOf(dialog)).toContain("SteamGridDB API key · Saved");
    const providers = [...dialog.querySelectorAll(".provider-option strong")].map((el) => textOf(el));
    expect(providers).toEqual(["SteamGridDB", "Steam Store", "IGDB", "VNDB"]);
  });

  it("closes the dialog when settings cannot be loaded", async () => {
    await openSettings({ get_settings: () => reject("storage") });
    expect(() => getDialog("Settings")).toThrow();
    expect(document.querySelectorAll("dialog")).toHaveLength(0);
  });

  // Known defect B-04 in BUGS.md: the message is stored in the closed dialog's
  // error slot, so nothing is shown. Remove `.fails` when it is fixed.
  it.fails("reports an error when settings cannot be loaded (B-04)", async () => {
    await openSettings({ get_settings: () => reject("storage") });
    expect(alerts().join(" ")).toContain("Settings couldn't be loaded.");
  });

  it("saves edited preferences, sanitized providers, and rescans the library", async () => {
    const { tauri, dialog } = await openSettings();

    await toggle(checkbox(dialog, "Steam"));
    await toggle(checkbox(dialog, "Minimize the launcher when a game starts"));
    await toggle(checkbox(dialog, "IGDB"));
    await submit(dialog.querySelector("form")!);

    const [{ settings }] = tauri.callsTo("update_settings") as unknown as { settings: Settings }[];
    expect(settings).toEqual(settingsRequest);
    expect(tauri.callsTo("get_library").at(-1)).toEqual({ refresh: true });
    expect(document.querySelectorAll("dialog")).toHaveLength(0);
    expect(textOf()).toContain("Settings saved.");
  });

  it("keeps the dialog open with an error when saving fails", async () => {
    const { dialog } = await openSettings({ update_settings: () => reject("storage") });
    await submit(dialog.querySelector("form")!);
    expect(getDialog("Settings")).toBeDefined();
    expect(alerts().join(" ")).toContain("Settings couldn't be saved.");
  });

  it("sends only changed credentials and clears drafts after saving", async () => {
    const { tauri, dialog } = await openSettings({
      set_api_keys: () => makeApiKeyStatus({ steamgriddb: false, vndb: true }),
    });

    const vndb = dialog.querySelector<HTMLInputElement>('input[placeholder="Enter a new token"]')!;
    await typeInto(vndb, "synthetic-token");
    await typeInto(
      dialog.querySelector<HTMLInputElement>('input[placeholder="Enter a new client ID"]')!,
      "synthetic-client-id",
    );
    const clearSteamGridDb = [...dialog.querySelectorAll(".clear-key")]
      .find((element) => textOf(element) === "Clear saved key")!
      .querySelector("input")!;
    await toggle(clearSteamGridDb);
    await click(getButton("Save keys", dialog));

    expect(tauri.callsTo("set_api_keys")).toEqual([{ keys: apiKeysRequest }]);
    expect(vndb.value).toBe("");
    expect(textOf(dialog)).toContain("VNDB API key · Saved");
    expect(textOf()).not.toContain("synthetic-token");
  });

  it("reports credential save failures without echoing the secret", async () => {
    const { dialog } = await openSettings({ set_api_keys: () => reject("storage") });
    const key = dialog.querySelector<HTMLInputElement>('input[placeholder="Enter a new key"]')!;
    await typeInto(key, "synthetic-secret");
    await click(getButton("Save keys", dialog));

    expect(alerts().join(" ")).toContain("Credentials couldn't be saved.");
    expect(alerts().join(" ")).not.toContain("synthetic-secret");
  });
});
