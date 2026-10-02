//! EA app (Windows). The EA app's own install list is encrypted, so — like
//! Steam ROM Manager — we read each game's `__Installer\installerdata.xml`
//! from EA install folders (EA Games dirs + "Electronic Arts" uninstall
//! entries). Games launch through the EA app (`link2ea://launchgame/<id>`).

use super::{list_dir, ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use std::path::{Path, PathBuf};

/// Decode UTF-8 or UTF-16 (LE/BE, BOM-detected) XML bytes.
pub fn decode_xml(bytes: &[u8]) -> String {
    match bytes {
        [0xFF, 0xFE, rest @ ..] => String::from_utf16_lossy(
            &rest
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect::<Vec<_>>(),
        ),
        [0xFE, 0xFF, rest @ ..] => String::from_utf16_lossy(
            &rest
                .chunks_exact(2)
                .map(|c| u16::from_be_bytes([c[0], c[1]]))
                .collect::<Vec<_>>(),
        ),
        _ => String::from_utf8_lossy(bytes)
            .trim_start_matches('\u{feff}')
            .to_string(),
    }
}

/// `(content id, title)` from an `installerdata.xml` document.
pub fn parse_installer_data(xml: &str) -> Option<(String, String)> {
    // EA manifests sometimes declare encoding="utf-16" even after we decoded them.
    let xml = xml
        .replacen("encoding=\"utf-16\"", "", 1)
        .replacen("encoding=\"UTF-16\"", "", 1);
    let doc = roxmltree::Document::parse(&xml).ok()?;
    let text = |name: &str| {
        doc.descendants()
            .filter(|n| n.has_tag_name(name))
            .find_map(|n| {
                n.text()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(String::from)
            })
    };
    let id = text("contentID")?;
    let title = doc
        .descendants()
        .filter(|n| n.has_tag_name("gameTitle"))
        .find(|n| {
            n.attribute("locale")
                .is_some_and(|l| l.eq_ignore_ascii_case("en_US"))
        })
        .and_then(|n| n.text())
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .or_else(|| text("gameTitle"))
        .or_else(|| text("title"))?;
    Some((id, title))
}

pub fn scan_game_dir(dir: &Path) -> Option<Game> {
    let bytes = std::fs::read(dir.join("__Installer").join("installerdata.xml")).ok()?;
    let (id, title) = parse_installer_data(&decode_xml(&bytes))?;
    let uri = format!("link2ea://launchgame/{}", super::encode_component(&id));
    Some(Game::new(Platform::Ea, id, title, LaunchTarget::uri(uri)).with_install_dir(dir))
}

fn candidate_dirs() -> Vec<PathBuf> {
    #[allow(unused_mut)]
    let mut dirs: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        for root in [
            r"C:\Program Files\EA Games",
            r"C:\Program Files (x86)\EA Games",
            r"C:\Program Files (x86)\Origin Games",
        ] {
            dirs.extend(list_dir(Path::new(root)));
        }
        for (_, key) in super::registry::uninstall_entries() {
            let publisher = super::registry::get_string(&key, "Publisher").unwrap_or_default();
            if publisher.contains("Electronic Arts") {
                if let Some(loc) = super::registry::get_string(&key, "InstallLocation") {
                    dirs.push(PathBuf::from(loc));
                }
            }
        }
    }
    dirs
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let _ = (ctx, list_dir as fn(&Path) -> Vec<PathBuf>);
    let mut games: Vec<Game> = candidate_dirs()
        .iter()
        .filter_map(|d| scan_game_dir(d))
        .collect();
    super::dedup(&mut games);
    Ok(games)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stores::test_util::write;

    const XML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<DiPManifest version="4.0">
  <contentIDs><contentID>1026023</contentID><contentID>1026024</contentID></contentIDs>
  <gameTitles><gameTitle locale="de_DE">Titel</gameTitle><gameTitle locale="en_US">Battlefield 1</gameTitle></gameTitles>
</DiPManifest>"#;

    #[test]
    fn parses_installer_data() {
        assert_eq!(
            parse_installer_data(XML),
            Some(("1026023".into(), "Battlefield 1".into()))
        );
        assert_eq!(parse_installer_data("<x/>"), None);
    }

    #[test]
    fn decodes_utf16_manifests() {
        let utf16: Vec<u8> = [0xFF, 0xFE]
            .into_iter()
            .chain(
                XML.replace("utf-8", "utf-16")
                    .encode_utf16()
                    .flat_map(u16::to_le_bytes),
            )
            .collect();
        assert_eq!(
            parse_installer_data(&decode_xml(&utf16)).unwrap().1,
            "Battlefield 1"
        );
    }

    #[test]
    fn scans_game_folder() {
        let dir = tempfile::tempdir().unwrap();
        write(&dir.path().join("__Installer/installerdata.xml"), XML);
        let g = scan_game_dir(dir.path()).unwrap();
        assert_eq!(g.id, "ea:1026023");
        assert_eq!(g.launch, LaunchTarget::uri("link2ea://launchgame/1026023"));
    }
}
