//! Rust ↔ TypeScript payload contracts.
//!
//! Responses are serialized into golden JSON under `tests/contracts/fixtures/`
//! and validated against the TypeScript types by `npm run test:contracts`.
//! Requests in `tests/contracts/fixtures/requests/` mirror what the UI sends
//! and must deserialize into the Rust command argument types.

mod common;

use common::{assert_golden, request_fixture, seed_scan, steam_game, temp_state};
use serde_json::json;
use std::path::PathBuf;
use ugl_lib::error::{AppError, ErrorCode};
use ugl_lib::metadata::{ApiKeys, GameMetadata, MetadataProvider};
use ugl_lib::models::{Game, LaunchTarget, LibrarySnapshot, Platform, PlatformStatus};
use ugl_lib::persistence::{CustomGame, CustomGameInput, Settings};

fn sample_metadata() -> GameMetadata {
    GameMetadata {
        description: Some("Synthetic description.".into()),
        developer: Some("Synthetic Studio".into()),
        publisher: None,
        release_date: Some("2020-01-01".into()),
        genres: vec!["Puzzle".into()],
        rating: Some(87.5),
        cover_url: Some("https://example.com/cover.png".into()),
        hero_url: None,
        sources: vec![MetadataProvider::SteamStore, MetadataProvider::Igdb],
        fetched_at: 1_700_000_000,
    }
}

/// A snapshot exercising every optional field in both its empty and filled form.
fn golden_snapshot() -> serde_json::Value {
    let (_dir, state) = temp_state();
    let installed = steam_game("10", "Installed Game")
        .with_install_dir("/synthetic/games/installed")
        .with_cover("https://example.com/cover.png")
        .with_hero("https://example.com/hero.png");
    let owned = Game::new(
        Platform::Gog,
        "20",
        "Owned Game",
        LaunchTarget::uri("goggalaxy://openGameView/20"),
    )
    .not_installed(LaunchTarget::uri("goggalaxy://openGameView/20"));
    let cloud = CustomGame {
        id: "fixed-id".into(),
        platform: Platform::GeforceNow,
        title: "Cloud Game".into(),
        launch: LaunchTarget::uri("https://play.geforcenow.com/games?game-id=synthetic"),
        cover_url: None,
    };
    let local = CustomGame {
        id: "local-id".into(),
        platform: Platform::Local,
        title: "Local Game".into(),
        launch: LaunchTarget::Executable {
            path: PathBuf::from("/synthetic/bin/game"),
            args: vec!["--windowed".into()],
            working_dir: Some(PathBuf::from("/synthetic/bin")),
        },
        cover_url: None,
    };

    seed_scan(
        &state,
        vec![installed.clone(), owned.clone()],
        1_700_000_000,
    );
    state
        .update(|d| {
            d.favorites.insert(installed.id.clone());
            d.hidden.insert(owned.id.clone());
            d.metadata.insert(installed.id.clone(), sample_metadata());
            d.custom_games.push(cloud.clone());
            d.custom_games.push(local.clone());
        })
        .unwrap();
    let data = state.lock_data().clone();

    // Platform rows are built explicitly because client availability is OS-specific.
    let snapshot = LibrarySnapshot {
        games: vec![
            state.entry(installed, &data, false),
            state.entry(owned, &data, false),
            state.entry(cloud.to_game(), &data, true),
            state.entry(local.to_game(), &data, true),
        ],
        platforms: vec![
            PlatformStatus {
                platform: Platform::Steam,
                enabled: true,
                game_count: 1,
                error: None,
                can_open: true,
                can_open_store: true,
            },
            PlatformStatus {
                platform: Platform::Epic,
                enabled: true,
                game_count: 0,
                error: Some("synthetic scan failure".into()),
                can_open: false,
                can_open_store: false,
            },
        ],
        scanned_at: 1_700_000_000,
    };
    serde_json::to_value(snapshot).unwrap()
}

#[test]
fn library_snapshot_payload() {
    assert_golden("library-snapshot.json", &golden_snapshot());
}

#[test]
fn snapshot_built_by_app_state_matches_golden_game_shape() {
    let (_dir, state) = temp_state();
    seed_scan(&state, vec![steam_game("1", "Any")], 1);
    let live = serde_json::to_value(state.snapshot()).unwrap();
    let golden = golden_snapshot();
    let keys = |v: &serde_json::Value| {
        let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
        k.sort();
        k
    };
    assert_eq!(keys(&live), keys(&golden));
    assert_eq!(keys(&live["games"][0]), keys(&golden["games"][0]));
    assert_eq!(keys(&live["platforms"][0]), keys(&golden["platforms"][0]));
    assert_eq!(
        live["platforms"].as_array().unwrap().len(),
        Platform::ALL.len()
    );
}

