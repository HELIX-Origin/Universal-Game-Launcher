//! Universal Game Launcher – Tauri backend.
//!
//! The launcher reads each official store app's local data to build one
//! library, and hands launching / installing / purchasing back to those apps.
//! See `AGENTS.md` for the architecture overview.

pub mod clients;
pub mod error;
pub mod launcher;
pub mod library;
pub mod metadata;
pub mod models;
pub mod persistence;
pub mod stores;
pub mod vdf;

use error::{AppError, AppResult, ErrorCode};
use library::AppState;
use metadata::{ApiKeyStatus, ApiKeys, GameMetadata};
use models::{LaunchTarget, LibraryEntry, LibrarySnapshot, Platform};
use persistence::{CustomGameInput, Settings};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

fn internal(e: impl std::fmt::Display) -> AppError {
    AppError::with(ErrorCode::Internal, e.to_string())
}

fn run_target(app: &AppHandle, target: &LaunchTarget) -> AppResult<()> {
    launcher::launch(target, |uri| {
        app.opener()
            .open_url(uri, None::<&str>)
            .map_err(|e| e.to_string())
    })
}

/// Return the library; rescans when `refresh` is set or nothing was scanned yet.
#[tauri::command]
async fn get_library(app: AppHandle, refresh: bool) -> AppResult<LibrarySnapshot> {
    let needs_scan = refresh || !app.state::<AppState>().lock_scan().done;
    if needs_scan {
        let settings = app.state::<AppState>().lock_data().settings.clone();
        let cache = tauri::async_runtime::spawn_blocking(move || library::scan_all(&settings))
            .await
            .map_err(internal)?;
        *app.state::<AppState>().lock_scan() = cache;
    }
    Ok(app.state::<AppState>().snapshot())
}

#[tauri::command]
fn launch_game(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let game = state.find_game(&id)?;
    run_target(&app, &game.launch)?;
    state.update(|d| d.record_launch(&id))?;
    if state.lock_data().settings.minimize_on_launch {
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.minimize();
        }
    }
    Ok(())
}

/// Ask the official store app to install a game it owns.
#[tauri::command]
fn install_game(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let game = state.find_game(&id)?;
    let target = game
        .install
        .ok_or_else(|| AppError::with(ErrorCode::LaunchFailed, "no install action"))?;
    run_target(&app, &target)
}

/// Open the platform's official app (or its storefront when `store` is set).
#[tauri::command]
fn open_client(app: AppHandle, platform: Platform, store: bool) -> AppResult<()> {
    let actions = clients::actions(platform);
    let target = if store { actions.store } else { actions.open };
    let target = target.ok_or_else(|| AppError::with(ErrorCode::LaunchFailed, platform.key()))?;
    run_target(&app, &target)
}

#[tauri::command]
fn open_install_dir(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    let dir = state
        .find_game(&id)?
        .install_dir
        .filter(|d| d.is_dir())
        .ok_or(ErrorCode::NoInstallDir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::with(ErrorCode::LaunchFailed, e.to_string()))
}

#[tauri::command]
fn add_custom_game(state: State<'_, AppState>, input: CustomGameInput) -> AppResult<LibraryEntry> {
    let custom = input.into_custom_game()?;
    let game = custom.to_game();
    state.update(|d| d.custom_games.push(custom))?;
    let data = state.lock_data().clone();
    Ok(state.entry(game, &data, true))
}

#[tauri::command]
fn remove_custom_game(state: State<'_, AppState>, id: String) -> AppResult<bool> {
    state.update(|d| d.remove_custom(&id))
}

#[tauri::command]
fn set_favorite(state: State<'_, AppState>, id: String, value: bool) -> AppResult<()> {
    state.update(|d| {
        if value {
            d.favorites.insert(id);
        } else {
            d.favorites.remove(&id);
        }
    })
}

#[tauri::command]
fn set_hidden(state: State<'_, AppState>, id: String, value: bool) -> AppResult<()> {
    state.update(|d| {
        if value {
            d.hidden.insert(id);
        } else {
            d.hidden.remove(&id);
        }
    })
}

#[tauri::command]
fn get_settings(state: State<'_, AppState>) -> Settings {
    state.lock_data().settings.clone()
}

#[tauri::command]
fn update_settings(state: State<'_, AppState>, settings: Settings) -> AppResult<Settings> {
    let settings = settings.sanitized();
    state.update(|d| d.settings = settings.clone())?;
    Ok(settings)
}

#[tauri::command]
fn get_api_key_status(state: State<'_, AppState>) -> ApiKeyStatus {
    state.lock_keys().status()
}

/// Store user-provided API keys. Keys are write-only from the UI's perspective.
#[tauri::command]
fn set_api_keys(state: State<'_, AppState>, keys: ApiKeys) -> AppResult<ApiKeyStatus> {
    let mut stored = state.lock_keys();
    stored.merge(keys);
    stored.save(&state.secrets_file())?;
    Ok(stored.status())
}

#[tauri::command]
async fn fetch_metadata(app: AppHandle, id: String, locale: String) -> AppResult<GameMetadata> {
    let state = app.state::<AppState>();
    let game = state.find_game(&id)?;
    let providers = state.lock_data().settings.metadata_providers.clone();
    let keys = state.lock_keys().clone();
    let meta = tauri::async_runtime::spawn_blocking(move || {
        metadata::fetch(&game, &providers, &keys, &locale, persistence::now_secs())
    })
    .await
    .map_err(internal)??;
    state.update(|d| d.metadata.insert(id, meta.clone()))?;
    Ok(meta)
}

#[tauri::command]
fn clear_metadata(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.update(|d| {
        d.metadata.remove(&id);
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    data_dir: String,
    os: &'static str,
}

#[tauri::command]
fn get_app_info(app: AppHandle, state: State<'_, AppState>) -> AppInfo {
    AppInfo {
        version: app.package_info().version.to_string(),
        data_dir: state.data_dir.display().to_string(),
        os: std::env::consts::OS,
    }
}

#[tauri::command]
fn open_data_dir(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    std::fs::create_dir_all(&state.data_dir)
        .map_err(|e| AppError::with(ErrorCode::Storage, e.to_string()))?;
    app.opener()
        .open_path(state.data_dir.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::with(ErrorCode::LaunchFailed, e.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState::new(data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_library,
            launch_game,
            install_game,
            open_client,
            open_install_dir,
            add_custom_game,
            remove_custom_game,
            set_favorite,
            set_hidden,
            get_settings,
            update_settings,
            get_api_key_status,
            set_api_keys,
            fetch_metadata,
            clear_metadata,
            get_app_info,
            open_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
