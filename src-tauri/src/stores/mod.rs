//! Store integrations. Each module exposes `scan(&ScanContext) -> ScanResult`
//! that discovers games **offline** from the official store app's own local
//! data (files / registry / local databases – no web APIs, no credentials).
//! Launching and installing are always handed back to the official app.
//!
//! Scanners must never panic and must treat a missing store as "0 games",
//! not as an error. Parsing logic is kept in pure functions that take paths
//! or file contents so it can be unit-tested with fixtures on every OS.

pub mod amazon;
pub mod battlenet;
pub mod ea;
pub mod epic;
pub mod gog;
pub mod heroic;
pub mod humble;
pub mod itch;
pub mod lutris;
pub mod origin;
pub mod steam;
pub mod ubisoft;
pub mod xbox;

#[cfg(windows)]
pub mod registry;

use crate::models::{Game, Platform};
use crate::persistence::Settings;
use std::path::{Path, PathBuf};

pub type ScanResult = Result<Vec<Game>, String>;

pub struct ScanContext {
    pub settings: Settings,
    pub home: Option<PathBuf>,
}

impl ScanContext {
    pub fn new(settings: Settings) -> Self {
        ScanContext {
            settings,
            home: dirs::home_dir(),
        }
    }

    /// `$HOME/<rel>` if the home directory is known.
    pub fn home_join(&self, rel: &str) -> Option<PathBuf> {
        self.home.as_ref().map(|h| h.join(rel))
    }
}

/// Platforms discovered by a scanner (everything except user-managed ones).
pub fn scanned_platforms() -> impl Iterator<Item = Platform> {
    Platform::ALL.into_iter().filter(|p| !p.is_user_managed())
}

pub fn scan_platform(platform: Platform, ctx: &ScanContext) -> ScanResult {
    match platform {
        Platform::Steam => steam::scan(ctx),
        Platform::Epic => epic::scan(ctx),
        Platform::Gog => gog::scan(ctx),
        Platform::Humble => humble::scan(ctx),
        Platform::Itch => itch::scan(ctx),
        Platform::Ubisoft => ubisoft::scan(ctx),
        Platform::Ea => ea::scan(ctx),
        Platform::Origin => origin::scan(ctx),
        Platform::Xbox => xbox::scan(ctx),
        Platform::Amazon => amazon::scan(ctx),
        Platform::BattleNet => battlenet::scan(ctx),
        Platform::Lutris => lutris::scan(ctx),
        Platform::GeforceNow | Platform::Xcloud | Platform::Local => Ok(Vec::new()),
    }
}

/// Read a file as UTF-8 (lossy, BOM stripped). `None` if unreadable.
pub fn read_text(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    Some(text.trim_start_matches('\u{feff}').to_string())
}

/// Read and parse a JSON file. `None` if missing or invalid.
pub fn read_json(path: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&read_text(path)?).ok()
}

/// List the entries of a directory (empty if unreadable).
pub fn list_dir(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).collect())
        .unwrap_or_default()
}

/// Open a SQLite database read-only so the owning launcher is never
/// disturbed. Returns `Ok(None)` if the file does not exist.
pub fn open_sqlite_readonly(path: &Path) -> Result<Option<rusqlite::Connection>, String> {
    use rusqlite::OpenFlags;
    if !path.is_file() {
        return Ok(None);
    }
    rusqlite::Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map(Some)
    .map_err(|e| format!("{}: {e}", path.display()))
}

/// `%ProgramData%` on Windows (defaults to `C:\ProgramData`).
#[cfg(windows)]
pub fn program_data() -> PathBuf {
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
}

/// Keep the first occurrence of every id (scanners may find the same game
/// through several sources, e.g. Epic launcher + Heroic).
pub fn dedup(games: &mut Vec<Game>) {
    let mut seen = std::collections::HashSet::new();
    games.retain(|g| seen.insert(g.id.clone()));
}

/// Percent-encode a single URI path/query component.
pub fn encode_component(s: &str) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    utf8_percent_encode(s, NON_ALPHANUMERIC).to_string()
}

#[cfg(test)]
pub(crate) mod test_util {
    use std::fs;
    use std::path::Path;

    pub fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LaunchTarget;

    #[test]
    fn dedup_keeps_first() {
        let mut v = vec![
            Game::new(Platform::Epic, "a", "First", LaunchTarget::uri("x:1")),
            Game::new(Platform::Epic, "a", "Second", LaunchTarget::uri("x:2")),
            Game::new(Platform::Epic, "b", "Other", LaunchTarget::uri("x:3")),
        ];
        dedup(&mut v);
        assert_eq!(
            v.iter().map(|g| g.title.as_str()).collect::<Vec<_>>(),
            ["First", "Other"]
        );
    }

    #[test]
    fn every_scanner_survives_an_empty_home() {
        let dir = tempfile::tempdir().unwrap();
        let ctx = ScanContext {
            settings: Settings::default(),
            home: Some(dir.path().to_path_buf()),
        };
        for p in scanned_platforms() {
            // Must not panic; may find system-wide installs on a dev machine.
            let _ = scan_platform(p, &ctx);
        }
    }

    #[test]
    fn scanner_launch_uris_are_allowed_by_the_launcher() {
        for uri in [
            "steam://rungameid/1",
            "com.epicgames.launcher://apps/x",
            "goggalaxy://openGameView/1",
            "humble://launch/x",
            "itch://caves/x/launch",
            "uplay://launch/1/0",
            "link2ea://launchgame/1",
            "origin2://game/launch?offerIds=1",
            "amazon-games://play/x",
            "heroic://launch?appName=x&runner=gog",
            "lutris:rungameid/1",
        ] {
            assert!(crate::launcher::check_uri(uri).is_ok(), "{uri}");
        }
    }

    #[test]
    fn read_text_strips_bom() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("a.json");
        std::fs::write(&p, "\u{feff}{\"a\":1}").unwrap();
        assert_eq!(read_json(&p).unwrap()["a"], 1);
    }
}
