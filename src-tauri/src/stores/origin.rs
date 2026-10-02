//! Origin (legacy EA client, Windows/macOS): `.mfst` manifests under
//! `%ProgramData%\Origin\LocalContent\<Game>\` hold a URL-encoded query
//! string with `id` and `dipInstallPath`. Games launch through the client
//! (`origin2://game/launch?offerIds=<id>`).

use super::{list_dir, ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use std::path::{Path, PathBuf};

fn decode(s: &str) -> String {
    percent_encoding::percent_decode_str(&s.replace('+', " "))
        .decode_utf8_lossy()
        .into_owned()
}

/// Parse a `.mfst` body. `folder_title` (the LocalContent sub-folder name)
/// is used as the game title.
pub fn parse_mfst(body: &str, folder_title: &str) -> Option<Game> {
    let query = body.trim().trim_start_matches('?');
    let mut id = None;
    let mut install: Option<String> = None;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        match k {
            "id" => id = Some(decode(v)),
            k if k.eq_ignore_ascii_case("dipinstallpath") => {
                let v = decode(v);
                if install.as_ref().is_none_or(|cur| v.len() > cur.len()) {
                    install = Some(v);
                }
            }
            _ => {}
        }
    }
    let id = id.filter(|i| !i.is_empty() && !i.ends_with("@steam"))?;
    let title = folder_title.trim();
    if title.is_empty() {
        return None;
    }
    let uri = format!(
        "origin2://game/launch?offerIds={}",
        super::encode_component(&id)
    );
    let mut g = Game::new(Platform::Origin, id, title, LaunchTarget::uri(uri));
    g.install_dir = install.filter(|p| !p.is_empty()).map(PathBuf::from);
    Some(g)
}

fn local_content() -> Option<PathBuf> {
    #[cfg(windows)]
    return Some(super::program_data().join(r"Origin\LocalContent"));
    #[cfg(target_os = "macos")]
    return Some(PathBuf::from(
        "/Library/Application Support/Origin/LocalContent",
    ));
    #[allow(unreachable_code)]
    None
}

pub fn scan_local_content(root: &Path) -> Vec<Game> {
    let mut games = Vec::new();
    for folder in list_dir(root).into_iter().filter(|p| p.is_dir()) {
        let title = folder
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        for f in list_dir(&folder) {
            if f.extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("mfst"))
            {
                games.extend(super::read_text(&f).and_then(|b| parse_mfst(&b, &title)));
            }
        }
    }
    games
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let _ = ctx;
    let mut games = local_content()
        .map(|r| scan_local_content(&r))
        .unwrap_or_default();
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::test_util::write;

    #[test]
    fn parses_manifests() {
        let root = tempfile::tempdir().unwrap();
        write(
            &root.path().join("Mass Effect 2/OFB-EAST50170.mfst"),
            "?currentstate=kReadyToStart&dipinstallpath=C%3a%5cGames%5cME2&dipInstallPath=C%3a%5cGames%5cMass%20Effect%202%5c&id=OFB-EAST%3a50170",
        );
        write(&root.path().join("Steam Game/x.mfst"), "?id=OFB-1%40steam");
        let games = scan_local_content(root.path());
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "origin:OFB-EAST:50170");
        assert_eq!(games[0].title, "Mass Effect 2");
        assert_eq!(
            games[0].install_dir,
            Some(PathBuf::from(r"C:\Games\Mass Effect 2\"))
        );
        assert_eq!(
            games[0].launch,
            LaunchTarget::uri("origin2://game/launch?offerIds=OFB%2DEAST%3A50170")
        );
    }
}
