//! Steam: `steamapps/libraryfolders.vdf` lists library roots, each holding
//! `appmanifest_<appid>.acf` files. Games launch through the Steam client
//! (`steam://rungameid/<appid>`).

use super::{list_dir, read_text, ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use crate::vdf;
use std::path::{Path, PathBuf};

/// `StateFlags` bit meaning the app is fully installed.
const STATE_FULLY_INSTALLED: u64 = 4;

/// Steam tools/runtimes that are not games.
const NON_GAME_APPIDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "1070560", // Steam Linux Runtime 1.0 (scout)
    "1391110", // Steam Linux Runtime 2.0 (soldier)
    "1628350", // Steam Linux Runtime 3.0 (sniper)
    "4183110", // Steam Linux Runtime 4.0
    "1493710", // Proton Experimental
    "2180100", // Proton Hotfix
    "1826330", // Proton EasyAntiCheat Runtime
    "1161040", // Proton BattlEye Runtime
    "250820",  // SteamVR
];

pub fn cover_url(appid: &str) -> String {
    format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_600x900.jpg")
}

pub fn hero_url(appid: &str) -> String {
    format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/library_hero.jpg")
}

fn candidate_roots(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(p) = &ctx.settings.steam_path {
        roots.push(p.clone());
    }
    #[cfg(windows)]
    {
        use super::registry;
        if let Some(p) = registry::hkcu_string(r"Software\Valve\Steam", "SteamPath") {
            roots.push(PathBuf::from(p));
        }
        if let Some(p) = registry::hklm_string(r"SOFTWARE\WOW6432Node\Valve\Steam", "InstallPath") {
            roots.push(PathBuf::from(p));
        }
        roots.push(PathBuf::from(r"C:\Program Files (x86)\Steam"));
    }
    #[cfg(target_os = "macos")]
    roots.extend(ctx.home_join("Library/Application Support/Steam"));
    #[cfg(all(unix, not(target_os = "macos")))]
    for rel in [
        ".local/share/Steam",
        ".steam/steam",
        ".steam/root",
        ".var/app/com.valvesoftware.Steam/.local/share/Steam",
        "snap/steam/common/.local/share/Steam",
    ] {
        roots.extend(ctx.home_join(rel));
    }
    roots
}

/// Library folders listed in `libraryfolders.vdf` (both the modern nested
/// format and the legacy `"1" "D:\\Lib"` format), plus the root itself.
pub fn library_folders(steam_root: &Path) -> Vec<PathBuf> {
    let mut libs = vec![steam_root.to_path_buf()];
    let file = steam_root.join("steamapps").join("libraryfolders.vdf");
    if let Some(doc) = read_text(&file).and_then(|t| vdf::parse(&t).ok()) {
        if let Some(map) = doc.get("libraryfolders").and_then(|v| v.as_object()) {
            for value in map.values() {
                let path = match value {
                    vdf::Vdf::Value(p) => Some(p.as_str()),
                    vdf::Vdf::Object(_) => value.str("path"),
                };
                if let Some(p) = path {
                    libs.push(PathBuf::from(p));
                }
            }
        }
    }
    libs
}

/// Parse one `appmanifest_*.acf`. Returns `None` for non-games and
/// incomplete installs.
pub fn parse_manifest(text: &str, library: &Path) -> Option<Game> {
    let doc = vdf::parse(text).ok()?;
    let app = doc.get("AppState")?;
    let appid = app.str("appid")?.trim();
    if appid.is_empty() || NON_GAME_APPIDS.contains(&appid) {
        return None;
    }
    let name = app.str("name").filter(|n| !n.trim().is_empty())?;
    if name.starts_with("Proton ") || name.starts_with("Steam Linux Runtime") {
        return None;
    }
    let flags: u64 = app
        .str("StateFlags")
        .and_then(|f| f.parse().ok())
        .unwrap_or(STATE_FULLY_INSTALLED);
    if flags & STATE_FULLY_INSTALLED == 0 {
        return None;
    }
    let mut game = Game::new(
        Platform::Steam,
        appid,
        name,
        LaunchTarget::uri(format!("steam://rungameid/{appid}")),
    )
    .with_cover(cover_url(appid))
    .with_hero(hero_url(appid));
    if let Some(dir) = app.str("installdir") {
        game.install_dir = Some(library.join("steamapps").join("common").join(dir));
    }
    game.size_bytes = app
        .str("SizeOnDisk")
        .and_then(|s| s.parse().ok())
        .filter(|s| *s > 0);
    Some(game)
}

