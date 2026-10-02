//! GOG: installed games are found through the Windows registry
//! (`HKLM\SOFTWARE\WOW6432Node\GOG.com\Games\<id>`) or macOS app bundles,
//! each carrying a `goggame-<id>.info` JSON file. Owned-but-not-installed
//! games come from GOG Galaxy's local `galaxy-2.0.db`. Launching goes through
//! GOG Galaxy when installed (falls back to the DRM-free executable); installs
//! are handed to Galaxy. Heroic-managed GOG games are included on all OSes.

use super::{heroic, list_dir, read_json, ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Info from a `goggame-<id>.info` file.
#[derive(Debug, PartialEq)]
pub struct GogInfo {
    pub id: String,
    pub name: String,
    pub exe: Option<(PathBuf, Vec<String>, Option<PathBuf>)>,
    pub is_dlc: bool,
}

pub fn parse_info(v: &Value, install_dir: &Path) -> Option<GogInfo> {
    let id = match &v["gameId"] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => return None,
    };
    let root_id = match &v["rootGameId"] {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    };
    let name = v["name"].as_str()?.to_string();
    let tasks = v["playTasks"].as_array().cloned().unwrap_or_default();
    let primary = tasks
        .iter()
        .find(|t| {
            t["isPrimary"] == Value::Bool(true)
                && t["type"].as_str().unwrap_or("FileTask") == "FileTask"
        })
        .or_else(|| {
            tasks
                .iter()
                .find(|t| t["type"].as_str().unwrap_or("FileTask") == "FileTask")
        });
    let exe = primary.and_then(|t| {
        let rel = t["path"].as_str()?;
        let args = t["arguments"]
            .as_str()
            .map(|a| a.split_whitespace().map(String::from).collect())
            .unwrap_or_default();
        let wd = t["workingDir"]
            .as_str()
            .filter(|w| !w.is_empty())
            .map(|w| install_dir.join(w));
        Some((
            install_dir.join(rel),
            args,
            wd.or_else(|| Some(install_dir.to_path_buf())),
        ))
    });
    Some(GogInfo {
        is_dlc: root_id.is_some_and(|r| r != id),
        id,
        name,
        exe,
    })
}

/// How to start a GOG game: GOG Galaxy if available, else the game's exe.
fn launch_target(
    galaxy: Option<&Path>,
    info: &GogInfo,
    install_dir: &Path,
) -> Option<LaunchTarget> {
    if let Some(client) = galaxy {
        let args = if cfg!(target_os = "macos") {
            vec!["--command=runGame".into(), format!("--gameId={}", info.id)]
        } else {
            vec![
                "/command=runGame".into(),
                format!("/gameId={}", info.id),
                format!("/path={}", install_dir.display()),
            ]
        };
        return Some(LaunchTarget::Executable {
            path: client.to_path_buf(),
            args,
            working_dir: client.parent().map(Path::to_path_buf),
        });
    }
    let (path, args, working_dir) = info.exe.clone()?;
    Some(LaunchTarget::Executable {
        path,
        args,
        working_dir,
    })
}

fn galaxy_client() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let dir = super::registry::hklm_string(
            r"SOFTWARE\WOW6432Node\GOG.com\GalaxyClient\paths",
            "client",
        )?;
        let exe = PathBuf::from(dir).join("GalaxyClient.exe");
        return exe.is_file().then_some(exe);
    }
    #[cfg(target_os = "macos")]
    {
        let exe = PathBuf::from("/Applications/GOG Galaxy.app/Contents/MacOS/GOG Galaxy");
        return exe.is_file().then_some(exe);
    }
    #[allow(unreachable_code)]
    None
}

fn galaxy_db() -> Option<PathBuf> {
    #[cfg(windows)]
    return Some(super::program_data().join(r"GOG.com\Galaxy\storage\galaxy-2.0.db"));
    #[cfg(target_os = "macos")]
    return Some(PathBuf::from(
        "/Users/Shared/GOG.com/Galaxy/Storage/galaxy-2.0.db",
    ));
    #[allow(unreachable_code)]
    None
}

/// Install directories of GOG games (registry on Windows, app bundles on macOS).
fn install_dirs(ctx: &ScanContext) -> Vec<PathBuf> {
    #[allow(unused_mut)]
    let mut out = Vec::new();
    #[cfg(windows)]
    for (_, key) in super::registry::hklm_subkeys(r"SOFTWARE\WOW6432Node\GOG.com\Games") {
        if let Some(p) = super::registry::get_string(&key, "path") {
            out.push(PathBuf::from(p));
        }
    }
    #[cfg(target_os = "macos")]
    {
        let mut apps = list_dir(Path::new("/Applications"));
        apps.extend(
            ctx.home_join("Applications")
                .map(|p| list_dir(&p))
                .unwrap_or_default(),
        );
        for app in apps {
            if app.extension().is_some_and(|e| e == "app") {
                out.push(app.join("Contents/Resources"));
            }
        }
    }
    let _ = ctx;
    out
}

