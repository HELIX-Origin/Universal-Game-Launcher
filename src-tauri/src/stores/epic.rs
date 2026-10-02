//! Epic Games Store: the Epic Games Launcher writes one JSON `.item`
//! manifest per installed app. Games launch through the launcher's
//! `com.epicgames.launcher://` protocol. Heroic-managed Epic games (any OS)
//! are included and launched through Heroic.

use super::{heroic, list_dir, read_json, ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use serde_json::Value;
use std::path::PathBuf;

fn manifest_dirs(ctx: &ScanContext) -> Vec<PathBuf> {
    #[allow(unused_mut)]
    let mut dirs = Vec::new();
    #[cfg(windows)]
    dirs.push(super::program_data().join(r"Epic\EpicGamesLauncher\Data\Manifests"));
    #[cfg(target_os = "macos")]
    dirs.extend(ctx.home_join("Library/Application Support/Epic/EpicGamesLauncher/Data/Manifests"));
    let _ = ctx;
    dirs
}

pub fn launch_uri(namespace: Option<&str>, item_id: Option<&str>, app_name: &str) -> String {
    use super::encode_component as enc;
    match (namespace, item_id) {
        (Some(ns), Some(id)) if !ns.is_empty() && !id.is_empty() => {
            format!(
                "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
                enc(ns),
                enc(id),
                enc(app_name)
            )
        }
        _ => format!(
            "com.epicgames.launcher://apps/{}?action=launch&silent=true",
            enc(app_name)
        ),
    }
}

/// Parse one `.item` manifest; `None` for DLC, engine plugins, incomplete installs.
pub fn parse_item(v: &Value) -> Option<Game> {
    let app_name = v["AppName"].as_str()?;
    if v["bIsIncompleteInstall"] == Value::Bool(true) {
        return None;
    }
    let categories: Vec<&str> = v["AppCategories"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let is_dlc = categories.contains(&"addons") && !categories.contains(&"addons/launchable");
    let is_plugin = categories
        .iter()
        .any(|c| *c == "plugins" || *c == "plugins/engine");
    let ue_plugin = v["CompatibleApps"].as_array().is_some_and(|a| {
        a.iter()
            .filter_map(Value::as_str)
            .any(|s| s.starts_with("UE_"))
    });
    let is_dlc_of_other = v["MainGameAppName"]
        .as_str()
        .is_some_and(|m| !m.is_empty() && m != app_name);
    if is_dlc || is_plugin || ue_plugin || is_dlc_of_other {
        return None;
    }
    let title = v["DisplayName"]
        .as_str()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(app_name);
    let uri = launch_uri(
        v["CatalogNamespace"].as_str(),
        v["CatalogItemId"].as_str(),
        app_name,
    );
    let mut game = Game::new(Platform::Epic, app_name, title, LaunchTarget::uri(uri));
    game.install_dir = v["InstallLocation"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    game.size_bytes = v["InstallSize"].as_u64().filter(|s| *s > 0);
    Some(game)
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let mut games = Vec::new();
    for dir in manifest_dirs(ctx) {
        for file in list_dir(&dir) {
            if file
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("item"))
            {
                games.extend(read_json(&file).as_ref().and_then(parse_item));
            }
        }
    }
    games.extend(heroic::scan_epic(ctx));
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_games_and_skips_dlc_and_plugins() {
        let game = json!({"AppName": "Sugar", "DisplayName": "Rocket League", "CatalogNamespace": "9773aa1a", "CatalogItemId": "530145df",
            "InstallLocation": "C:\\Games\\rocketleague", "AppCategories": ["public", "games", "applications"]});
        let g = parse_item(&game).unwrap();
        assert_eq!(g.id, "epic:Sugar");
        assert_eq!(g.title, "Rocket League");
        assert_eq!(
            g.launch,
            LaunchTarget::uri("com.epicgames.launcher://apps/9773aa1a%3A530145df%3ASugar?action=launch&silent=true")
        );

        assert!(parse_item(&json!({"AppName": "d", "AppCategories": ["addons"]})).is_none());
        assert!(parse_item(
            &json!({"AppName": "d", "AppCategories": ["addons", "addons/launchable"]})
        )
        .is_some());
        assert!(parse_item(
            &json!({"AppName": "p", "AppCategories": ["plugins", "plugins/engine"]})
        )
        .is_none());
        assert!(parse_item(&json!({"AppName": "p", "CompatibleApps": ["UE_5.3"]})).is_none());
        assert!(parse_item(&json!({"AppName": "i", "bIsIncompleteInstall": true})).is_none());
        assert!(parse_item(&json!({"AppName": "x", "MainGameAppName": "Other"})).is_none());
    }

    #[test]
    fn legacy_launch_uri_without_catalog_ids() {
        assert_eq!(
            launch_uri(None, None, "Fortnite"),
            "com.epicgames.launcher://apps/Fortnite?action=launch&silent=true"
        );
    }
}
