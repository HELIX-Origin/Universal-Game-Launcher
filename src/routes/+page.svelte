<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n";
  import {
    filterLibraryGames,
    gameActionCommand,
    type GameEntry,
    type GameMetadata,
    type LibrarySnapshot,
    type Platform,
    type Filter,
    type Settings,
    type ApiKeyStatus,
    type ApiKeyField,
  } from "$lib/library";

    type CustomPlatform = "local" | "geforce-now" | "xcloud";
    type Dialog = "add" | "settings" | null;

    const metadataProviders = [
      { id: "steamGridDb", name: "SteamGridDB", hint: t("settings.providerHints.steamGridDb"), keyRequired: true },
      { id: "steamStore", name: "Steam Store", hint: t("settings.providerHints.steamStore"), keyRequired: false },
      { id: "igdb", name: "IGDB", hint: t("settings.providerHints.igdb"), keyRequired: true },
      { id: "vndb", name: "VNDB", hint: t("settings.providerHints.vndb"), keyRequired: false },
    ] as const;

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

  const errorMessages: Record<string, string> = {
    gameNotFound: t("errors.gameNotFound"),
    launchFailed: t("errors.launchFailed"),
    storage: t("errors.storage"),
  };

  let snapshot = $state<LibrarySnapshot | null>(null);
  let query = $state("");
  let filter = $state<Filter>("all");
  let loading = $state(true);
  let refreshing = $state(false);
  let loadError = $state("");
  let actionError = $state("");
  let notice = $state("");
  let activeGameId = $state("");
  let activeDialog = $state<Dialog>(null);
  let dialogError = $state("");
  let savingDialog = $state(false);
  let settings = $state<Settings | null>(null);
  let apiKeyStatus = $state<ApiKeyStatus | null>(null);
  let apiKeyDrafts = $state<Record<ApiKeyField, string>>({
    steamgriddb: "",
    igdbClientId: "",
    igdbClientSecret: "",
    vndb: "",
  });
  let clearApiKeys = $state<Record<ApiKeyField, boolean>>({
    steamgriddb: false,
    igdbClientId: false,
    igdbClientSecret: false,
    vndb: false,
  });
  let fetchingMetadataId = $state("");
  let newPlatform = $state<CustomPlatform>("local");
  let newTitle = $state("");
  let newExecutable = $state("");
  let newArgs = $state("");
  let newUrl = $state("");
  let newCoverUrl = $state("");
  let pendingRemoval = $state<GameEntry | null>(null);

  let visibleGames = $derived(
    filterLibraryGames(snapshot?.games ?? [], query, filter, platformNames),
  );

  onMount(() => {
    void loadLibrary();
  });

  async function loadLibrary(refresh = false) {
    if (refreshing) return;
    if (snapshot) {
      refreshing = true;
    } else {
      loading = true;
    }
    loadError = "";
    actionError = "";
    notice = "";

    try {
      snapshot = await invoke<LibrarySnapshot>("get_library", { refresh });
    } catch {
      loadError = t("page.libraryError");
    } finally {
      loading = false;
      refreshing = false;
    }
  }

  async function runGameAction(game: GameEntry) {
    const command = gameActionCommand(game);
    if (!command || activeGameId) return;

    activeGameId = game.id;
    actionError = "";
    notice = "";
    try {
      await invoke(command, { id: game.id });
      notice = command === "launch_game"
        ? t("page.launchRequest", { title: game.title })
        : t("page.installRequest", { platform: platformNames[game.platform] });
    } catch (error) {
      const code =
        typeof error === "object" && error !== null && "code" in error
          ? String(error.code)
          : "";
      actionError = errorMessages[code] ?? t("errors.internal");
    } finally {
      activeGameId = "";
    }
  }

  async function toggleFavorite(game: GameEntry) {
    if (activeGameId) return;

    activeGameId = game.id;
    actionError = "";
    notice = "";
    try {
      await invoke("set_favorite", { id: game.id, value: !game.favorite });
      if (snapshot) {
        snapshot = {
          ...snapshot,
          games: snapshot.games.map((entry) =>
            entry.id === game.id ? { ...entry, favorite: !entry.favorite } : entry,
          ),
        };
      }
    } catch {
      actionError = t("page.favoriteSavedError");
    } finally {
      activeGameId = "";
    }
  }

  async function toggleHidden(game: GameEntry) {
    if (activeGameId) return;

    activeGameId = game.id;
    actionError = "";
    notice = "";
    try {
      await invoke("set_hidden", { id: game.id, value: !game.hidden });
      if (snapshot) {
        snapshot = {
          ...snapshot,
          games: snapshot.games.map((entry) =>
            entry.id === game.id ? { ...entry, hidden: !entry.hidden } : entry,
          ),
        };
      }
    } catch {
      actionError = t("page.visibilitySavedError");
    } finally {
      activeGameId = "";
    }
  }

  async function openInstallFolder(game: GameEntry) {
    actionError = "";
    try {
      await invoke("open_install_dir", { id: game.id });
    } catch (error) {
      const code =
        typeof error === "object" && error !== null && "code" in error
          ? String(error.code)
          : "";
      actionError =
        errorMessages[code] ??
        (code === "noInstallDir"
          ? t("page.installFolderUnavailable")
          : t("page.installFolderOpenError"));
    }
  }

  function openAddGame() {
    newPlatform = "local";
    newTitle = "";
    newExecutable = "";
    newArgs = "";
    newUrl = "";
    newCoverUrl = "";
    dialogError = "";
    activeDialog = "add";
  }

  async function chooseExecutable() {
    const selected = await openFileDialog({
      directory: false,
      multiple: false,
    });
    if (typeof selected === "string") newExecutable = selected;
  }

  async function addCustomGame(event: SubmitEvent) {
    event.preventDefault();
    if (savingDialog) return;
    savingDialog = true;
    dialogError = "";
    try {
      const added = await invoke<GameEntry>("add_custom_game", {
        input: {
          platform: newPlatform,
          title: newTitle,
          executable: newPlatform === "local" ? newExecutable : null,
          args: newPlatform === "local" ? newArgs.trim().split(/\s+/).filter(Boolean) : [],
          url: newPlatform === "local" ? null : newUrl,
          coverUrl: newCoverUrl || null,
        },
      });
      if (snapshot) snapshot = { ...snapshot, games: [...snapshot.games, added] };
      activeDialog = null;
      notice = t("page.addedNotice", { title: added.title });
    } catch (error) {
      const code =
        typeof error === "object" && error !== null && "code" in error
          ? String(error.code)
          : "";
      dialogError =
        errorMessages[code] ??
        ({
          titleRequired: t("errors.titleRequired"),
          titleTooLong: t("errors.titleTooLong"),
          executableRequired: t("errors.executableRequired"),
          executableNotAbsolute: t("errors.executableNotAbsolute"),
          executableNotFound: t("errors.executableNotFound"),
          urlRequired: t("errors.urlRequired"),
          urlNotHttps: t("errors.urlNotHttps"),
          urlHostNotAllowed: t("errors.urlHostNotAllowed"),
          invalidCoverUrl: t("errors.invalidCoverUrl"),
          unsupportedPlatform: t("errors.unsupportedPlatform"),
        }[code] ?? t("page.addGameError"));
    } finally {
      savingDialog = false;
    }
  }

  async function removeCustomGame() {
    if (!pendingRemoval || savingDialog) return;
    savingDialog = true;
    dialogError = "";
    try {
      const removed = await invoke<boolean>("remove_custom_game", { id: pendingRemoval.id });
      if (!removed) throw new Error(t("page.gameNotFound"));
      if (snapshot) {
        snapshot = {
          ...snapshot,
          games: snapshot.games.filter((game) => game.id !== pendingRemoval?.id),
        };
      }
      notice = t("page.removedNotice", { title: pendingRemoval.title });
      pendingRemoval = null;
    } catch {
      dialogError = t("page.gameRemovalError");
    } finally {
      savingDialog = false;
    }
  }

  async function openSettings() {
    dialogError = "";
    activeDialog = "settings";
    savingDialog = true;
    try {
      const [loadedSettings, keyStatus] = await Promise.all([
        invoke<Settings>("get_settings"),
        invoke<ApiKeyStatus>("get_api_key_status"),
      ]);
      settings = {
        ...loadedSettings,
        metadataProviders: loadedSettings.metadataProviders.filter((provider) => provider !== "rawg"),
      };
      apiKeyStatus = keyStatus;
    } catch {
      dialogError = t("page.settingsLoadError");
      activeDialog = null;
    } finally {
      savingDialog = false;
    }
  }

  function toggleMetadataProvider(provider: string) {
    if (!settings) return;
    const selected = new Set(settings.metadataProviders);
    if (selected.has(provider)) selected.delete(provider);
    else selected.add(provider);
    settings = {
      ...settings,
      metadataProviders: metadataProviders
        .map(({ id }) => id)
        .filter((id) => selected.has(id)),
    };
  }

  async function saveApiKeys() {
    if (savingDialog) return;
    savingDialog = true;
    dialogError = "";
    const keys: Partial<Record<ApiKeyField, string>> = {};
    for (const field of Object.keys(apiKeyDrafts) as ApiKeyField[]) {
      if (apiKeyDrafts[field].trim()) keys[field] = apiKeyDrafts[field];
      else if (clearApiKeys[field]) keys[field] = "";
    }
    try {
      apiKeyStatus = await invoke<ApiKeyStatus>("set_api_keys", { keys });
      apiKeyDrafts = {
        steamgriddb: "",
        igdbClientId: "",
        igdbClientSecret: "",
        vndb: "",
      };
      clearApiKeys = {
        steamgriddb: false,
        igdbClientId: false,
        igdbClientSecret: false,
        vndb: false,
      };
      notice = t("page.credentialStatusUpdated");
    } catch {
      dialogError = t("page.credentialSaveError");
    } finally {
      savingDialog = false;
    }
  }

  async function fetchGameMetadata(game: GameEntry) {
    if (fetchingMetadataId) return;
    fetchingMetadataId = game.id;
    actionError = "";
    notice = "";
    try {
      const locale = settings?.locale ?? "en";
      const metadata = await invoke<GameMetadata>("fetch_metadata", { id: game.id, locale });
      if (snapshot) {
        snapshot = {
          ...snapshot,
          games: snapshot.games.map((entry) =>
            entry.id === game.id ? { ...entry, metadata } : entry,
          ),
        };
      }
      notice = t("page.metadataUpdatedNotice", { title: game.title });
    } catch (error) {
      const code =
        typeof error === "object" && error !== null && "code" in error
          ? String(error.code)
          : "";
      actionError =
        errorMessages[code] ??
        ({
          metadataNoProviders: t("page.metadataNoProviders"),
          metadataMissingKey: t("page.metadataMissingKey"),
          metadataNotFound: t("errors.metadataNotFound"),
          metadataRequestFailed: t("errors.metadataRequestFailed"),
        }[code] ?? t("page.metadataFetchError"));
    } finally {
      fetchingMetadataId = "";
    }
  }

  function togglePlatform(platform: Platform) {
    if (!settings) return;
    const disabled = new Set(settings.disabledPlatforms);
    if (disabled.has(platform)) disabled.delete(platform);
    else disabled.add(platform);
    settings = { ...settings, disabledPlatforms: [...disabled] };
  }

  async function saveSettings(event: SubmitEvent) {
    event.preventDefault();
    if (!settings || savingDialog) return;
    savingDialog = true;
    dialogError = "";
    try {
      settings = await invoke<Settings>("update_settings", { settings });
      activeDialog = null;
      await loadLibrary(true);
      notice = t("page.settingsSaved");
    } catch {
      dialogError = t("page.settingsSaveError");
    } finally {
      savingDialog = false;
    }
  }

  function formatScanTime(seconds: number) {
    if (!seconds) return t("page.notScannedYet");
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(seconds * 1000));
  }
