//! Ubisoft Connect (Windows): `HKLM\SOFTWARE\WOW6432Node\Ubisoft\Launcher\Installs\<id>`
//! → `InstallDir`. Titles come from the matching "Uplay Install <id>"
//! uninstall entry. Games launch through Ubisoft Connect (`uplay://launch/<id>/0`).

use super::{ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use std::path::{Path, PathBuf};

pub fn game_from(id: &str, install_dir: &str, display_name: Option<&str>) -> Option<Game> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let dir = PathBuf::from(install_dir.replace('/', std::path::MAIN_SEPARATOR_STR));
    let title = display_name.map(str::to_string).or_else(|| {
        Path::new(install_dir.trim_end_matches(['/', '\\']))
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
    })?;
    Some(
        Game::new(
            Platform::Ubisoft,
            id,
            title,
            LaunchTarget::uri(format!("uplay://launch/{id}/0")),
        )
        .with_install_dir(dir),
    )
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let _ = ctx;
    #[allow(unused_mut)]
    let mut games = Vec::new();
    #[cfg(windows)]
    {
        use super::registry;
        let names: std::collections::HashMap<String, String> = registry::uninstall_entries()
            .into_iter()
            .filter_map(|(k, key)| {
                let id = k.strip_prefix("Uplay Install ")?.to_string();
                Some((id, registry::get_string(&key, "DisplayName")?))
            })
            .collect();
        for (id, key) in registry::hklm_subkeys(r"SOFTWARE\Ubisoft\Launcher\Installs") {
            if let Some(dir) = registry::get_string(&key, "InstallDir") {
                games.extend(game_from(&id, &dir, names.get(&id).map(String::as_str)));
            }
        }
    }
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_games_from_registry_values() {
        let g = game_from(
            "635",
            "C:/Program Files (x86)/Ubisoft/Games/Far Cry 3/",
            None,
        )
        .unwrap();
        assert_eq!(g.title, "Far Cry 3");
        assert_eq!(g.launch, LaunchTarget::uri("uplay://launch/635/0"));
        assert_eq!(
            game_from("635", "C:/x", Some("Far Cry® 3")).unwrap().title,
            "Far Cry® 3"
        );
        assert!(game_from("abc", "C:/x", None).is_none());
    }
}
