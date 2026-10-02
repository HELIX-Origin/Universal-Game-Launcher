//! Shared helpers for backend integration tests.
//!
//! Fixtures are synthetic. Never read real store clients, user data or
//! credentials from these tests.
#![allow(dead_code)]

use std::path::PathBuf;
use ugl_lib::library::AppState;
use ugl_lib::models::{Game, LaunchTarget, Platform};

/// Environment variable that rewrites golden contract fixtures instead of comparing.
pub const UPDATE_ENV: &str = "UGL_UPDATE_CONTRACTS";

/// `tests/contracts/fixtures` at the repository root, shared with the frontend suite.
pub fn contract_fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("contracts")
        .join("fixtures")
}

/// Compare `value` with a committed golden JSON file, or rewrite it when
/// `UGL_UPDATE_CONTRACTS=1` is set.
pub fn assert_golden(name: &str, value: &serde_json::Value) {
    let path = contract_fixtures_dir().join(name);
    let rendered = format!("{}\n", serde_json::to_string_pretty(value).unwrap());
    if std::env::var(UPDATE_ENV).is_ok_and(|v| v == "1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, rendered).unwrap();
        return;
    }
    let committed = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing contract fixture {}; run `{UPDATE_ENV}=1 cargo test --test contracts`",
            path.display()
        )
    });
    let committed: serde_json::Value = serde_json::from_str(&committed).unwrap();
    assert_eq!(
        &committed,
        value,
        "Rust payload for {name} changed. If intentional, run `{UPDATE_ENV}=1 cargo test --test contracts` \
         and update the TypeScript types in src/lib/library.ts, then run `npm run test:contracts`."
    );
}

/// Read a frontend-shaped request fixture from `tests/contracts/fixtures/requests`.
pub fn request_fixture<T: serde::de::DeserializeOwned>(name: &str) -> T {
    let path = contract_fixtures_dir().join("requests").join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name} does not deserialize: {e}"))
}

/// An `AppState` rooted in a fresh temporary data directory.
pub fn temp_state() -> (tempfile::TempDir, AppState) {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new(dir.path().to_path_buf());
    (dir, state)
}

pub fn steam_game(id: &str, title: &str) -> Game {
    Game::new(
        Platform::Steam,
        id,
        title,
        LaunchTarget::uri(format!("steam://rungameid/{id}")),
    )
}

/// Seed the scan cache as if a scan had just completed.
pub fn seed_scan(state: &AppState, games: Vec<Game>, scanned_at: i64) {
    let mut scan = state.lock_scan();
    scan.games = games;
    scan.scanned_at = scanned_at;
    scan.done = true;
}
