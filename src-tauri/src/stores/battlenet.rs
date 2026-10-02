//! Battle.net (Windows): installed Blizzard / Activision games appear as
//! uninstall entries whose `UninstallString` carries `--uid=<uid>`. Games
//! launch through the Battle.net app (`Battle.net.exe --exec="launch <code>"`).

use super::{ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use std::path::{Path, PathBuf};

/// Battle.net agent uid prefix → product code used by `--exec="launch <code>"`.
const PRODUCTS: &[(&str, &str)] = &[
    ("wow", "WoW"),
    ("diablo3", "D3"),
    ("s2", "S2"),
    ("s1", "S1"),
    ("hs_beta", "WTCG"),
    ("heroes", "Hero"),
    ("prometheus", "Pro"),
    ("viper", "VIPR"),
    ("odin", "ODIN"),
    ("zeus", "ZEUS"),
    ("osi", "OSI"),
    ("fenris", "Fen"),
    ("fen", "Fen"),
    ("anbs", "ANBS"),
    ("auks", "AUKS"),
    ("w3", "W3"),
];

pub fn uid_from_uninstall(uninstall: &str) -> Option<String> {
    let start = uninstall.find("--uid=")? + "--uid=".len();
    let uid: String = uninstall[start..]
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != '"')
        .collect();
    (!uid.is_empty()).then_some(uid)
}

pub fn product_code(uid: &str) -> Option<&'static str> {
    let lower = uid.to_ascii_lowercase();
    // Longest matching prefix wins (e.g. "fenris" before "fen").
    PRODUCTS
        .iter()
        .filter(|(p, _)| lower == *p || lower.starts_with(&format!("{p}_")))
        .max_by_key(|(p, _)| p.len())
        .map(|(_, c)| *c)
}

pub fn game_from(
    display_name: &str,
    uninstall: &str,
    install_dir: Option<&str>,
    client: &Path,
) -> Option<Game> {
    let name = display_name.trim();
    if name.is_empty()
        || name.ends_with("Test")
        || name.ends_with("Beta")
        || !uninstall.contains("Battle.net")
    {
        return None;
    }
    let uid = uid_from_uninstall(uninstall)?;
    if uid.eq_ignore_ascii_case("battle.net") {
        return None;
    }
    let code = product_code(&uid)?;
    let launch = LaunchTarget::Executable {
        path: client.to_path_buf(),
        args: vec![format!("--exec=launch {code}")],
        working_dir: client.parent().map(Path::to_path_buf),
    };
    let mut g = Game::new(Platform::BattleNet, code, name, launch);
    g.install_dir = install_dir.filter(|d| !d.is_empty()).map(PathBuf::from);
    Some(g)
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let _ = ctx;
    #[allow(unused_mut)]
    let mut games = Vec::new();
    #[cfg(windows)]
    {
        use super::registry::{get_string, uninstall_entries};
        let entries = uninstall_entries();
        let client = entries
            .iter()
            .find(|(_, k)| {
                get_string(k, "UninstallString").is_some_and(|u| u.contains("--uid=battle.net"))
            })
            .and_then(|(_, k)| get_string(k, "InstallLocation"))
            .map(|d| PathBuf::from(d).join("Battle.net.exe"))
            .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)\Battle.net\Battle.net.exe"));
        if client.is_file() {
            for (_, key) in &entries {
                let (Some(name), Some(un)) = (
                    get_string(key, "DisplayName"),
                    get_string(key, "UninstallString"),
                ) else {
                    continue;
                };
                games.extend(game_from(
                    &name,
                    &un,
                    get_string(key, "InstallLocation").as_deref(),
                    &client,
                ));
            }
        }
        super::dedup(&mut games);
    }
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_uninstall_entries_to_games() {
        let client = PathBuf::from(r"C:\BNet\Battle.net.exe");
        let un = r#""C:\ProgramData\Battle.net\Agent\Blizzard Uninstaller.exe" --lang=enUS --uid=fenris --displayname="Diablo IV""#;
        let g = game_from("Diablo IV", un, Some(r"C:\Games\Diablo IV"), &client).unwrap();
        assert_eq!(g.id, "battle-net:Fen");
        match g.launch {
            LaunchTarget::Executable { args, .. } => assert_eq!(args, ["--exec=launch Fen"]),
            other => panic!("{other:?}"),
        }
        let wow = r#""C:\ProgramData\Battle.net\Agent\Blizzard Uninstaller.exe" --uid=wow_enus --lang=enUS"#;
        assert_eq!(
            game_from("World of Warcraft", wow, None, &client)
                .unwrap()
                .store_id,
            "WoW"
        );
        assert!(game_from("Some Beta", wow, None, &client).is_none());
        assert!(game_from("Battle.net", "Battle.net --uid=battle.net", None, &client).is_none());
        assert!(game_from("Unknown", "Battle.net --uid=mystery", None, &client).is_none());
        assert!(game_from("Not Blizzard", "uninstall.exe --uid=wow", None, &client).is_none());
    }
}