</script>

<svelte:head>
  <title>{t("nav.library")} · {t("app.name")}</title>
  <meta
    name="description"
    content="Browse and launch games from your installed store clients in one library."
  />
</svelte:head>

<div class="app-shell">
  <aside class="sidebar" aria-label={t("page.mainNavigation")}>
    <a class="brand" href="#library" aria-label={t("app.name")}>
      <span class="brand-mark" aria-hidden="true">U</span>
      <span class="brand-name">universal<span>launcher</span></span>
    </a>

    <div class="sidebar-label">{t("nav.library")}</div>
    <button class:active={filter === "all"} class="nav-link" onclick={() => (filter = "all")}>
      <span class="nav-icon" aria-hidden="true">▦</span>
      {t("page.allGames")}
    </button>
    <button class:active={filter === "favorites"} class="nav-link" onclick={() => (filter = "favorites")}>
      <span class="nav-icon" aria-hidden="true">♡</span>{t("nav.favorites")}
    </button>
    <button class:active={filter === "hidden"} class="nav-link" onclick={() => (filter = "hidden")}>
      <span class="nav-icon" aria-hidden="true">◌</span>{t("nav.hidden")}
    </button>

    <div class="sidebar-label tools-label">{t("page.manage")}</div>
    <button class="nav-link" onclick={openAddGame}>
      <span class="nav-icon" aria-hidden="true">＋</span>{t("nav.addGame")}
    </button>
    <button class="nav-link" onclick={openSettings}>
      <span class="nav-icon" aria-hidden="true">⚙</span>{t("nav.settings")}
    </button>

    <div class="sidebar-footer">
      <span class="status-dot" aria-hidden="true"></span>
      <span>{t("page.localLibrary")}</span>
    </div>
  </aside>

  <main id="library" class="main-content">
    <header class="topbar">
      <div class="breadcrumb">{t("page.collection")} <span>/</span> {t("nav.library")}</div>
      <button
        class="refresh-button"
        onclick={() => loadLibrary(true)}
        disabled={loading || refreshing}
        aria-label={refreshing ? t("library.scanning") : t("library.refresh")}
      >
        <span class:spinning={refreshing} aria-hidden="true">↻</span>
        {refreshing ? t("page.scanning") : t("page.rescan")}
      </button>
    </header>

    <section class="page-heading" aria-labelledby="page-title">
      <div>
        <p class="eyebrow">{t("page.libraryEyebrow")}</p>
        <h1 id="page-title">{t("nav.library")}</h1>
        <p class="subtitle">
          {#if snapshot}
            {@const games = filter === "hidden"
              ? snapshot.games.filter((game) => game.hidden).length
              : snapshot.games.filter((game) => !game.hidden).length}
            {@const platforms = snapshot.platforms.filter((platform) => platform.gameCount > 0).length}
            {t(games === 1 ? "home.stats.one" : "home.stats.other", { count: games, platforms })}
          {:else}
            {t("page.collectionSubtitle")}
          {/if}
        </p>
      </div>
      {#if snapshot}
        <div class="last-scan">{t("page.lastScan")} <strong>{formatScanTime(snapshot.scannedAt)}</strong></div>
      {/if}
    </section>

    {#if snapshot}
      {@const platformErrors = snapshot.platforms.filter((platform) => platform.error)}
      {#if platformErrors.length > 0}
        <div class="scan-warning" role="status">
          <span aria-hidden="true">!</span>
          {t("page.someLibrariesFailed")}
        </div>
      {/if}
    {/if}

    {#if actionError}
      <div class="message error-message" role="alert">
        {actionError}
        <button class="message-dismiss" onclick={() => (actionError = "")} aria-label={t("page.dismissError")}>
          ×
        </button>
      </div>
    {/if}
    {#if notice}
      <div class="message success-message" role="status">
        {notice}
        <button class="message-dismiss" onclick={() => (notice = "")} aria-label={t("page.dismissMessage")}>
          ×
        </button>
      </div>
    {/if}

    <section class="library-tools" aria-label={t("page.libraryControls")}>
      <label class="search-box">
        <span aria-hidden="true">⌕</span>
        <span class="visually-hidden">{t("page.searchGames")}</span>
        <input bind:value={query} placeholder={t("library.searchPlaceholder")} type="search" />
        {#if query}
          <button class="clear-search" onclick={() => (query = "")} aria-label={t("page.clearSearch")}>×</button>
        {/if}
      </label>
      <div class="filter-tabs" aria-label={t("page.filterGames")}>
        <button class:chosen={filter === "all"} aria-pressed={filter === "all"} onclick={() => (filter = "all")}>
          {t("page.allGames")}
        </button>
        <button
          class:chosen={filter === "installed"}
          aria-pressed={filter === "installed"}
          onclick={() => (filter = "installed")}
        >
          {t("nav.installed")}
        </button>
        <button
          class:chosen={filter === "favorites"}
          aria-pressed={filter === "favorites"}
          onclick={() => (filter = "favorites")}
        >
          {t("nav.favorites")}
        </button>
        <button
          class:chosen={filter === "hidden"}
          aria-pressed={filter === "hidden"}
          onclick={() => (filter = "hidden")}
        >
          {t("nav.hidden")}
        </button>
      </div>
    </section>

    {#if loading}
      <section class="loading-state" aria-live="polite">
        <div class="loading-spinner" aria-hidden="true"></div>
        <h2>{t("page.findingGames")}</h2>
        <p>{t("page.checkingLibraries")}</p>
      </section>
    {:else if loadError}
      <section class="empty-state" role="alert">
        <div class="empty-icon" aria-hidden="true">↻</div>
        <h2>{t("page.libraryUnavailable")}</h2>
        <p>{loadError}</p>
        <button class="primary-button" onclick={() => loadLibrary()}>{t("page.tryAgain")}</button>
      </section>
    {:else if visibleGames.length === 0}
      <section class="empty-state">
        <div class="empty-icon" aria-hidden="true">{snapshot?.games.length ? "⌕" : "▦"}</div>
        {#if snapshot?.games.length}
          <h2>{t("page.noGamesMatch")}</h2>
          <p>{t("page.changeSearchOrFilter")}</p>
          <button
            class="secondary-button"
            onclick={() => {
              query = "";
              filter = "all";
            }}>{t("library.clearFilters")}</button
          >
        {:else}
          <h2>{t("page.libraryReady")}</h2>
          <p>
            {t("page.emptyLibraryHint")}
          </p>
          <button class="secondary-button" onclick={() => loadLibrary(true)}>{t("library.refresh")}</button>
        {/if}
      </section>
    {:else}
      <section class="game-grid" aria-label={t("page.games")}>
        {#each visibleGames as game (game.id)}
          <article class="game-card">
            <div class="cover-wrap">
              {#if game.coverUrl || game.metadata?.coverUrl || game.heroUrl || game.metadata?.heroUrl}
                <img
                  class="cover-image"
                  src={game.coverUrl ?? game.metadata?.coverUrl ?? game.heroUrl ?? game.metadata?.heroUrl ?? ""}
                  alt=""
                  loading="lazy"
                />
              {:else}
                <div class="cover-placeholder" aria-hidden="true">
                  <span>{game.title.slice(0, 1).toLocaleUpperCase()}</span>
                </div>
              {/if}
              <span class:installed={game.installed} class="game-status">
                {game.custom ? t("page.addedByYou") : game.installed ? t("game.status.installed") : game.install ? t("game.status.notInstalled") : t("game.status.cloud")}
              </span>
              <button
                class:favorite-active={game.favorite}
                class="favorite-button"
                onclick={() => toggleFavorite(game)}
                disabled={activeGameId === game.id}
                aria-label={game.favorite
                  ? t("game.unfavorite") + `: ${game.title}`
                  : t("game.favorite") + `: ${game.title}`}
                aria-pressed={game.favorite}
              >
                {game.favorite ? "♥" : "♡"}
              </button>
              <button
                class="visibility-button"
                onclick={() => toggleHidden(game)}
                disabled={activeGameId === game.id}
                aria-label={game.hidden ? `${t("game.unhide")}: ${game.title}` : `${t("game.hide")}: ${game.title}`}
                title={game.hidden ? t("game.unhide") : t("game.hide")}
              >
                {game.hidden ? "◉" : "⊘"}
              </button>
              {#if game.custom}
                <button
                  class="remove-button"
                  onclick={() => {
                    dialogError = "";
                    pendingRemoval = game;
                  }}
                  aria-label={t("page.removeTitle", { title: game.title })}
                  title={t("game.remove")}
                >×</button>
              {/if}
              {#if game.installDir}
                <button
                  class="folder-button"
                  onclick={() => openInstallFolder(game)}
                  aria-label={t("page.openInstallFolderFor", { title: game.title })}
                  title={t("game.openFolder")}
                >↗</button>
              {/if}
              <div class="cover-shade"></div>
            </div>
            <div class="game-info">
              <div class="game-copy">
                <h2 title={game.title}>{game.title}</h2>
                <p>{platformNames[game.platform]}</p>
              </div>
              {#if game.installed || game.install}
                <button
                  class="game-action"
                  onclick={() => runGameAction(game)}
                  disabled={activeGameId === game.id}
                  aria-label={game.installed
                    ? t("page.launchTitle", { title: game.title })
                    : t("page.installTitle", { title: game.title })}
                >
                  {#if activeGameId === game.id}
                    <span class="small-spinner" aria-hidden="true"></span>
                  {:else if game.installed}
                    <span aria-hidden="true">▶</span>
                  {:else}
                    <span aria-hidden="true">↓</span>
                  {/if}
                </button>
              {/if}
            </div>
            {#if game.metadata}
              <div class="metadata-details">
                {#if game.metadata.developer}
                  <p class="metadata-developer">{game.metadata.developer}</p>
                {/if}
                {#if game.metadata.description}
                  <p class="metadata-description">{game.metadata.description}</p>
                {/if}
                {#if game.metadata.genres.length}
                  <p class="metadata-genres">{game.metadata.genres.slice(0, 3).join(" · ")}</p>
                {/if}
              </div>
            {/if}
            <button
              class="metadata-button"
              onclick={() => fetchGameMetadata(game)}
              disabled={fetchingMetadataId === game.id || Boolean(fetchingMetadataId)}
            >
              {fetchingMetadataId === game.id
                ? t("page.fetchingDetails")
                : game.metadata
                  ? t("page.refreshDetails")
                  : t("page.fetchDetails")}
            </button>
          </article>
        {/each}
      </section>
    {/if}

    {#if snapshot && !loading && visibleGames.length > 0}
      <p class="results-count">
        {t("page.showingResults", { visible: visibleGames.length, total: filter === "hidden"
          ? snapshot.games.filter((game) => game.hidden).length
          : snapshot.games.filter((game) => !game.hidden).length })}
      </p>
    {/if}
  </main>
</div>

{#if activeDialog === "add"}
  <div class="modal-backdrop">
    <dialog open class="modal" aria-modal="true" aria-labelledby="add-title">
      <div class="modal-heading">
        <div>
          <p class="eyebrow">{t("page.personalLibrary")}</p>
          <h2 id="add-title">{t("addGame.title")}</h2>
        </div>
        <button class="message-dismiss" onclick={() => (activeDialog = null)} aria-label={t("page.close")}>×</button>
      </div>
      <form onsubmit={addCustomGame}>
        <label class="form-field">
          <span>{t("page.gameType")}</span>
          <select bind:value={newPlatform}>
            <option value="local">{t("addGame.tabs.local")}</option>
            <option value="geforce-now">{t("page.geforceShortcut")}</option>
            <option value="xcloud">{t("page.xcloudShortcut")}</option>
          </select>
        </label>
        <label class="form-field">
          <span>{t("addGame.name")}</span>
          <input bind:value={newTitle} maxlength="200" required placeholder={t("page.gameName")} />
        </label>
        {#if newPlatform === "local"}
          <div class="form-field">
              <span>{t("addGame.executable")}</span>
            <div class="file-picker">
                <input value={newExecutable} readonly placeholder={t("addGame.pickExecutable")} />
                <button class="secondary-button" type="button" onclick={chooseExecutable}>{t("addGame.browse")}</button>
            </div>
          </div>
          <label class="form-field">
            <span>{t("addGame.arguments")} <small>{t("page.optionalSeparatedBySpaces")}</small></span>
            <input bind:value={newArgs} placeholder={t("addGame.argumentsPlaceholder")} />
          </label>
        {:else}
          <label class="form-field">
            <span>{t("addGame.url")}</span>
            <input
              bind:value={newUrl}
              type="url"
              required
              placeholder={newPlatform === "xcloud" ? "https://www.xbox.com/play/…" : "https://play.geforcenow.com/…"}
            />
          </label>
        {/if}
        <label class="form-field">
          <span>{t("addGame.coverUrl")} <small>{t("page.httpsOnly")}</small></span>
          <input bind:value={newCoverUrl} type="url" placeholder="https://…" />
        </label>
        {#if dialogError}<p class="form-error" role="alert">{dialogError}</p>{/if}
        <div class="modal-actions">
          <button class="secondary-button" type="button" onclick={() => (activeDialog = null)}>{t("addGame.cancel")}</button>
          <button class="primary-button" type="submit" disabled={savingDialog}>
            {savingDialog ? t("page.adding") : t("addGame.submit")}
          </button>
        </div>
      </form>
    </dialog>
  </div>
{:else if activeDialog === "settings"}
  <div class="modal-backdrop">
    <dialog open class="modal settings-modal" aria-modal="true" aria-labelledby="settings-title">
      <div class="modal-heading">
        <div>
          <p class="eyebrow">{t("page.preferences")}</p>
          <h2 id="settings-title">{t("settings.title")}</h2>
        </div>
        <button class="message-dismiss" onclick={() => (activeDialog = null)} aria-label={t("page.close")}>×</button>
      </div>
      {#if !settings && savingDialog}
        <div class="settings-loading" role="status">{t("page.settingsLoading")}</div>
      {:else if settings}
        <form onsubmit={saveSettings}>
          <fieldset class="platform-options">
            <legend>{t("page.enabledLibraries")}</legend>
            <p class="field-hint">{t("page.disabledLibrariesHint")}</p>
            {#each Object.entries(platformNames) as [platform, name]}
              <label class="platform-option">
                <span>{name}</span>
                <input
                  type="checkbox"
                  checked={!settings.disabledPlatforms.includes(platform as Platform)}
                  onchange={() => togglePlatform(platform as Platform)}
                />
              </label>
            {/each}
          </fieldset>
          <label class="platform-option preference-option">
            <span>{t("settings.minimizeOnLaunch")}</span>
            <input
              type="checkbox"
              checked={settings.minimizeOnLaunch}
              onchange={(event) =>
                (settings = { ...settings!, minimizeOnLaunch: event.currentTarget.checked })}
            />
          </label>
          <fieldset class="provider-options">
            <legend>{t("settings.metadataProviders")}</legend>
            <p class="field-hint">
              {t("page.rawgNotice")}
            </p>
            {#each metadataProviders as provider}
              <label class="provider-option">
                <span>
                  <strong>{provider.name}</strong>
                  <small>{provider.hint}</small>
                </span>
                <input
                  type="checkbox"
                  checked={settings.metadataProviders.includes(provider.id)}
                  onchange={() => toggleMetadataProvider(provider.id)}
                />
              </label>
            {/each}
          </fieldset>
          <fieldset class="credential-options">
            <legend>{t("settings.sections.metadata")}</legend>
            <p class="field-hint">
              {t("page.credentialHint")}
            </p>
            <label class="form-field">
              <span>SteamGridDB {t("settings.apiKey")} {apiKeyStatus?.steamgriddb ? `· ${t("settings.keySaved")}` : `· ${t("settings.keyNotSet")}`}</span>
              <input
                type="password"
                autocomplete="new-password"
                bind:value={apiKeyDrafts.steamgriddb}
                placeholder={t("page.enterNewKey")}
              />
              {#if apiKeyStatus?.steamgriddb}
                <span class="clear-key">
                  <input type="checkbox" bind:checked={clearApiKeys.steamgriddb} />
                  {t("page.clearSavedKey")}
                </span>
              {/if}
            </label>
            <label class="form-field">
              <span>IGDB / Twitch {t("settings.clientId")} {apiKeyStatus?.igdbClientId ? `· ${t("settings.keySaved")}` : `· ${t("settings.keyNotSet")}`}</span>
              <input
                type="password"
                autocomplete="new-password"
                bind:value={apiKeyDrafts.igdbClientId}
                placeholder={t("page.enterNewClientId")}
              />
              {#if apiKeyStatus?.igdbClientId}
                <span class="clear-key">
                  <input type="checkbox" bind:checked={clearApiKeys.igdbClientId} />
                  {t("page.clearSavedClientId")}
                </span>
              {/if}
            </label>
            <label class="form-field">
              <span>IGDB / Twitch {t("settings.clientSecret")} {apiKeyStatus?.igdbClientSecret ? `· ${t("settings.keySaved")}` : `· ${t("settings.keyNotSet")}`}</span>
              <input
                type="password"
                autocomplete="new-password"
                bind:value={apiKeyDrafts.igdbClientSecret}
                placeholder={t("page.enterNewClientSecret")}
              />
              {#if apiKeyStatus?.igdbClientSecret}
                <span class="clear-key">
                  <input type="checkbox" bind:checked={clearApiKeys.igdbClientSecret} />
                  {t("page.clearSavedClientSecret")}
                </span>
              {/if}
            </label>
            <label class="form-field">
              <span>VNDB {t("settings.apiKey")} {apiKeyStatus?.vndb ? `· ${t("settings.keySaved")}` : `· ${t("settings.keyNotSet")}`} <small>{t("settings.keyOptional")}</small></span>
              <input
                type="password"
                autocomplete="new-password"
                bind:value={apiKeyDrafts.vndb}
                placeholder={t("page.enterNewToken")}
              />
              {#if apiKeyStatus?.vndb}
                <span class="clear-key">
                  <input type="checkbox" bind:checked={clearApiKeys.vndb} />
                  {t("page.clearSavedToken")}
                </span>
              {/if}
            </label>
            <button class="secondary-button save-keys" type="button" onclick={saveApiKeys} disabled={savingDialog}>
              {t("settings.saveKeys")}
            </button>
          </fieldset>
          {#if dialogError}<p class="form-error" role="alert">{dialogError}</p>{/if}
          <div class="modal-actions">
            <button class="secondary-button" type="button" onclick={() => (activeDialog = null)}>{t("addGame.cancel")}</button>
            <button class="primary-button" type="submit" disabled={savingDialog}>
              {savingDialog ? t("settings.saving") : t("settings.saveSettings")}
            </button>
          </div>
        </form>
      {/if}
    </dialog>
  </div>
{/if}

{#if pendingRemoval}
  <div class="modal-backdrop">
    <dialog open class="modal confirm-modal" role="alertdialog" aria-modal="true" aria-labelledby="remove-title">
      <div class="modal-heading">
        <div>
          <p class="eyebrow">{t("game.remove")}</p>
          <h2 id="remove-title">{t("game.removeConfirm", { title: pendingRemoval.title })}</h2>
        </div>
        <button class="message-dismiss" onclick={() => (pendingRemoval = null)} aria-label={t("page.close")}>×</button>
      </div>
      <p class="field-hint">{t("page.removalNotice")}</p>
      {#if dialogError}<p class="form-error" role="alert">{dialogError}</p>{/if}
      <div class="modal-actions">
        <button class="secondary-button" onclick={() => (pendingRemoval = null)}>{t("addGame.cancel")}</button>
        <button class="danger-button" onclick={removeCustomGame} disabled={savingDialog}>
          {savingDialog ? t("page.removing") : t("page.removeGame")}
        </button>
      </div>
    </dialog>
  </div>
{/if}

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(html) {
    min-width: 320px;
    min-height: 100%;
    background: #101114;
  }

  :global(body) {
    margin: 0;
    color: #f5f5f6;
    font-family:
      Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    font-size: 14px;
    line-height: 1.5;
    -webkit-font-smoothing: antialiased;
  }

  :global(button),
  :global(input) {
    font: inherit;
  }

  :global(button:focus-visible),
  :global(a:focus-visible),
  :global(input:focus-visible) {
    outline: 2px solid #bafc54;
    outline-offset: 3px;
  }

  .app-shell {
    min-height: 100vh;
  }

  .sidebar {
    position: fixed;
    inset: 0 auto 0 0;
    z-index: 2;
    display: flex;
    width: 232px;
    flex-direction: column;
    padding: 27px 18px 20px;
    border-right: 1px solid #25262b;
    background: #141518;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 11px;
    margin: 0 0 45px 5px;
    color: inherit;
    text-decoration: none;
  }

  .brand-mark {
    display: grid;
    width: 35px;
    height: 35px;
    place-items: center;
    border-radius: 11px;
    background: #bafc54;
    color: #141712;
    font-size: 20px;
    font-weight: 900;
  }

  .brand-name {
    font-size: 13px;
    font-weight: 750;
    letter-spacing: -0.4px;
  }

  .brand-name span {
    display: block;
    color: #888990;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1.8px;
    text-transform: uppercase;
  }

  .sidebar-label {
    margin: 0 10px 10px;
    color: #777981;
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 1.3px;
    text-transform: uppercase;
  }

  .tools-label {
    margin-top: 31px;
  }

  .nav-link {
    display: flex;
    width: 100%;
    align-items: center;
    gap: 12px;
    margin-bottom: 4px;
    padding: 11px 12px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: #a8a9af;
    cursor: pointer;
    font-size: 13px;
    text-align: left;
    text-decoration: none;
  }

  .nav-link:hover,
  .nav-link.active {
    background: #222329;
    color: #f8f8f9;
  }

  .nav-link.active {
    box-shadow: inset 2px 0 #bafc54;
  }

  .nav-icon {
    width: 18px;
    color: #bafc54;
    font-size: 18px;
    line-height: 1;
    text-align: center;
  }

  .sidebar-footer {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    padding: 13px 11px;
    color: #96979e;
    font-size: 12px;
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: #bafc54;
    box-shadow: 0 0 9px #bafc5470;
  }

  .main-content {
    width: min(100% - 232px, 1600px);
    min-height: 100vh;
    margin-left: 232px;
    padding: 0 48px 48px;
  }

  .topbar {
    display: flex;
    height: 73px;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #25262b;
  }

  .breadcrumb {
    color: #8e9098;
    font-size: 12px;
  }

  .breadcrumb span {
    margin: 0 9px;
    color: #4d4f56;
  }

  .refresh-button,
  .secondary-button,
  .primary-button {
    display: inline-flex;
    min-height: 37px;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 0 13px;
    border: 1px solid #383a40;
    border-radius: 7px;
    background: #1d1f23;
    color: #e7e8ea;
    cursor: pointer;
    font-size: 12px;
    font-weight: 650;
    transition: background 140ms ease, border-color 140ms ease;
  }

  .refresh-button:hover,
  .secondary-button:hover {
    border-color: #5a5d65;
    background: #27292e;
  }

  .refresh-button:disabled,
  .game-action:disabled,
  .favorite-button:disabled {
    cursor: wait;
    opacity: 0.65;
  }

  .refresh-button > span {
    color: #bafc54;
    font-size: 18px;
    line-height: 1;
  }

  .spinning {
    animation: spin 1s linear infinite;
  }

  .page-heading {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 20px;
    margin: 43px 0 27px;
  }

  .eyebrow {
    margin: 0 0 7px;
    color: #bafc54;
    font-size: 10px;
    font-weight: 750;
    letter-spacing: 1.65px;
  }

  h1 {
    margin: 0;
    font-size: clamp(30px, 4vw, 41px);
    font-weight: 750;
    letter-spacing: -1.9px;
    line-height: 1.12;
  }

  .subtitle {
    margin: 10px 0 0;
    color: #93949b;
    font-size: 13px;
  }

  .last-scan {
    padding-bottom: 4px;
    color: #81838a;
    font-size: 11px;
  }

  .last-scan strong {
    margin-left: 4px;
    color: #bcbec3;
    font-weight: 550;
  }

  .scan-warning,
  .message {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    margin: 0 0 18px;
    padding: 11px 40px 11px 13px;
    border: 1px solid #544225;
    border-radius: 7px;
    background: #292317;
    color: #e7c77f;
    font-size: 12px;
  }

  .scan-warning > span {
    display: grid;
    width: 19px;
    height: 19px;
    flex: 0 0 auto;
    place-items: center;
    border-radius: 50%;
    background: #554321;
    font-weight: 800;
  }

  .message {
    justify-content: space-between;
  }

  .error-message {
    border-color: #5d3435;
    background: #2a1c1e;
    color: #f0aaaa;
  }

  .success-message {
    border-color: #38532a;
    background: #1d291a;
    color: #c8efa1;
  }

  .message-dismiss,
  .clear-search {
    border: 0;
    background: none;
    color: inherit;
    cursor: pointer;
    font-size: 20px;
    line-height: 1;
  }

  .message-dismiss {
    position: absolute;
    top: 50%;
    right: 11px;
    transform: translateY(-50%);
  }

  .library-tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 18px;
    margin-bottom: 22px;
  }

  .search-box {
    display: flex;
    width: min(100%, 350px);
    height: 39px;
    align-items: center;
    gap: 9px;
    padding: 0 11px;
    border: 1px solid #303138;
    border-radius: 7px;
    background: #191a1e;
    color: #85878e;
  }

  .search-box > span:not(.visually-hidden) {
    font-size: 22px;
    line-height: 1;
  }

  .search-box input {
    width: 100%;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: #f3f3f5;
    font-size: 12px;
  }

  .search-box input::placeholder {
    color: #777981;
  }

  .clear-search {
    color: #92939a;
  }

  .filter-tabs {
    display: flex;
    flex: 0 0 auto;
    gap: 3px;
    padding: 3px;
    border: 1px solid #292a30;
    border-radius: 8px;
    background: #17181b;
  }

  .filter-tabs button {
    padding: 6px 10px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: #9799a0;
    cursor: pointer;
    font-size: 11px;
  }

  .filter-tabs button:hover {
    color: #e8e9eb;
  }

  .filter-tabs button.chosen {
    background: #303137;
    color: #f8f8f9;
  }

  .game-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(178px, 1fr));
    gap: 22px 17px;
  }

  .game-card {
    min-width: 0;
  }

  .cover-wrap {
    position: relative;
    overflow: hidden;
    aspect-ratio: 0.77;
    border: 1px solid #2b2c32;
    border-radius: 9px;
    background: #202126;
    isolation: isolate;
  }

  .cover-image,
  .cover-placeholder,
  .cover-shade {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .cover-image {
    z-index: -2;
    object-fit: cover;
    transition: transform 260ms ease;
  }

  .game-card:hover .cover-image {
    transform: scale(1.04);
  }

  .cover-placeholder {
    z-index: -2;
    display: grid;
    place-items: center;
    background:
      radial-gradient(ellipse at 50% 100%, #4a5737 0, transparent 50%),
      linear-gradient(145deg, #353840, #1e2025 70%);
  }

  .cover-placeholder span {
    color: #d4edb4;
    font-size: 68px;
    font-weight: 800;
    opacity: 0.72;
    text-shadow: 0 5px 20px #0008;
  }

  .cover-shade {
    z-index: -1;
    background: linear-gradient(180deg, #0007 0%, transparent 34%, transparent 62%, #000b 100%);
    pointer-events: none;
  }

  .game-status {
    position: absolute;
    top: 10px;
    left: 10px;
    padding: 4px 7px;
    border: 1px solid #ffffff24;
    border-radius: 5px;
    background: #141519c9;
    color: #d1d2d6;
    font-size: 9px;
    font-weight: 650;
    backdrop-filter: blur(7px);
  }

  .game-status.installed {
    color: #d1f8a6;
  }

  .favorite-button {
    position: absolute;
    top: 8px;
    right: 8px;
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid #ffffff24;
    border-radius: 50%;
    background: #141519c9;
    color: #fff;
    cursor: pointer;
    font-size: 17px;
    backdrop-filter: blur(7px);
  }

  .visibility-button {
    position: absolute;
    top: 45px;
    right: 8px;
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid #ffffff24;
    border-radius: 50%;
    background: #141519c9;
    color: #fff;
    cursor: pointer;
    font-size: 15px;
    backdrop-filter: blur(7px);
  }

  .visibility-button:hover {
    color: #bafc54;
  }

  .favorite-button:hover,
  .favorite-button.favorite-active {
    color: #ff7893;
  }

  .game-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 7px;
    padding: 10px 1px 0;
  }

  .remove-button {
    position: absolute;
    top: 45px;
    left: 8px;
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid #ffffff24;
    border-radius: 50%;
    background: #141519c9;
    color: #fff;
    cursor: pointer;
    font-size: 19px;
    backdrop-filter: blur(7px);
  }

  .remove-button:hover {
    color: #ff8999;
  }

  .folder-button {
    position: absolute;
    top: 81px;
    right: 8px;
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid #ffffff24;
    border-radius: 50%;
    background: #141519c9;
    color: #fff;
    cursor: pointer;
    font-size: 15px;
    backdrop-filter: blur(7px);
  }

  .folder-button:hover {
    color: #bafc54;
  }

  .game-copy {
    min-width: 0;
  }

  .game-copy h2 {
    overflow: hidden;
    margin: 0;
    color: #e9e9eb;
    font-size: 12px;
    font-weight: 650;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-copy p {
    overflow: hidden;
    margin: 3px 0 0;
    color: #85878e;
    font-size: 10px;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-action {
    display: grid;
    width: 31px;
    height: 31px;
    flex: 0 0 auto;
    place-items: center;
    border: 1px solid #bafc54;
    border-radius: 8px;
    background: #bafc54;
    color: #171a13;
    cursor: pointer;
    font-size: 12px;
    transition: background 140ms ease, transform 140ms ease;
  }

  .game-action:hover:not(:disabled) {
    transform: translateY(-1px);
    background: #d0ff88;
  }

  .small-spinner,
  .loading-spinner {
    display: block;
    border: 2px solid #43464c;
    border-top-color: #bafc54;
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  .small-spinner {
    width: 14px;
    height: 14px;
  }

  .loading-state,
  .empty-state {
    display: flex;
    min-height: 330px;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 36px 20px;
    text-align: center;
  }

  .loading-spinner {
    width: 30px;
    height: 30px;
    margin-bottom: 19px;
    border-width: 3px;
  }

  .loading-state h2,
  .empty-state h2 {
    margin: 0;
    color: #e8e9eb;
    font-size: 17px;
    font-weight: 650;
  }

  .loading-state p,
  .empty-state p {
    max-width: 390px;
    margin: 8px 0 18px;
    color: #92949b;
    font-size: 12px;
  }

  .empty-icon {
    display: grid;
    width: 56px;
    height: 56px;
    place-items: center;
    margin-bottom: 18px;
    border: 1px solid #34363c;
    border-radius: 17px;
    background: #1d1f23;
    color: #bafc54;
    font-size: 27px;
  }

  .primary-button {
    border-color: #bafc54;
    background: #bafc54;
    color: #171a13;
  }

  .primary-button:hover {
    background: #d0ff88;
  }

  .results-count {
    margin: 25px 0 0;
    color: #777981;
    font-size: 10px;
    text-align: center;
  }

  .visually-hidden {
    position: absolute;
    overflow: hidden;
    width: 1px;
    height: 1px;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    clip-path: inset(50%);
  }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 5;
    display: grid;
    overflow-y: auto;
    place-items: center;
    padding: 24px;
    background: #08090bd9;
    backdrop-filter: blur(5px);
  }

  .modal {
    position: relative;
    width: min(100%, 480px);
    max-height: min(88vh, 760px);
    overflow-y: auto;
    margin: auto;
    padding: 24px;
    border: 1px solid #383a40;
    border-radius: 13px;
    background: #191a1e;
    box-shadow: 0 24px 90px #0009;
  }

  .settings-modal {
    width: min(100%, 540px);
  }

  .confirm-modal {
    width: min(100%, 430px);
  }

  .modal-heading {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    margin-bottom: 22px;
  }

  .modal-heading .eyebrow {
    margin-bottom: 5px;
  }

  .modal-heading h2 {
    margin: 0;
    font-size: 21px;
    letter-spacing: -0.5px;
  }

  .form-field {
    display: grid;
    gap: 7px;
    margin: 0 0 15px;
    color: #d6d7da;
    font-size: 12px;
    font-weight: 600;
  }

  .form-field small {
    margin-left: 5px;
    color: #898b92;
    font-size: 10px;
    font-weight: 400;
  }

  .form-field input,
  .form-field select {
    width: 100%;
    min-height: 39px;
    padding: 8px 10px;
    border: 1px solid #383a40;
    border-radius: 6px;
    background: #111215;
    color: #f3f3f5;
    font-size: 12px;
  }

  .form-field input::placeholder {
    color: #777981;
  }

  .file-picker {
    display: flex;
    gap: 8px;
  }

  .file-picker input {
    min-width: 0;
  }

  .file-picker button {
    flex: 0 0 auto;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 9px;
    margin-top: 22px;
  }

  .form-error {
    margin: 8px 0;
    color: #f0aaaa;
    font-size: 12px;
  }

  .platform-options {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0 20px;
    margin: 0;
    padding: 0;
    border: 0;
  }

  .platform-options legend {
    margin-bottom: 4px;
    color: #e5e6e8;
    font-size: 13px;
    font-weight: 700;
  }

  .field-hint {
    margin: 4px 0 14px;
    color: #93959c;
    font-size: 11px;
    line-height: 1.5;
  }

  .platform-option {
    display: flex;
    min-height: 38px;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    border-bottom: 1px solid #292a30;
    color: #c6c7cb;
    font-size: 11px;
  }

  .platform-option input {
    width: 15px;
    height: 15px;
    accent-color: #bafc54;
  }

  .preference-option {
    margin-top: 17px;
    padding: 0 0 12px;
  }

  .settings-loading {
    padding: 30px 0;
    color: #a8a9af;
    text-align: center;
  }

  .danger-button {
    min-height: 37px;
    padding: 0 13px;
    border: 1px solid #7a343d;
    border-radius: 7px;
    background: #8c3844;
    color: #fff;
    cursor: pointer;
    font-size: 12px;
    font-weight: 650;
  }

  .danger-button:hover {
    background: #a34250;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 950px) {
    .main-content {
      padding-right: 30px;
      padding-left: 30px;
    }

    .game-grid {
      grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
      gap: 19px 13px;
    }
  }

  @media (max-width: 680px) {
    .sidebar {
      position: static;
      width: 100%;
      height: 61px;
      flex-direction: row;
      align-items: center;
      justify-content: flex-start;
      gap: 5px;
      overflow-x: auto;
      padding: 0 17px;
      border-right: 0;
      border-bottom: 1px solid #25262b;
    }

    .brand {
      flex: 0 0 auto;
      margin: 0 auto 0 0;
    }

    .brand-mark {
      width: 32px;
      height: 32px;
    }

    .sidebar-label,
    .sidebar-footer {
      display: none;
    }

    .sidebar .nav-link {
      width: auto;
      flex: 0 0 auto;
      padding: 8px 10px;
      box-shadow: none;
      font-size: 11px;
    }

    .sidebar .nav-link.active {
      background: #222329;
    }

    .sidebar .nav-icon {
      display: none;
    }

    .main-content {
      width: 100%;
      margin: 0;
      padding: 0 18px 34px;
    }

    .topbar {
      height: 57px;
    }

    .page-heading {
      align-items: flex-start;
      flex-direction: column;
      gap: 9px;
      margin: 30px 0 21px;
    }

    .last-scan {
      padding: 0;
    }

    .library-tools {
      align-items: stretch;
      flex-direction: column;
      gap: 10px;
    }

    .search-box {
      width: 100%;
    }

    .filter-tabs {
      align-self: flex-start;
    }

    .game-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 18px 12px;
    }
  }

  @media (max-width: 420px) {
    .sidebar {
      gap: 3px;
      padding: 0 10px;
    }

    .brand-name {
      display: none;
    }

    .sidebar .nav-link {
      padding: 8px;
      font-size: 0;
    }

    .sidebar .nav-icon {
      display: inline;
      font-size: 18px;
    }
  }
</style>
