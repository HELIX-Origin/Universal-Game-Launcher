//! Persistent user data: custom games, favorites, hidden games, play history
//! and settings. Stored as JSON in the Tauri app-data directory.

use crate::error::{AppError, AppResult, ErrorCode};
use crate::metadata::{GameMetadata, MetadataProvider};
use crate::models::{Game, LaunchTarget, Platform};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DATA_FILE: &str = "library.json";
const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Platforms the user switched off (not scanned / not shown).
    pub disabled_platforms: BTreeSet<Platform>,
    /// Optional override for the Steam installation directory.
    pub steam_path: Option<PathBuf>,
    /// `"dark"`, `"light"` or `"system"`.
    pub theme: String,
    /// Library layout: `"grid"` or `"list"`.
    pub view_mode: String,
    /// Minimize the launcher window after a game is started.
    pub minimize_on_launch: bool,
    /// BCP-47 UI locale; `None` follows the operating system language.
    pub locale: Option<String>,
    /// Metadata providers the user opted into, in priority order.
    pub metadata_providers: Vec<MetadataProvider>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            disabled_platforms: BTreeSet::new(),
            steam_path: None,
            theme: "dark".into(),
            view_mode: "grid".into(),
            minimize_on_launch: false,
            locale: None,
            metadata_providers: Vec::new(),
        }
    }
}

impl Settings {
    pub fn is_enabled(&self, platform: Platform) -> bool {
        !self.disabled_platforms.contains(&platform)
    }

    /// Normalise values coming from the UI.
    pub fn sanitized(mut self) -> Self {
        if !matches!(self.theme.as_str(), "dark" | "light" | "system") {
            self.theme = "dark".into();
        }
        if !matches!(self.view_mode.as_str(), "grid" | "list") {
            self.view_mode = "grid".into();
        }
        if self
            .steam_path
            .as_ref()
            .is_some_and(|p| p.as_os_str().is_empty())
        {
            self.steam_path = None;
        }
        let valid_locale = |l: &String| {
            !l.is_empty()
                && l.len() <= 16
                && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        };
        if self.locale.as_ref().is_some_and(|l| !valid_locale(l)) {
            self.locale = None;
        }
        let mut seen = BTreeSet::new();
        self.metadata_providers.retain(|p| seen.insert(*p));
        self
    }
}

/// A user-created library entry (local executable or cloud shortcut).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomGame {
    pub id: String,
    pub platform: Platform,
    pub title: String,
    pub launch: LaunchTarget,
    #[serde(default)]
    pub cover_url: Option<String>,
}

impl CustomGame {
    pub fn to_game(&self) -> Game {
        let mut g = Game::new(
            self.platform,
            self.id.clone(),
            self.title.clone(),
            self.launch.clone(),
        );
        if let LaunchTarget::Executable {
            working_dir, path, ..
        } = &self.launch
        {
            g.install_dir = working_dir
                .clone()
                .or_else(|| path.parent().map(Path::to_path_buf));
        }
        g.cover_url = self.cover_url.clone();
        g
    }
}

/// Payload sent by the "Add game" dialog.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomGameInput {
    pub platform: Platform,
    pub title: String,
    /// Local games: path to the executable (or `.app` bundle on macOS).
    pub executable: Option<PathBuf>,
    /// Local games: command-line arguments.
    #[serde(default)]
    pub args: Vec<String>,
    /// Cloud games: https URL that opens the game in the streaming service.
    pub url: Option<String>,
    pub cover_url: Option<String>,
}

/// Hosts accepted for cloud shortcuts. Keeps the "cloud" entries honest and
/// prevents the launcher from becoming a generic URL opener.
pub fn allowed_cloud_hosts(platform: Platform) -> &'static [&'static str] {
    match platform {
        Platform::GeforceNow => &["play.geforcenow.com", "www.nvidia.com", "geforcenow.com"],
        Platform::Xcloud => &["www.xbox.com", "xbox.com"],
        _ => &[],
    }
}

fn url_host(url: &str) -> Option<&str> {
    let rest = url.strip_prefix("https://")?;
    let authority = rest.split(['/', '?', '#']).next()?;
    if authority.contains('@') {
        return None;
    }
    let host = authority.split(':').next()?;
    (!host.is_empty()).then_some(host)
}

fn optional_https(url: Option<String>) -> AppResult<Option<String>> {
    match url.map(|u| u.trim().to_string()).filter(|u| !u.is_empty()) {
        None => Ok(None),
        Some(u) if url_host(&u).is_some() => Ok(Some(u)),
        Some(u) => Err(AppError::with(ErrorCode::InvalidCoverUrl, u)),
    }
}

