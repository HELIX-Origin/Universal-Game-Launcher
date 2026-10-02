//! itch.io app: installs ("caves") live in the butler SQLite database
//! (`<itch config>/db/butler.db`). Games launch through the itch app
//! (`itch://caves/<caveId>/launch`).

use super::{ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use serde_json::Value;
use std::path::PathBuf;

fn db_paths(ctx: &ScanContext) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = dirs::config_dir()
        .map(|d| d.join("itch").join("db").join("butler.db"))
        .into_iter()
        .collect();
    #[cfg(all(unix, not(target_os = "macos")))]
    out.extend(ctx.home_join(".var/app/io.itch.itch/config/itch/db/butler.db"));
    let _ = ctx;
    out
}

pub fn read_caves(conn: &rusqlite::Connection) -> Result<Vec<Game>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT c.id, g.title, g.cover_url, c.verdict FROM caves c JOIN games g ON c.game_id = g.id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, Option<String>>(3)?,
        ))
    })?;
    let mut games = Vec::new();
    for (cave_id, title, cover, verdict) in rows.flatten() {
        let Some(title) = title.filter(|t| !t.trim().is_empty()) else {
            continue;
        };
        let uri = format!("itch://caves/{}/launch", super::encode_component(&cave_id));
        let mut game = Game::new(Platform::Itch, cave_id, title, LaunchTarget::uri(uri));
        game.cover_url = cover.filter(|c| c.starts_with("https://"));
        game.install_dir = verdict
            .and_then(|v| serde_json::from_str::<Value>(&v).ok())
            .and_then(|v| v["basePath"].as_str().map(PathBuf::from));
        games.push(game);
    }
    Ok(games)
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let mut games = Vec::new();
    for path in db_paths(ctx) {
        if let Some(conn) = super::open_sqlite_readonly(&path)? {
            games.extend(read_caves(&conn).map_err(|e| format!("{}: {e}", path.display()))?);
        }
    }
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_caves_joined_with_games() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE games (id INTEGER, title TEXT, cover_url TEXT);
             CREATE TABLE caves (id TEXT, game_id INTEGER, verdict TEXT);
             INSERT INTO games VALUES (1, 'Celeste Classic', 'https://img.itch.zone/a.png'), (2, NULL, NULL);
             INSERT INTO caves VALUES ('cave-uuid', 1, '{\"basePath\":\"/games/celeste\",\"candidates\":[{\"path\":\"celeste\"}]}'),
                                      ('other', 2, NULL);",
        )
        .unwrap();
        let games = read_caves(&conn).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "itch:cave-uuid");
        assert_eq!(
            games[0].launch,
            LaunchTarget::uri("itch://caves/cave%2Duuid/launch")
        );
        assert_eq!(games[0].install_dir, Some(PathBuf::from("/games/celeste")));
        assert_eq!(
            games[0].cover_url.as_deref(),
            Some("https://img.itch.zone/a.png")
        );
    }
}
