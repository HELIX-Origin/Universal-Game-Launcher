//! Integration tests for library state across modules: scan cache, user
//! data persistence, custom games and settings, exercised through the same
//! `AppState` API the Tauri commands use.

mod common;

use common::{seed_scan, steam_game, temp_state};
use ugl_lib::error::ErrorCode;
use ugl_lib::library::AppState;
use ugl_lib::metadata::{ApiKeys, MetadataProvider};
use ugl_lib::models::Platform;
use ugl_lib::persistence::{CustomGameInput, Settings, UserData};

fn cloud_input(title: &str) -> CustomGameInput {
    serde_json::from_value(serde_json::json!({
        "platform": "geforce-now",
        "title": title,
        "url": "https://play.geforcenow.com/games?game-id=synthetic",
    }))
    .unwrap()
}

#[test]
fn user_state_survives_a_restart() {
    let (dir, state) = temp_state();
    seed_scan(&state, vec![steam_game("1", "Persisted")], 10);
    state
        .update(|d| {
            d.favorites.insert("steam:1".into());
            d.hidden.insert("steam:1".into());
            d.record_launch("steam:1");
        })
        .unwrap();

    let restarted = AppState::new(dir.path().to_path_buf());
    seed_scan(&restarted, vec![steam_game("1", "Persisted")], 20);
    let entry = &restarted.snapshot().games[0];
    assert!(entry.favorite);
    assert!(entry.hidden);
    assert_eq!(entry.play_count, 1);
    assert!(entry.last_played.is_some());
}

#[test]
fn custom_games_are_findable_and_removable() {
    let (_dir, state) = temp_state();
    let custom = cloud_input("Streamed").into_custom_game().unwrap();
    let id = custom.to_game().id;
    state.update(|d| d.custom_games.push(custom)).unwrap();
    state.update(|d| d.favorites.insert(id.clone())).unwrap();

    let snapshot = state.snapshot();
    assert_eq!(snapshot.games.len(), 1);
    assert!(snapshot.games[0].custom);
    assert_eq!(state.find_game(&id).unwrap().platform, Platform::GeforceNow);

    assert!(state.update(|d| d.remove_custom(&id)).unwrap());
    assert!(!state.lock_data().favorites.contains(&id));
    assert_eq!(
        state.find_game(&id).unwrap_err().code,
        ErrorCode::GameNotFound
    );
    assert!(!state.update(|d| d.remove_custom(&id)).unwrap());
}

#[test]
fn disabled_platforms_are_hidden_from_snapshot_and_counts() {
    let (_dir, state) = temp_state();
    seed_scan(&state, vec![steam_game("1", "A"), steam_game("2", "B")], 1);
    state
        .update(|d| {
            d.settings.disabled_platforms.insert(Platform::Steam);
        })
        .unwrap();

    let snapshot = state.snapshot();
    assert!(snapshot.games.is_empty());
    let steam = snapshot
        .platforms
        .iter()
        .find(|p| p.platform == Platform::Steam)
        .unwrap();
    assert!(!steam.enabled);
    assert_eq!(steam.game_count, 0);
}

#[test]
fn snapshot_lists_every_platform_once_in_stable_order() {
    let (_dir, state) = temp_state();
    let platforms: Vec<_> = state
        .snapshot()
        .platforms
        .iter()
        .map(|p| p.platform)
        .collect();
    assert_eq!(platforms, Platform::ALL.to_vec());
}

#[test]
fn settings_from_the_ui_are_sanitized_before_persisting() {
    let (dir, state) = temp_state();
    let incoming: Settings = serde_json::from_value(serde_json::json!({
        "theme": "neon",
        "viewMode": "carousel",
        "locale": "not a locale!",
        "steamPath": "",
        "metadataProviders": ["rawg", "igdb", "igdb", "vndb"],
    }))
    .unwrap();
    let sanitized = incoming.sanitized();
    state.update(|d| d.settings = sanitized.clone()).unwrap();

    let reloaded = UserData::load(&dir.path().join(ugl_lib::persistence::DATA_FILE));
    assert_eq!(reloaded.settings, sanitized);
    assert_eq!(reloaded.settings.theme, "dark");
    assert_eq!(reloaded.settings.view_mode, "grid");
    assert_eq!(reloaded.settings.locale, None);
    assert_eq!(reloaded.settings.steam_path, None);
    assert_eq!(
        reloaded.settings.metadata_providers,
        vec![MetadataProvider::Igdb, MetadataProvider::Vndb]
    );
}

#[test]
fn corrupt_user_data_is_backed_up_and_replaced_with_defaults() {
    let (dir, state) = temp_state();
    let data_file = state.data_file();
    std::fs::write(&data_file, "{ not json").unwrap();

    let restarted = AppState::new(dir.path().to_path_buf());
    assert_eq!(*restarted.lock_data(), UserData::default());
    assert!(data_file.with_extension("json.bak").exists());
}

#[test]
fn api_keys_persist_and_report_only_presence() {
    let (dir, state) = temp_state();
    {
        let mut keys = state.lock_keys();
        keys.merge(ApiKeys {
            steamgriddb: Some("synthetic".into()),
            ..ApiKeys::default()
        });
        keys.save(&state.secrets_file()).unwrap();
    }
    let restarted = AppState::new(dir.path().to_path_buf());
    let status = restarted.lock_keys().status();
    assert!(status.steamgriddb);
    assert!(!status.igdb_client_id);
}

#[cfg(unix)]
#[test]
fn api_key_file_is_private_to_the_user() {
    use std::os::unix::fs::PermissionsExt;
    let (_dir, state) = temp_state();
    ApiKeys {
        vndb: Some("synthetic".into()),
        ..ApiKeys::default()
    }
    .save(&state.secrets_file())
    .unwrap();
    let mode = std::fs::metadata(state.secrets_file())
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o077,
        0,
        "secrets file must not be group/world readable"
    );
}