impl CustomGameInput {
    /// Validate the input and turn it into a [`CustomGame`] with a fresh id.
    pub fn into_custom_game(self) -> AppResult<CustomGame> {
        let title = self.title.trim().to_string();
        if title.is_empty() {
            return Err(ErrorCode::TitleRequired.into());
        }
        if title.chars().count() > 200 {
            return Err(ErrorCode::TitleTooLong.into());
        }
        let cover_url = optional_https(self.cover_url)?;
        let launch = match self.platform {
            Platform::Local => {
                let path = self.executable.ok_or(ErrorCode::ExecutableRequired)?;
                if !path.is_absolute() {
                    return Err(AppError::with(
                        ErrorCode::ExecutableNotAbsolute,
                        path.display().to_string(),
                    ));
                }
                let is_app_bundle = path.extension().is_some_and(|e| e == "app") && path.is_dir();
                if !path.is_file() && !is_app_bundle {
                    return Err(AppError::with(
                        ErrorCode::ExecutableNotFound,
                        path.display().to_string(),
                    ));
                }
                LaunchTarget::Executable {
                    working_dir: path.parent().map(Path::to_path_buf),
                    path,
                    args: self.args.into_iter().filter(|a| !a.is_empty()).collect(),
                }
            }
            p if p.is_cloud() => {
                let url = self
                    .url
                    .map(|u| u.trim().to_string())
                    .filter(|u| !u.is_empty())
                    .ok_or(ErrorCode::UrlRequired)?;
                let host = url_host(&url)
                    .ok_or_else(|| AppError::with(ErrorCode::UrlNotHttps, url.clone()))?;
                let allowed = allowed_cloud_hosts(p);
                if !allowed.iter().any(|h| host.eq_ignore_ascii_case(h)) {
                    return Err(AppError::with(
                        ErrorCode::UrlHostNotAllowed,
                        allowed.join(", "),
                    ));
                }
                LaunchTarget::uri(url)
            }
            _ => return Err(ErrorCode::UnsupportedPlatform.into()),
        };
        Ok(CustomGame {
            id: new_custom_id(),
            platform: self.platform,
            title,
            launch,
            cover_url,
        })
    }
}

fn new_custom_id() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}{:x}", nanos, COUNTER.fetch_add(1, Ordering::Relaxed))
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UserData {
    pub version: u32,
    pub custom_games: Vec<CustomGame>,
    pub favorites: BTreeSet<String>,
    pub hidden: BTreeSet<String>,
    pub last_played: BTreeMap<String, i64>,
    pub play_count: BTreeMap<String, u32>,
    /// Enriched metadata fetched from opt-in providers, by game id.
    pub metadata: BTreeMap<String, GameMetadata>,
    pub settings: Settings,
}

impl Default for UserData {
    fn default() -> Self {
        UserData {
            version: CURRENT_VERSION,
            custom_games: Vec::new(),
            favorites: BTreeSet::new(),
            hidden: BTreeSet::new(),
            last_played: BTreeMap::new(),
            play_count: BTreeMap::new(),
            metadata: BTreeMap::new(),
            settings: Settings::default(),
        }
    }
}