pub fn scan_root(steam_root: &Path) -> Vec<Game> {
    let mut games = Vec::new();
    let mut seen_libs = std::collections::HashSet::new();
    for lib in library_folders(steam_root) {
        let key = lib.canonicalize().unwrap_or_else(|_| lib.clone());
        if !seen_libs.insert(key) {
            continue;
        }
        for file in list_dir(&lib.join("steamapps")) {
            let is_manifest = file
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("appmanifest_") && n.ends_with(".acf"));
            if is_manifest {
                if let Some(g) = read_text(&file).and_then(|t| parse_manifest(&t, &lib)) {
                    games.push(g);
                }
            }
        }
    }
    games
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let mut games = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for root in candidate_roots(ctx) {
        if !root.join("steamapps").is_dir() {
            continue;
        }
        let key = root.canonicalize().unwrap_or_else(|_| root.clone());
        if seen.insert(key) {
            games.extend(scan_root(&root));
        }
    }
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::test_util::write;

    fn manifest(appid: &str, name: &str, flags: u32) -> String {
        format!(
            "\"AppState\"\n{{\n\t\"appid\"\t\t\"{appid}\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\t\"{flags}\"\n\t\"installdir\"\t\t\"{name}\"\n\t\"SizeOnDisk\"\t\t\"1024\"\n}}"
        )
    }

    #[test]
    fn scans_all_library_folders() {
        let root = tempfile::tempdir().unwrap();
        let extra = tempfile::tempdir().unwrap();
        let extra_path = extra.path().to_string_lossy().replace('\\', "\\\\");
        write(
            &root.path().join("steamapps/libraryfolders.vdf"),
            &format!("\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{extra_path}\"\n\t\t\"apps\" {{ \"620\" \"1\" }}\n\t}}\n}}"),
        );
        write(
            &root.path().join("steamapps/appmanifest_570.acf"),
            &manifest("570", "Dota 2", 4),
        );
        write(
            &root.path().join("steamapps/appmanifest_228980.acf"),
            &manifest("228980", "Steamworks Common Redistributables", 4),
        );
        write(
            &root.path().join("steamapps/appmanifest_1493710.acf"),
            &manifest("1493710", "Proton Experimental", 4),
        );
        write(
            &root.path().join("steamapps/appmanifest_999.acf"),
            &manifest("999", "Downloading", 1026 & !4),
        );
        write(
            &extra.path().join("steamapps/appmanifest_620.acf"),
            &manifest("620", "Portal 2", 4),
        );
        write(&extra.path().join("steamapps/notamanifest.txt"), "x");

        let mut games = scan_root(root.path());
        games.sort_by(|a, b| a.title.cmp(&b.title));
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].title, "Dota 2");
        assert_eq!(games[0].id, "steam:570");
        assert_eq!(games[0].launch, LaunchTarget::uri("steam://rungameid/570"));
        assert_eq!(games[0].size_bytes, Some(1024));
        assert_eq!(
            games[0].install_dir.as_deref(),
            Some(root.path().join("steamapps/common/Dota 2").as_path())
        );
        assert!(games[0]
            .cover_url
            .as_deref()
            .unwrap()
            .contains("/570/library_600x900.jpg"));
        assert_eq!(games[1].title, "Portal 2");
    }

    #[test]
    fn legacy_library_format_is_supported() {
        let root = tempfile::tempdir().unwrap();
        write(
            &root.path().join("steamapps/libraryfolders.vdf"),
            "\"LibraryFolders\" { \"TimeNextStatsReport\" \"1\" \"1\" \"/mnt/games\" }",
        );
        let libs = library_folders(root.path());
        assert!(libs.contains(&PathBuf::from("/mnt/games")));
    }

    #[test]
    fn scan_uses_settings_override() {
        let root = tempfile::tempdir().unwrap();
        write(
            &root.path().join("steamapps/appmanifest_10.acf"),
            &manifest("10", "Counter-Strike", 4),
        );
        let mut ctx = ScanContext {
            settings: Default::default(),
            home: None,
        };
        ctx.settings.steam_path = Some(root.path().to_path_buf());
        let games = scan(&ctx).unwrap();
        assert!(games.iter().any(|g| g.id == "steam:10"));
    }
}