pub fn scan_install_dir(dir: &Path, galaxy: Option<&Path>) -> Vec<Game> {
    list_dir(dir)
        .into_iter()
        .filter(|f| {
            f.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("goggame-") && n.ends_with(".info"))
        })
        .filter_map(|f| parse_info(&read_json(&f)?, dir))
        .filter(|info| !info.is_dlc)
        .filter_map(|info| {
            let launch = launch_target(galaxy, &info, dir)?;
            Some(
                Game::new(Platform::Gog, info.id.clone(), info.name.clone(), launch)
                    .with_install_dir(dir),
            )
        })
        .collect()
}

/// Owned GOG games from Galaxy's DB (`gog_<id>` release keys with a title).
pub fn owned_from_galaxy_db(
    conn: &rusqlite::Connection,
) -> Result<Vec<(String, String)>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT lr.releaseKey, gp.value FROM LibraryReleases lr \
         JOIN GamePieces gp ON gp.releaseKey = lr.releaseKey \
         JOIN GamePieceTypes gpt ON gpt.id = gp.gamePieceTypeId \
         WHERE gpt.type = 'title' AND lr.releaseKey LIKE 'gog\\_%' ESCAPE '\\'",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for row in rows.flatten() {
        let (key, value) = row;
        let id = key.trim_start_matches("gog_").to_string();
        let title = serde_json::from_str::<Value>(&value)
            .ok()
            .and_then(|v| v["title"].as_str().map(String::from));
        if let Some(title) = title.filter(|t| !t.trim().is_empty()) {
            if seen.insert(id.clone()) {
                out.push((id, title));
            }
        }
    }
    Ok(out)
}

pub fn install_uri(id: &str) -> String {
    format!("goggalaxy://openGameView/{}", super::encode_component(id))
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let galaxy = galaxy_client();
    let mut games: Vec<Game> = install_dirs(ctx)
        .iter()
        .flat_map(|d| scan_install_dir(d, galaxy.as_deref()))
        .collect();
    games.extend(heroic::scan_gog(ctx));
    super::dedup(&mut games);

    if let Some(db) = galaxy_db() {
        if let Ok(Some(conn)) = super::open_sqlite_readonly(&db) {
            let installed: HashSet<String> = games.iter().map(|g| g.store_id.clone()).collect();
            for (id, title) in owned_from_galaxy_db(&conn).unwrap_or_default() {
                if !installed.contains(&id) {
                    let uri = install_uri(&id);
                    games.push(
                        Game::new(Platform::Gog, id, title, LaunchTarget::uri(uri.clone()))
                            .not_installed(LaunchTarget::uri(uri)),
                    );
                }
            }
        }
    }
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::test_util::write;
    use serde_json::json;

    #[test]
    fn parses_info_and_prefers_galaxy() {
        let dir = tempfile::tempdir().unwrap();
        write(
            &dir.path().join("goggame-1207658924.info"),
            &json!({"gameId": "1207658924", "rootGameId": "1207658924", "name": "Unreal Gold",
                "playTasks": [{"isPrimary": true, "type": "FileTask", "path": "System\\Unreal.exe", "arguments": "-log"}]})
            .to_string(),
        );
        write(
            &dir.path().join("goggame-2.info"),
            &json!({"gameId": "2", "rootGameId": "1207658924", "name": "DLC"}).to_string(),
        );

        let direct = scan_install_dir(dir.path(), None);
        assert_eq!(direct.len(), 1);
        match &direct[0].launch {
            LaunchTarget::Executable { path, args, .. } => {
                assert!(
                    path.ends_with("System\\Unreal.exe")
                        || path.ends_with("System/Unreal.exe")
                        || path.to_string_lossy().contains("Unreal.exe")
                );
                assert_eq!(args, &["-log"]);
            }
            other => panic!("unexpected {other:?}"),
        }

        let galaxy = PathBuf::from("/opt/galaxy/GalaxyClient.exe");
        let via = scan_install_dir(dir.path(), Some(&galaxy));
        match &via[0].launch {
            LaunchTarget::Executable { path, args, .. } => {
                assert_eq!(path, &galaxy);
                assert!(args.iter().any(|a| a.ends_with("gameId=1207658924")));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn reads_owned_games_from_galaxy_db() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE LibraryReleases (releaseKey TEXT);
             CREATE TABLE GamePieceTypes (id INTEGER, type TEXT);
             CREATE TABLE GamePieces (releaseKey TEXT, gamePieceTypeId INTEGER, value TEXT);
             INSERT INTO LibraryReleases VALUES ('gog_1'), ('steam_570'), ('gog_2');
             INSERT INTO GamePieceTypes VALUES (1, 'title'), (2, 'meta');
             INSERT INTO GamePieces VALUES ('gog_1', 1, '{\"title\":\"Witcher\"}'), ('gog_1', 2, '{}'),
               ('steam_570', 1, '{\"title\":\"Dota\"}'), ('gog_2', 1, '{\"title\":\"\"}');",
        )
        .unwrap();
        assert_eq!(
            owned_from_galaxy_db(&conn).unwrap(),
            [("1".to_string(), "Witcher".to_string())]
        );
    }
}
