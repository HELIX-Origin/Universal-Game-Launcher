//! Amazon Games: the Amazon Games app keeps installs in
//! `%LOCALAPPDATA%\Amazon Games\Data\Games\Sql\GameInstallInfo.sqlite`
//! (table `DbSet`). Games launch through the app (`amazon-games://play/<id>`).
//! Heroic-managed (nile) Amazon games are included on all OSes.

use super::{heroic, ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use std::path::PathBuf;

pub fn read_db(conn: &rusqlite::Connection) -> Result<Vec<Game>, rusqlite::Error> {
    let mut stmt =
        conn.prepare("SELECT Id, ProductTitle, InstallDirectory FROM DbSet WHERE Installed = 1")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, Option<String>>(2)?,
        ))
    })?;
    let mut games = Vec::new();
    for (id, title, dir) in rows.flatten() {
        let Some(title) = title.filter(|t| !t.trim().is_empty()) else {
            continue;
        };
        let uri = format!("amazon-games://play/{}", super::encode_component(&id));
        let mut g = Game::new(Platform::Amazon, id, title, LaunchTarget::uri(uri));
        g.install_dir = dir.filter(|d| !d.is_empty()).map(PathBuf::from);
        games.push(g);
    }
    Ok(games)
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let mut games = Vec::new();
    if cfg!(windows) {
        if let Some(db) = dirs::data_local_dir()
            .map(|d| d.join(r"Amazon Games\Data\Games\Sql\GameInstallInfo.sqlite"))
        {
            if let Some(conn) = super::open_sqlite_readonly(&db)? {
                games.extend(read_db(&conn).map_err(|e| e.to_string())?);
            }
        }
    }
    games.extend(heroic::scan_amazon(ctx));
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_installed_rows() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE DbSet (Id TEXT, ProductTitle TEXT, InstallDirectory TEXT, Installed INTEGER);
             INSERT INTO DbSet VALUES ('amzn1.adg.product.1', 'Fallout 3', 'C:\\Amazon\\Fallout 3', 1),
                                      ('amzn1.adg.product.2', 'Not Installed', NULL, 0);",
        )
        .unwrap();
        let games = read_db(&conn).unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].title, "Fallout 3");
        assert!(
            matches!(&games[0].launch, LaunchTarget::Uri { uri } if uri.starts_with("amazon-games://play/amzn1"))
        );
    }
}
