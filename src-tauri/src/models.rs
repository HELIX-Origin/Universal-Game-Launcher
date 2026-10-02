//! Core data model shared by every store integration and the frontend.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Every storefront / source the launcher knows about.
///
/// Serialized as kebab-case strings (e.g. `"epic"`, `"geforce-now"`), which is
/// the exact value used by the TypeScript `Platform` union in `src/lib/types.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    Steam,
    Epic,
    Gog,
    Humble,
    Itch,
    Ubisoft,
    Ea,
    Origin,
    Xbox,
    Amazon,
    BattleNet,
    Lutris,
    GeforceNow,
    Xcloud,
    Local,
}

impl Platform {
    pub const ALL: [Platform; 15] = [
        Platform::Steam,
        Platform::Epic,
        Platform::Gog,
        Platform::Humble,
        Platform::Itch,
        Platform::Ubisoft,
        Platform::Ea,
        Platform::Origin,
        Platform::Xbox,
        Platform::Amazon,
        Platform::BattleNet,
        Platform::Lutris,
        Platform::GeforceNow,
        Platform::Xcloud,
        Platform::Local,
    ];

    /// Stable identifier used as the prefix of game ids.
    pub fn key(self) -> &'static str {
        match self {
            Platform::Steam => "steam",
            Platform::Epic => "epic",
            Platform::Gog => "gog",
            Platform::Humble => "humble",
            Platform::Itch => "itch",
            Platform::Ubisoft => "ubisoft",
            Platform::Ea => "ea",
            Platform::Origin => "origin",
            Platform::Xbox => "xbox",
            Platform::Amazon => "amazon",
            Platform::BattleNet => "battle-net",
            Platform::Lutris => "lutris",
            Platform::GeforceNow => "geforce-now",
            Platform::Xcloud => "xcloud",
            Platform::Local => "local",
        }
    }

    /// Cloud platforms stream games and are never discovered on disk.
    pub fn is_cloud(self) -> bool {
        matches!(self, Platform::GeforceNow | Platform::Xcloud)
    }

    /// Platforms whose entries are created by the user instead of a scanner.
    pub fn is_user_managed(self) -> bool {
        self.is_cloud() || self == Platform::Local
    }
}

/// How a game is started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaunchTarget {
    /// Hand a URI (store protocol or https URL) to the OS.
    Uri { uri: String },
    /// Spawn an executable directly.
    #[serde(rename_all = "camelCase")]
    Executable {
        path: PathBuf,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        working_dir: Option<PathBuf>,
    },
}

impl LaunchTarget {
    pub fn uri(uri: impl Into<String>) -> Self {
        LaunchTarget::Uri { uri: uri.into() }
    }

    pub fn exe(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let working_dir = path.parent().map(|p| p.to_path_buf());
        LaunchTarget::Executable {
            path,
            args: Vec::new(),
            working_dir,
        }
    }
}

/// A single game entry as produced by a scanner (before user metadata is merged in).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Globally unique id: `"<platform-key>:<store id>"`.
    pub id: String,
    pub platform: Platform,
    /// The id inside the store (Steam appid, Epic AppName, ...).
    pub store_id: String,
    pub title: String,
    pub install_dir: Option<PathBuf>,
    /// Starts the game. For store games this always goes through the store's
    /// own client (Steam, Epic, ...), which handles DRM, updates and launching.
    pub launch: LaunchTarget,
    /// `false` for owned games that are known offline but not installed yet.
    pub installed: bool,
    /// Hands installation off to the store client (e.g. `steam://install/<id>`).
    /// Always `None` for local games – those are added from disk, never downloaded.
    pub install: Option<LaunchTarget>,
    /// Remote or local cover art URL (portrait if possible).
    pub cover_url: Option<String>,
    /// Wide hero/banner art URL.
    pub hero_url: Option<String>,
    pub size_bytes: Option<u64>,
}

impl Game {
    pub fn new(
        platform: Platform,
        store_id: impl Into<String>,
        title: impl Into<String>,
        launch: LaunchTarget,
    ) -> Self {
        let store_id = store_id.into();
        Game {
            id: format!("{}:{}", platform.key(), store_id),
            platform,
            store_id,
            title: title.into().trim().to_string(),
            install_dir: None,
            launch,
            installed: true,
            install: None,
            cover_url: None,
            hero_url: None,
            size_bytes: None,
        }
    }

    pub fn with_install_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.install_dir = Some(dir.into());
        self
    }

    /// Mark as owned-but-not-installed; `install` asks the store client to download it.
    pub fn not_installed(mut self, install: LaunchTarget) -> Self {
        self.installed = false;
        self.install = Some(install);
        self
    }

    pub fn with_cover(mut self, url: impl Into<String>) -> Self {
        self.cover_url = Some(url.into());
        self
    }

    pub fn with_hero(mut self, url: impl Into<String>) -> Self {
        self.hero_url = Some(url.into());
        self
    }
}

/// A game as presented to the UI: scanner data + user metadata.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    #[serde(flatten)]
    pub game: Game,
    pub favorite: bool,
    pub hidden: bool,
    /// Unix epoch seconds.
    pub last_played: Option<i64>,
    pub play_count: u32,
    /// `true` for user-created entries (local executables, cloud shortcuts).
    pub custom: bool,
    /// Opt-in metadata from third-party providers (if fetched).
    pub metadata: Option<crate::metadata::GameMetadata>,
}

/// Per-platform result of the last scan (shown in Settings → Platforms).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformStatus {
    pub platform: Platform,
    pub enabled: bool,
    pub game_count: usize,
    pub error: Option<String>,
    /// The official app can be opened ("Open app").
    pub can_open: bool,
    /// The official app's storefront can be opened ("Open store").
    pub can_open_store: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySnapshot {
    pub games: Vec<LibraryEntry>,
    pub platforms: Vec<PlatformStatus>,
    /// Unix epoch seconds of the last completed scan.
    pub scanned_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_serializes_to_kebab_case_matching_key() {
        for p in Platform::ALL {
            let json = serde_json::to_string(&p).unwrap();
            assert_eq!(json, format!("\"{}\"", p.key()));
        }
    }

    #[test]
    fn game_id_is_prefixed_with_platform() {
        let g = Game::new(
            Platform::Steam,
            "570",
            "  Dota 2 ",
            LaunchTarget::uri("steam://rungameid/570"),
        );
        assert_eq!(g.id, "steam:570");
        assert_eq!(g.title, "Dota 2");
    }

    #[test]
    fn launch_target_serializes_with_kind_tag() {
        let json = serde_json::to_value(LaunchTarget::uri("steam://x")).unwrap();
        assert_eq!(json["kind"], "uri");
        let exe = serde_json::to_value(LaunchTarget::exe("/games/a/run")).unwrap();
        assert_eq!(exe["kind"], "executable");
        assert_eq!(exe["workingDir"], "/games/a");
    }
}