#[test]
fn settings_payloads() {
    assert_golden(
        "settings.default.json",
        &serde_json::to_value(Settings::default()).unwrap(),
    );
    let customized = Settings {
        disabled_platforms: [Platform::Origin, Platform::BattleNet].into(),
        steam_path: Some(PathBuf::from("/synthetic/steam")),
        theme: "light".into(),
        view_mode: "list".into(),
        minimize_on_launch: true,
        locale: Some("en-US".into()),
        metadata_providers: vec![MetadataProvider::SteamGridDb, MetadataProvider::Vndb],
    };
    assert_golden(
        "settings.customized.json",
        &serde_json::to_value(customized).unwrap(),
    );
}

#[test]
fn api_key_status_payload() {
    let keys = ApiKeys {
        steamgriddb: Some("synthetic".into()),
        igdb_client_id: None,
        igdb_client_secret: Some("  ".into()),
        vndb: None,
        rawg: Some("legacy".into()),
    };
    let status = serde_json::to_value(keys.status()).unwrap();
    assert!(
        !status.to_string().contains("synthetic"),
        "key status must never include secret values"
    );
    assert_golden("api-key-status.json", &status);
}

#[test]
fn error_payloads() {
    let codes = [
        ErrorCode::TitleRequired,
        ErrorCode::TitleTooLong,
        ErrorCode::InvalidCoverUrl,
        ErrorCode::ExecutableRequired,
        ErrorCode::ExecutableNotAbsolute,
        ErrorCode::ExecutableNotFound,
        ErrorCode::UrlRequired,
        ErrorCode::UrlNotHttps,
        ErrorCode::UrlHostNotAllowed,
        ErrorCode::UnsupportedPlatform,
        ErrorCode::GameNotFound,
        ErrorCode::LaunchFailed,
        ErrorCode::UriSchemeNotAllowed,
        ErrorCode::NoInstallDir,
        ErrorCode::Storage,
        ErrorCode::MetadataNoProviders,
        ErrorCode::MetadataMissingKey,
        ErrorCode::MetadataRequestFailed,
        ErrorCode::MetadataNotFound,
        ErrorCode::Internal,
    ];
    let errors = json!({
        "codes": codes.iter().map(|c| serde_json::to_value(c).unwrap()).collect::<Vec<_>>(),
        "withoutDetail": AppError::new(ErrorCode::GameNotFound),
        "withDetail": AppError::with(ErrorCode::Storage, "synthetic detail"),
    });
    assert_golden("errors.json", &errors);
}

#[test]
fn metadata_providers_payload() {
    let mut providers: Vec<_> = MetadataProvider::ALL.to_vec();
    providers.push(MetadataProvider::Rawg);
    assert_golden(
        "metadata-providers.json",
        &json!({
            "selectable": MetadataProvider::ALL,
            "all": providers,
        }),
    );
}

#[test]
fn platforms_payload() {
    assert_golden(
        "platforms.json",
        &json!(Platform::ALL
            .iter()
            .map(|p| json!({ "id": p, "key": p.key(), "cloud": p.is_cloud(), "userManaged": p.is_user_managed() }))
            .collect::<Vec<_>>()),
    );
}

#[test]
fn add_custom_game_requests_deserialize() {
    let local: CustomGameInput = request_fixture("add-custom-game.local.json");
    assert_eq!(local.platform, Platform::Local);
    assert_eq!(local.title, "Homebrew");
    assert_eq!(local.executable, Some(PathBuf::from("/synthetic/bin/game")));
    assert_eq!(local.args, vec!["--windowed", "--fps", "60"]);
    assert_eq!(local.url, None);

    let cloud: CustomGameInput = request_fixture("add-custom-game.cloud.json");
    assert_eq!(cloud.platform, Platform::Xcloud);
    assert_eq!(
        cloud.url.as_deref(),
        Some("https://www.xbox.com/play/synthetic")
    );
    assert_eq!(
        cloud.cover_url.as_deref(),
        Some("https://example.com/cover.png")
    );
    let game = cloud.into_custom_game().expect("cloud fixture is valid");
    assert_eq!(game.platform, Platform::Xcloud);
}

#[test]
fn update_settings_request_deserializes_without_dropping_fields() {
    let settings: Settings = request_fixture("update-settings.json");
    let expected: Settings =
        serde_json::from_value(serde_json::to_value(&settings).unwrap()).unwrap();
    assert_eq!(settings, expected);
    assert!(settings.disabled_platforms.contains(&Platform::Steam));
    assert!(settings.minimize_on_launch);
    assert_eq!(settings.locale.as_deref(), Some("en"));
    assert_eq!(
        settings.metadata_providers,
        vec![MetadataProvider::SteamStore, MetadataProvider::Igdb]
    );
    assert_ne!(settings, Settings::default());
}

#[test]
fn set_api_keys_request_deserializes() {
    let keys: ApiKeys = request_fixture("set-api-keys.json");
    let mut stored = ApiKeys {
        steamgriddb: Some("old".into()),
        ..ApiKeys::default()
    };
    stored.merge(keys);
    let status = stored.status();
    assert!(!status.steamgriddb, "empty string clears a stored key");
    assert!(status.vndb);
    assert!(status.igdb_client_id);
    assert!(!status.igdb_client_secret, "omitted keys stay untouched");
}
