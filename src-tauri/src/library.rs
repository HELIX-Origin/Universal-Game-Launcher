//! In-memory library state: last scan results + persisted user data.

use crate::clients;
use crate::error::{AppError, AppResult, ErrorCode};
use crate::metadata::ApiKeys;
use crate::models::{Game, LibraryEntry, LibrarySnapshot, Platform, PlatformStatus};
use crate::persistence::{now_secs, Settings, UserData};
use crate::stores::{self, ScanContext};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Default, Clone)]
pub struct ScanCache {
    pub games: Vec<Game>,
    pub errors: BTreeMap<Platform, String>,
    pub scanned_at: i64,
    pub done: bool,
}

pub struct AppState {
    pub data_dir: PathBuf,
    pub data: Mutex<UserData>,
    pub keys: Mutex<ApiKeys>,
    pub scan: Mutex<ScanCache>,
}

/// Run every enabled scanner in parallel.
pub fn scan_all(settings: &Settings) -> ScanCache {
    let ctx = ScanContext::new(settings.clone());
    let platforms: Vec<Platform> = stores::scanned_platforms()
        .filter(|p| settings.is_enabled(*p))
        .collect();
    let results: Vec<(Platform, stores::ScanResult)> = std::thread::scope(|s| {
        let handles: Vec<_> = platforms
            .iter()
            .map(|&p| {
                let ctx = &ctx;
                (p, s.spawn(move || stores::scan_platform(p, ctx)))
            })
            .collect();
        handles
            .into_iter()
            .map(|(p, h)| {
                (
                    p,
                    h.join()
                        .unwrap_or_else(|_| Err("scanner panicked".to_string())),
                )
            })
            .collect()
    });
    let mut cache = ScanCache {
        scanned_at: now_secs(),
        done: true,
        ..Default::default()
    };
    for (p, r) in results {
        match r {
            Ok(games) => cache.games.extend(games),
            Err(e) => {
                log::warn!("{p:?} scan failed: {e}");
                cache.errors.insert(p, e);
            }
        }
    }
    stores::dedup(&mut cache.games);
    cache
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        let data = UserData::load(&data_dir.join(crate::persistence::DATA_FILE));
        let keys = ApiKeys::load(&data_dir.join(crate::metadata::SECRETS_FILE));
        AppState {
            data_dir,
            data: Mutex::new(data),
            keys: Mutex::new(keys),
            scan: Mutex::new(ScanCache::default()),
        }
    }

    pub fn data_file(&self) -> PathBuf {
        self.data_dir.join(crate::persistence::DATA_FILE)
    }

    pub fn secrets_file(&self) -> PathBuf {
        self.data_dir.join(crate::metadata::SECRETS_FILE)
    }

    pub fn lock_data(&self) -> std::sync::MutexGuard<'_, UserData> {
        self.data.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn lock_scan(&self) -> std::sync::MutexGuard<'_, ScanCache> {
        self.scan.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn lock_keys(&self) -> std::sync::MutexGuard<'_, ApiKeys> {
        self.keys.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Mutate user data and persist it.
    pub fn update<T>(&self, f: impl FnOnce(&mut UserData) -> T) -> AppResult<T> {
        let mut data = self.lock_data();
        let out = f(&mut data);
        data.save(&self.data_file())?;
        Ok(out)
    }

    /// Find a game by id in the last scan or among custom games.
    pub fn find_game(&self, id: &str) -> AppResult<Game> {
        if let Some(g) = self.lock_scan().games.iter().find(|g| g.id == id) {
            return Ok(g.clone());
        }
        self.lock_data()
            .custom_games
            .iter()
            .map(|c| c.to_game())
            .find(|g| g.id == id)
            .ok_or_else(|| AppError::with(ErrorCode::GameNotFound, id))
    }

    pub fn entry(&self, game: Game, data: &UserData, custom: bool) -> LibraryEntry {
        LibraryEntry {
            favorite: data.favorites.contains(&game.id),
            hidden: data.hidden.contains(&game.id),
            last_played: data.last_played.get(&game.id).copied(),
            play_count: data.play_count.get(&game.id).copied().unwrap_or(0),
            metadata: data.metadata.get(&game.id).cloned(),
            custom,
            game,
        }
    }

    pub fn snapshot(&self) -> LibrarySnapshot {
        let scan = self.lock_scan().clone();
        let data = self.lock_data().clone();
        let settings = &data.settings;
        let mut games: Vec<LibraryEntry> = scan
            .games
            .into_iter()
            .filter(|g| settings.is_enabled(g.platform))
            .map(|g| self.entry(g, &data, false))
            .collect();
        games.extend(
            data.custom_games
                .iter()
                .filter(|c| settings.is_enabled(c.platform))
                .map(|c| self.entry(c.to_game(), &data, true)),
        );
        let platforms = Platform::ALL
            .into_iter()
            .map(|p| {
                let actions = clients::actions(p);
                PlatformStatus {
                    platform: p,
                    enabled: settings.is_enabled(p),
                    game_count: games.iter().filter(|e| e.game.platform == p).count(),
                    error: scan.errors.get(&p).cloned(),
                    can_open: actions.open.is_some(),
                    can_open_store: actions.store.is_some(),
                }
            })
            .collect();
        LibrarySnapshot {
            games,
            platforms,
            scanned_at: scan.scanned_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LaunchTarget;
    use crate::persistence::CustomGameInput;

    fn state() -> (tempfile::TempDir, AppState) {
        let dir = tempfile::tempdir().unwrap();
        let s = AppState::new(dir.path().to_path_buf());
        (dir, s)
    }

    #[test]
    fn snapshot_merges_scan_custom_and_user_data() {
        let (_d, s) = state();
        s.lock_scan().games.push(Game::new(
            Platform::Steam,
            "1",
            "Scanned",
            LaunchTarget::uri("steam://rungameid/1"),
        ));
        s.lock_scan().games.push(Game::new(
            Platform::Origin,
            "2",
            "Disabled",
            LaunchTarget::uri("origin2://x"),
        ));
        let custom = CustomGameInput {
            platform: Platform::Xcloud,
            title: "Cloud".into(),
            executable: None,
            args: vec![],
            url: Some("https://www.xbox.com/play/games/x/1".into()),
            cover_url: None,
        }
        .into_custom_game()
        .unwrap();
        let custom_id = custom.to_game().id;
        s.update(|d| {
            d.custom_games.push(custom);
            d.favorites.insert("steam:1".into());
            d.settings.disabled_platforms.insert(Platform::Origin);
        })
        .unwrap();

        let snap = s.snapshot();
        assert_eq!(snap.games.len(), 2);
        let steam = snap.games.iter().find(|e| e.game.id == "steam:1").unwrap();
        assert!(steam.favorite && !steam.custom);
        assert!(snap
            .games
            .iter()
            .any(|e| e.game.id == custom_id && e.custom));
        let origin = snap
            .platforms
            .iter()
            .find(|p| p.platform == Platform::Origin)
            .unwrap();
        assert!(!origin.enabled);
        assert_eq!(origin.game_count, 0);
        assert!(
            snap.platforms
                .iter()
                .find(|p| p.platform == Platform::Steam)
                .unwrap()
                .can_open
        );

        assert!(s.find_game("steam:1").is_ok());
        assert!(s.find_game(&custom_id).is_ok());
        assert_eq!(
            s.find_game("nope").unwrap_err().code,
            ErrorCode::GameNotFound
        );
    }

    #[test]
    fn update_persists_to_disk() {
        let (d, s) = state();
        s.update(|data| data.record_launch("steam:1")).unwrap();
        let reloaded = AppState::new(d.path().to_path_buf());
        assert_eq!(reloaded.lock_data().play_count["steam:1"], 1);
    }

    #[test]
    fn scan_all_respects_disabled_platforms() {
        let mut settings = Settings::default();
        for p in Platform::ALL {
            settings.disabled_platforms.insert(p);
        }
        let cache = scan_all(&settings);
        assert!(cache.done && cache.games.is_empty() && cache.errors.is_empty());
    }
}