impl UserData {
    /// Load from disk. A missing file yields defaults; a corrupt file is
    /// backed up (`library.json.bak`) and replaced by defaults so the app
    /// always starts.
    pub fn load(path: &Path) -> UserData {
        match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str::<UserData>(&text) {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("corrupt user data at {}: {e}", path.display());
                    let _ = fs::copy(path, path.with_extension("json.bak"));
                    UserData::default()
                }
            },
            Err(_) => UserData::default(),
        }
    }

    /// Atomically write to disk (write temp file, then rename).
    pub fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| {
                AppError::with(ErrorCode::Storage, format!("{}: {e}", dir.display()))
            })?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::with(ErrorCode::Internal, e.to_string()))?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json)
            .map_err(|e| AppError::with(ErrorCode::Storage, format!("{}: {e}", tmp.display())))?;
        fs::rename(&tmp, path)
            .map_err(|e| AppError::with(ErrorCode::Storage, format!("{}: {e}", path.display())))
    }

    pub fn record_launch(&mut self, id: &str) {
        self.last_played.insert(id.to_string(), now_secs());
        *self.play_count.entry(id.to_string()).or_insert(0) += 1;
    }

    pub fn remove_custom(&mut self, id: &str) -> bool {
        let before = self.custom_games.len();
        self.custom_games.retain(|g| g.to_game().id != id);
        let removed = self.custom_games.len() != before;
        if removed {
            self.favorites.remove(id);
            self.hidden.remove(id);
            self.last_played.remove(id);
            self.play_count.remove(id);
            self.metadata.remove(id);
        }
        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(platform: Platform) -> CustomGameInput {
        CustomGameInput {
            platform,
            title: "Test".into(),
            executable: None,
            args: vec![],
            url: None,
            cover_url: None,
        }
    }

    #[test]
    fn roundtrips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join(DATA_FILE);
        let mut data = UserData::default();
        data.favorites.insert("steam:570".into());
        data.record_launch("steam:570");
        data.settings.disabled_platforms.insert(Platform::Origin);
        data.save(&path).unwrap();
        let loaded = UserData::load(&path);
        assert_eq!(loaded, data);
        assert_eq!(loaded.play_count["steam:570"], 1);
    }

    #[test]
    fn corrupt_file_is_backed_up_and_defaults_used() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(DATA_FILE);
        fs::write(&path, "{not json").unwrap();
        assert_eq!(UserData::load(&path), UserData::default());
        assert!(path.with_extension("json.bak").exists());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let data: UserData = serde_json::from_str(r#"{"favorites":["a"]}"#).unwrap();
        assert!(data.favorites.contains("a"));
        assert_eq!(data.settings.theme, "dark");
    }

    #[test]
    fn validates_local_games() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("game.exe");
        fs::write(&exe, "").unwrap();

        let mut ok = input(Platform::Local);
        ok.executable = Some(exe.clone());
        ok.args = vec!["-windowed".into(), "".into()];
        let game = ok.into_custom_game().unwrap();
        assert_eq!(
            game.launch,
            LaunchTarget::Executable {
                path: exe,
                args: vec!["-windowed".into()],
                working_dir: Some(dir.path().into())
            }
        );
        assert!(game.to_game().id.starts_with("local:"));

        let mut missing = input(Platform::Local);
        missing.executable = Some(dir.path().join("nope.exe"));
        assert!(missing.into_custom_game().is_err());

        let mut relative = input(Platform::Local);
        relative.executable = Some(PathBuf::from("game.exe"));
        assert_eq!(
            relative.into_custom_game().unwrap_err().code,
            ErrorCode::ExecutableNotAbsolute
        );

        let mut untitled = input(Platform::Local);
        untitled.title = "   ".into();
        assert_eq!(
            untitled.into_custom_game().unwrap_err().code,
            ErrorCode::TitleRequired
        );
    }

    #[test]
    fn validates_cloud_urls() {
        let mut gfn = input(Platform::GeforceNow);
        gfn.url = Some("https://play.geforcenow.com/mall/#/deeplink?game-id=123".into());
        assert!(gfn.into_custom_game().is_ok());

        let mut xcloud = input(Platform::Xcloud);
        xcloud.url = Some("https://www.xbox.com/en-US/play/games/halo/9NP1P1WFS0LB".into());
        assert!(xcloud.into_custom_game().is_ok());

        for bad in [
            "http://play.geforcenow.com/",
            "https://evil.example.com/",
            "https://play.geforcenow.com@evil.example.com/",
            "file:///etc/passwd",
            "javascript:alert(1)",
        ] {
            let mut i = input(Platform::GeforceNow);
            i.url = Some(bad.into());
            assert!(i.into_custom_game().is_err(), "{bad} should be rejected");
        }

        let mut bad_cover = input(Platform::Xcloud);
        bad_cover.url = Some("https://www.xbox.com/play".into());
        bad_cover.cover_url = Some("javascript:alert(1)".into());
        assert!(bad_cover.into_custom_game().is_err());
    }

    #[test]
    fn store_platforms_cannot_be_added_manually() {
        let mut steam = input(Platform::Steam);
        steam.url = Some("https://store.steampowered.com".into());
        assert!(steam.into_custom_game().is_err());
    }

    #[test]
    fn removing_custom_game_clears_metadata() {
        let mut data = UserData::default();
        let mut i = input(Platform::Xcloud);
        i.url = Some("https://www.xbox.com/play".into());
        let g = i.into_custom_game().unwrap();
        let id = g.to_game().id;
        data.custom_games.push(g);
        data.favorites.insert(id.clone());
        data.record_launch(&id);
        assert!(data.remove_custom(&id));
        assert!(
            data.custom_games.is_empty() && data.favorites.is_empty() && data.play_count.is_empty()
        );
        assert!(!data.remove_custom(&id));
    }

    #[test]
    fn settings_are_sanitized() {
        let s = Settings {
            theme: "neon".into(),
            view_mode: "3d".into(),
            steam_path: Some(PathBuf::new()),
            locale: Some("../../etc".into()),
            ..Settings::default()
        }
        .sanitized();
        assert_eq!(s.locale, None);
        assert_eq!(
            Settings {
                locale: Some("pt-BR".into()),
                ..Settings::default()
            }
            .sanitized()
            .locale
            .as_deref(),
            Some("pt-BR")
        );
        assert_eq!(s.theme, "dark");
        assert_eq!(s.view_mode, "grid");
        assert_eq!(s.steam_path, None);
    }
}
