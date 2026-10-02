//! Heroic Games Launcher (all OSes). Heroic manages Epic (legendary), GOG
//! (gogdl) and Amazon (nile) games; we surface them under their real store
//! and launch them through Heroic's `heroic://` protocol.

use super::{read_json, ScanContext};
use crate::models::{Game, LaunchTarget, Platform};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub fn config_roots(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(cfg) = dirs::config_dir() {
        roots.push(cfg.join("heroic"));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    roots.extend(ctx.home_join(".var/app/com.heroicgameslauncher.hgl/config/heroic"));
    #[cfg(not(all(unix, not(target_os = "macos"))))]
    let _ = ctx;
    roots.retain(|r| r.is_dir());
    roots
}

pub fn launch_uri(app_name: &str, runner: &str) -> String {
    format!(
        "heroic://launch?appName={}&runner={runner}",
        super::encode_component(app_name)
    )
}

fn folder_title(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
}

/// `legendaryConfig/legendary/installed.json`: map keyed by app_name.
pub fn parse_legendary_installed(v: &Value) -> Vec<Game> {
    let Some(map) = v.as_object() else {
        return Vec::new();
    };
    map.values()
        .filter(|g| g["is_dlc"] != Value::Bool(true))
        .filter_map(|g| {
            let app = g["app_name"].as_str()?;
            let title = g["title"]
                .as_str()
                .map(String::from)
                .or_else(|| g["install_path"].as_str().and_then(folder_title))?;
            let mut game = Game::new(
                Platform::Epic,
                app,
                title,
                LaunchTarget::uri(launch_uri(app, "legendary")),
            );
            game.install_dir = g["install_path"].as_str().map(PathBuf::from);
            game.size_bytes = g["install_size"].as_u64();
            Some(game)
        })
        .collect()
}

/// `gog_store/installed.json`: `{"installed": [{appName, install_path, ...}]}`.
pub fn parse_gog_installed(
    v: &Value,
    titles: &std::collections::HashMap<String, String>,
) -> Vec<Game> {
    let Some(list) = v["installed"].as_array() else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|g| {
            let app = g["appName"].as_str()?;
            let path = g["install_path"].as_str();
            let title = titles
                .get(app)
                .cloned()
                .or_else(|| path.and_then(folder_title))?;
            let mut game = Game::new(
                Platform::Gog,
                app,
                title,
                LaunchTarget::uri(launch_uri(app, "gog")),
            );
            game.install_dir = path.map(PathBuf::from);
            Some(game)
        })
        .collect()
}

/// `nile_config/nile/installed.json`: `[{id, path, version}]`.
pub fn parse_nile_installed(
    v: &Value,
    titles: &std::collections::HashMap<String, String>,
) -> Vec<Game> {
    let Some(list) = v.as_array() else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|g| {
            let id = g["id"].as_str()?;
            let path = g["path"].as_str();
            let title = titles
                .get(id)
                .cloned()
                .or_else(|| path.and_then(folder_title))?;
            let mut game = Game::new(
                Platform::Amazon,
                id,
                title,
                LaunchTarget::uri(launch_uri(id, "nile")),
            );
            game.install_dir = path.map(PathBuf::from);
            Some(game)
        })
        .collect()
}

/// Best-effort title lookup from Heroic's library caches (`app_name`/`id` → title).
fn titles_from(
    v: Option<Value>,
    list_key: Option<&str>,
    id_keys: &[&str],
) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let Some(v) = v else { return map };
    let list = match list_key {
        Some(k) => v[k].clone(),
        None => v,
    };
    for item in list.as_array().into_iter().flatten() {
        let id = id_keys.iter().find_map(|k| item[*k].as_str());
        let title = item["title"]
            .as_str()
            .or_else(|| item["product"]["title"].as_str());
        if let (Some(id), Some(title)) = (id, title) {
            map.insert(id.to_string(), title.to_string());
        }
    }
    map
}

pub fn scan_epic(ctx: &ScanContext) -> Vec<Game> {
    config_roots(ctx)
        .iter()
        .filter_map(|r| read_json(&r.join("legendaryConfig/legendary/installed.json")))
        .flat_map(|v| parse_legendary_installed(&v))
        .collect()
}

pub fn scan_gog(ctx: &ScanContext) -> Vec<Game> {
    let mut games = Vec::new();
    for root in config_roots(ctx) {
        let titles = titles_from(
            read_json(&root.join("store_cache/gog_library.json")),
            Some("games"),
            &["app_name"],
        );
        if let Some(v) = read_json(&root.join("gog_store/installed.json")) {
            games.extend(parse_gog_installed(&v, &titles));
        }
    }
    games
}

pub fn scan_amazon(ctx: &ScanContext) -> Vec<Game> {
    let mut games = Vec::new();
    for root in config_roots(ctx) {
        let titles = titles_from(
            read_json(&root.join("nile_config/nile/library.json")),
            None,
            &["id"],
        );
        if let Some(v) = read_json(&root.join("nile_config/nile/installed.json")) {
            games.extend(parse_nile_installed(&v, &titles));
        }
    }
    games
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn parses_legendary_installed() {
        let v = json!({
            "Fortnite": {"app_name": "Fortnite", "title": "Fortnite", "install_path": "/games/Fortnite", "is_dlc": false, "install_size": 10},
            "SomeDlc": {"app_name": "SomeDlc", "title": "DLC", "is_dlc": true}
        });
        let games = parse_legendary_installed(&v);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "epic:Fortnite");
        assert_eq!(
            games[0].launch,
            LaunchTarget::uri("heroic://launch?appName=Fortnite&runner=legendary")
        );
    }

    #[test]
    fn parses_gog_and_nile_with_title_fallback() {
        let gog =
            json!({"installed": [{"appName": "1207658924", "install_path": "/games/Unreal Gold"}]});
        let g = parse_gog_installed(&gog, &HashMap::new());
        assert_eq!(g[0].title, "Unreal Gold");
        assert_eq!(g[0].platform, Platform::Gog);

        let titles = HashMap::from([("amzn1.x".to_string(), "Amazon Game".to_string())]);
        let n = parse_nile_installed(&json!([{"id": "amzn1.x", "path": "/g/x"}]), &titles);
        assert_eq!(n[0].title, "Amazon Game");
        assert!(n[0].launch == LaunchTarget::uri("heroic://launch?appName=amzn1%2Ex&runner=nile"));
    }

    #[test]
    fn title_cache_supports_both_shapes() {
        let gog = titles_from(
            Some(json!({"games": [{"app_name": "1", "title": "A"}]})),
            Some("games"),
            &["app_name"],
        );
        assert_eq!(gog["1"], "A");
        let nile = titles_from(
            Some(json!([{"id": "x", "product": {"title": "B"}}])),
            None,
            &["id"],
        );
        assert_eq!(nile["x"], "B");
    }
}
