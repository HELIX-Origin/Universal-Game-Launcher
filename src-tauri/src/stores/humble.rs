//! Humble App (Windows): `%APPDATA%\Humble App\config.json`, array
//! `game-collection-4`. Games launch through the Humble App
//! (`humble://launch/<machineName>`).

use super::{ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use serde_json::Value;
use std::path::PathBuf;

pub fn parse_config(v: &Value) -> Vec<Game> {
    let Some(list) = v["game-collection-4"].as_array() else {
        return Vec::new();
    };
    list.iter()
        .filter(|g| matches!(g["status"].as_str(), Some("downloaded" | "installed")))
        .filter_map(|g| {
            let machine = g["machineName"].as_str().filter(|s| !s.is_empty())?;
            let title = g["gameName"].as_str().filter(|s| !s.trim().is_empty())?;
            let uri = format!("humble://launch/{}", super::encode_component(machine));
            let mut game = Game::new(Platform::Humble, machine, title, LaunchTarget::uri(uri));
            game.install_dir = g["filePath"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(PathBuf::from);
            if let Some(img) = g["imagePath"]
                .as_str()
                .filter(|s| s.starts_with("https://"))
            {
                game.cover_url = Some(img.to_string());
            }
            Some(game)
        })
        .collect()
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let _ = ctx;
    let mut games = Vec::new();
    if cfg!(windows) {
        if let Some(cfg) = dirs::config_dir().map(|d| d.join("Humble App").join("config.json")) {
            if let Some(v) = super::read_json(&cfg) {
                games = parse_config(&v);
            }
        }
    }
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_installed_games_only() {
        let v = json!({"game-collection-4": [
            {"machineName": "dead_cells", "gameName": "Dead Cells", "status": "installed", "filePath": "C:\\Humble\\Dead Cells", "imagePath": "https://img/x.png"},
            {"machineName": "hades", "gameName": "Hades", "status": "available"},
            {"machineName": "", "gameName": "Broken", "status": "downloaded"}
        ]});
        let games = parse_config(&v);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "humble:dead_cells");
        assert_eq!(
            games[0].launch,
            LaunchTarget::uri("humble://launch/dead%5Fcells")
        );
        assert_eq!(games[0].cover_url.as_deref(), Some("https://img/x.png"));
    }
}
