//! Lutris (Linux): games live in `~/.local/share/lutris/pga.db` (table
//! `games`). Launching and installing go through Lutris
//! (`lutris:rungameid/<id>`, `lutris:install/<slug>`).

use super::{ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use std::path::PathBuf;

pub fn read_db(conn: &rusqlite::Connection) -> Result<Vec<Game>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT id, name, slug, directory, installed, installer_slug FROM games")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, Option<String>>(5)?,
        ))
    })?;
    let mut games = Vec::new();
    for (id, name, slug, dir, installed, installer_slug) in rows.flatten() {
        let Some(name) = name.filter(|n| !n.trim().is_empty()) else {
            continue;
        };
        let mut g = Game::new(
            Platform::Lutris,
            id.to_string(),
            name,
            LaunchTarget::uri(format!("lutris:rungameid/{id}")),
        );
        g.install_dir = dir.filter(|d| !d.is_empty()).map(PathBuf::from);
        if installed != Some(1) {
            let Some(slug) = installer_slug.or(slug).filter(|s| !s.is_empty()) else {
                continue;
            };
            g = g.not_installed(LaunchTarget::uri(format!(
                "lutris:install/{}",
                super::encode_component(&slug)
            )));
        }
        games.push(g);
    }
    Ok(games)
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let mut games = Vec::new();
    if cfg!(all(unix, not(target_os = "macos"))) {
        let candidates = [
            dirs::data_dir().map(|d| d.join("lutris/pga.db")),
            ctx.home_join(".var/app/net.lutris.Lutris/data/lutris/pga.db"),
        ];
        for path in candidates.into_iter().flatten() {
            if let Some(conn) = super::open_sqlite_readonly(&path)? {
                games.extend(read_db(&conn).map_err(|e| format!("{}: {e}", path.display()))?);
            }
        }
    }
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_installed_and_uninstalled_games() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE games (id INTEGER, name TEXT, slug TEXT, directory TEXT, installed INTEGER, installer_slug TEXT);
             INSERT INTO games VALUES (1, 'Doom', 'doom', '/games/doom', 1, NULL),
                                      (2, 'Quake', 'quake', NULL, 0, 'quake-gog'),
                                      (3, '', 'x', NULL, 1, NULL);",
        )
        .unwrap();
        let games = read_db(&conn).unwrap();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].launch, LaunchTarget::uri("lutris:rungameid/1"));
        assert!(games[0].installed);
        assert!(!games[1].installed);
        assert_eq!(
            games[1].install,
            Some(LaunchTarget::uri("lutris:install/quake%2Dgog"))
        );
    }
}
