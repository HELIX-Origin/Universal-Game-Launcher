//! Optional metadata enrichment from third-party providers.
//!
//! * Strictly opt-in: nothing is fetched unless the user enables a provider
//!   in Settings → Metadata and (where required) supplies their **own** key.
//! * Keys are stored in a separate `secrets.json` (owner-only permissions on
//!   Unix) and are never sent back to the webview – only "is set" flags.
//! * Requests go only to the hard-coded provider hosts below, from Rust (the
//!   provider APIs do not allow browser CORS).
//! * Response parsing is pure (`parse_*` functions) so it is unit-tested
//!   without network access.

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::{Game, Platform};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub const SECRETS_FILE: &str = "secrets.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum MetadataProvider {
    /// Artwork (grids / heroes). Requires a SteamGridDB API key.
    SteamGridDb,
    /// Steam Store search + app details. No key required.
    SteamStore,
    /// IGDB (Twitch developer client id + secret).
    Igdb,
    /// The Visual Novel Database. Token optional.
    Vndb,
    /// Legacy persisted value; discontinued and never queried.
    Rawg,
}

impl MetadataProvider {
    pub const ALL: [MetadataProvider; 4] = [
        MetadataProvider::SteamGridDb,
        MetadataProvider::SteamStore,
        MetadataProvider::Igdb,
        MetadataProvider::Vndb,
    ];
}

/// User-provided API keys. Empty strings are treated as "not set".
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApiKeys {
    pub steamgriddb: Option<String>,
    pub igdb_client_id: Option<String>,
    pub igdb_client_secret: Option<String>,
    pub vndb: Option<String>,
    pub rawg: Option<String>,
}

/// What the UI is allowed to know about the keys.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyStatus {
    pub steamgriddb: bool,
    pub igdb_client_id: bool,
    pub igdb_client_secret: bool,
    pub vndb: bool,
    pub rawg: bool,
}

fn set(v: &Option<String>) -> bool {
    v.as_deref().is_some_and(|s| !s.trim().is_empty())
}

fn key(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

impl ApiKeys {
    pub fn status(&self) -> ApiKeyStatus {
        ApiKeyStatus {
            steamgriddb: set(&self.steamgriddb),
            igdb_client_id: set(&self.igdb_client_id),
            igdb_client_secret: set(&self.igdb_client_secret),
            vndb: set(&self.vndb),
            rawg: set(&self.rawg),
        }
    }

    /// Apply an update from the UI: `None` keeps the stored key, `Some("")`
    /// clears it, anything else replaces it.
    pub fn merge(&mut self, update: ApiKeys) {
        fn apply(slot: &mut Option<String>, new: Option<String>) {
            if let Some(v) = new {
                let v = v.trim().to_string();
                *slot = (!v.is_empty()).then_some(v);
            }
        }
        apply(&mut self.steamgriddb, update.steamgriddb);
        apply(&mut self.igdb_client_id, update.igdb_client_id);
        apply(&mut self.igdb_client_secret, update.igdb_client_secret);
        apply(&mut self.vndb, update.vndb);
        apply(&mut self.rawg, update.rawg);
    }

    pub fn load(path: &Path) -> ApiKeys {
        fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        let storage = |e: std::io::Error| AppError::with(ErrorCode::Storage, e.to_string());
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(storage)?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::with(ErrorCode::Internal, e.to_string()))?;
        let tmp = path.with_extension("json.tmp");
        write_private(&tmp, json.as_bytes()).map_err(storage)?;
        fs::rename(&tmp, path).map_err(storage)
    }
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(bytes)
}

#[cfg(not(unix))]
fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    // %APPDATA% is already per-user on Windows.
    fs::write(path, bytes)
}

/// Enriched metadata for one game (merged across providers).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameMetadata {
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    /// 0–100.
    pub rating: Option<f32>,
    pub cover_url: Option<String>,
    pub hero_url: Option<String>,
    pub sources: Vec<MetadataProvider>,
    pub fetched_at: i64,
}

impl GameMetadata {
    pub fn is_empty(&self) -> bool {
        self.description.is_none()
            && self.developer.is_none()
            && self.publisher.is_none()
            && self.release_date.is_none()
            && self.genres.is_empty()
            && self.rating.is_none()
            && self.cover_url.is_none()
            && self.hero_url.is_none()
    }

    /// Fill fields that are still empty from `other` (earlier providers win).
    pub fn fill_from(&mut self, other: GameMetadata, source: MetadataProvider) {
        if other.is_empty() {
            return;
        }
        fn fill<T>(slot: &mut Option<T>, v: Option<T>) {
            if slot.is_none() {
                *slot = v;
            }
        }
        fill(&mut self.description, other.description);
        fill(&mut self.developer, other.developer);
        fill(&mut self.publisher, other.publisher);
        fill(&mut self.release_date, other.release_date);
        fill(&mut self.rating, other.rating);
        fill(&mut self.cover_url, other.cover_url);
        fill(&mut self.hero_url, other.hero_url);
        if self.genres.is_empty() {
            self.genres = other.genres;
        }
        if !self.sources.contains(&source) {
            self.sources.push(source);
        }
    }
}

// ---------------------------------------------------------------- helpers

fn s(v: &Value) -> Option<String> {
    v.as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
}

fn https(v: &Value) -> Option<String> {
    s(v).filter(|u| u.starts_with("https://"))
}

fn names(v: &Value, field: &str) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(|x| s(&x[field])).collect())
        .unwrap_or_default()
}

/// Remove HTML tags / BBCode and collapse whitespace. Output is rendered as
/// plain text by the UI (never as HTML).
pub fn plain_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut depth_angle = false;
    let mut depth_square = false;
    for c in input.chars() {
        match c {
            '<' => depth_angle = true,
            '>' if depth_angle => {
                depth_angle = false;
                out.push(' ');
            }
            '[' => depth_square = true,
            ']' if depth_square => depth_square = false,
            _ if depth_angle || depth_square => {}
            _ => out.push(c),
        }
    }
    let out = out
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ");
    let mut text = String::new();
    for para in out.split("\n\n") {
        let p = para.split_whitespace().collect::<Vec<_>>().join(" ");
        if !p.is_empty() {
            if !text.is_empty() {
                text.push_str("\n\n");
            }
            text.push_str(&p);
        }
    }
    text
}

fn normalize(title: &str) -> String {
    title
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Pick the best search hit: exact (normalized) title match, else the first.
fn best_match<'a>(items: &'a [Value], title: &str, name_field: &str) -> Option<&'a Value> {
    let wanted = normalize(title);
    items
        .iter()
        .find(|i| {
            i[name_field]
                .as_str()
                .is_some_and(|n| normalize(n) == wanted)
        })
        .or_else(|| items.first())
}

// ---------------------------------------------------------------- parsers

pub fn parse_steam_search(v: &Value, title: &str) -> Option<u64> {
    best_match(v["items"].as_array()?, title, "name")?["id"].as_u64()
}

pub fn parse_steam_details(v: &Value, appid: u64) -> GameMetadata {
    let d = &v[appid.to_string()]["data"];
    if v[appid.to_string()]["success"] != json!(true) {
        return GameMetadata::default();
    }
    GameMetadata {
        description: s(&d["short_description"]).map(|t| plain_text(&t)),
        developer: names_str(&d["developers"]).into_iter().next(),
        publisher: names_str(&d["publishers"]).into_iter().next(),
        release_date: s(&d["release_date"]["date"]),
        genres: names(&d["genres"], "description"),
        rating: d["metacritic"]["score"].as_f64().map(|r| r as f32),
        cover_url: Some(crate::stores::steam::cover_url(&appid.to_string())),
        hero_url: Some(crate::stores::steam::hero_url(&appid.to_string())),
        ..Default::default()
    }
}

fn names_str(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(s).collect())
        .unwrap_or_default()
}

pub fn parse_sgdb_search(v: &Value, title: &str) -> Option<u64> {
    best_match(v["data"].as_array()?, title, "name")?["id"].as_u64()
}

pub fn parse_sgdb_first_image(v: &Value) -> Option<String> {
    v["data"].as_array()?.iter().find_map(|i| https(&i["url"]))
}

pub fn igdb_image(image_id: &str, size: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/{size}/{image_id}.jpg")
}

pub fn parse_igdb_games(v: &Value, title: &str) -> GameMetadata {
    let Some(items) = v.as_array() else {
        return GameMetadata::default();
    };
    let Some(g) = best_match(items, title, "name") else {
        return GameMetadata::default();
    };
    let companies = g["involved_companies"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let company = |role: &str| {
        companies
            .iter()
            .find(|c| c[role] == json!(true))
            .and_then(|c| s(&c["company"]["name"]))
    };
    let release_date = g["first_release_date"].as_i64().map(unix_to_ymd);
    GameMetadata {
        description: s(&g["summary"]),
        developer: company("developer"),
        publisher: company("publisher"),
        release_date,
        genres: names(&g["genres"], "name"),
        rating: g["total_rating"].as_f64().map(|r| r as f32),
        cover_url: s(&g["cover"]["image_id"]).map(|id| igdb_image(&id, "t_cover_big")),
        hero_url: g["artworks"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|a| s(&a["image_id"]))
            .map(|id| igdb_image(&id, "t_1080p")),
        ..Default::default()
    }
}

pub fn parse_vndb(v: &Value, title: &str) -> GameMetadata {
    let Some(items) = v["results"].as_array() else {
        return GameMetadata::default();
    };
    let Some(vn) = best_match(items, title, "title") else {
        return GameMetadata::default();
    };
    GameMetadata {
        description: s(&vn["description"]).map(|d| plain_text(&d)),
        developer: names(&vn["developers"], "name").into_iter().next(),
        release_date: s(&vn["released"]),
        genres: names(&vn["tags"], "name").into_iter().take(5).collect(),
        // VNDB ratings are 10–100.
        rating: vn["rating"].as_f64().map(|r| r as f32),
        cover_url: https(&vn["image"]["url"]),
        ..Default::default()
    }
}

/// Days-since-epoch → `YYYY-MM-DD` (civil-from-days, Howard Hinnant).
fn unix_to_ymd(secs: i64) -> String {
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Steam Store `l=` parameter for a UI locale (falls back to English).
pub fn steam_language(locale: &str) -> &'static str {
    let lower = locale.to_ascii_lowercase();
    match lower.as_str() {
        "pt-br" => return "brazilian",
        "es-419" | "es-mx" => return "latam",
        "zh-tw" | "zh-hant" | "zh-hk" => return "tchinese",
        "zh-cn" | "zh-hans" | "zh" => return "schinese",
        _ => {}
    }
    match lower.split('-').next().unwrap_or("") {
        "ar" => "arabic",
        "bg" => "bulgarian",
        "cs" => "czech",
        "da" => "danish",
        "de" => "german",
        "el" => "greek",
        "es" => "spanish",
        "fi" => "finnish",
        "fr" => "french",
        "hu" => "hungarian",
        "id" => "indonesian",
        "it" => "italian",
        "ja" => "japanese",
        "ko" => "koreana",
        "nb" | "no" | "nn" => "norwegian",
        "nl" => "dutch",
        "pl" => "polish",
        "pt" => "portuguese",
        "ro" => "romanian",
        "ru" => "russian",
        "sv" => "swedish",
        "th" => "thai",
        "tr" => "turkish",
        "uk" => "ukrainian",
        "vi" => "vietnamese",
        _ => "english",
    }
}

// ---------------------------------------------------------------- network

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .user_agent(concat!("UniversalGameLauncher/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

fn req_err(e: ureq::Error) -> AppError {
    AppError::with(ErrorCode::MetadataRequestFailed, e.to_string())
}

fn read_json(mut resp: ureq::http::Response<ureq::Body>) -> AppResult<Value> {
    resp.body_mut().read_json::<Value>().map_err(req_err)
}

fn missing_key(provider: &str) -> AppError {
    AppError::with(ErrorCode::MetadataMissingKey, provider)
}

fn fetch_steam_store(a: &ureq::Agent, game: &Game, locale: &str) -> AppResult<GameMetadata> {
    let lang = steam_language(locale);
    let appid = match (game.platform, game.store_id.parse::<u64>()) {
        (Platform::Steam, Ok(id)) => Some(id),
        _ => {
            let r = a
                .get("https://store.steampowered.com/api/storesearch/")
                .query("term", &game.title)
                .query("l", lang)
                .query("cc", "US")
                .call()
                .map_err(req_err)?;
            parse_steam_search(&read_json(r)?, &game.title)
        }
    };
    let Some(appid) = appid else {
        return Ok(GameMetadata::default());
    };
    let r = a
        .get("https://store.steampowered.com/api/appdetails")
        .query("appids", appid.to_string())
        .query("l", lang)
        .call()
        .map_err(req_err)?;
    Ok(parse_steam_details(&read_json(r)?, appid))
}

fn fetch_sgdb(a: &ureq::Agent, game: &Game, keys: &ApiKeys) -> AppResult<GameMetadata> {
    let k = key(&keys.steamgriddb).ok_or_else(|| missing_key("steamGridDb"))?;
    let auth = ["Bearer", k].join(" ");
    let base = "https://www.steamgriddb.com/api/v2";
    let target = if game.platform == Platform::Steam && game.store_id.parse::<u64>().is_ok() {
        format!("steam/{}", game.store_id)
    } else {
        let url = format!(
            "{base}/search/autocomplete/{}",
            crate::stores::encode_component(&game.title)
        );
        let r = a
            .get(&url)
            .header("Authorization", &auth)
            .call()
            .map_err(req_err)?;
        match parse_sgdb_search(&read_json(r)?, &game.title) {
            Some(id) => format!("game/{id}"),
            None => return Ok(GameMetadata::default()),
        }
    };
    let get = |kind: &str, query: &[(&str, &str)]| -> AppResult<Option<String>> {
        let mut req = a
            .get(format!("{base}/{kind}/{target}"))
            .header("Authorization", &auth);
        for (k, v) in query {
            req = req.query(*k, *v);
        }
        match req.call() {
            Ok(r) => Ok(parse_sgdb_first_image(&read_json(r)?)),
            Err(ureq::Error::StatusCode(404)) => Ok(None),
            Err(e) => Err(req_err(e)),
        }
    };
    Ok(GameMetadata {
        cover_url: get("grids", &[("dimensions", "600x900")])?,
        hero_url: get("heroes", &[])?,
        ..Default::default()
    })
}

fn fetch_igdb(a: &ureq::Agent, game: &Game, keys: &ApiKeys) -> AppResult<GameMetadata> {
    let id = key(&keys.igdb_client_id).ok_or_else(|| missing_key("igdb"))?;
    let secret = key(&keys.igdb_client_secret).ok_or_else(|| missing_key("igdb"))?;
    // App-access token (client-credentials grant) for the user's own Twitch app.
    let r = a
        .post("https://id.twitch.tv/oauth2/token")
        .query("client_id", id)
        .query("client_secret", secret)
        .query("grant_type", "client_credentials")
        .send_empty()
        .map_err(req_err)?;
    let token = s(&read_json(r)?["access_token"])
        .ok_or_else(|| AppError::with(ErrorCode::MetadataRequestFailed, "igdb token"))?;
    let escaped = game.title.replace('\\', "\\\\").replace('"', "\\\"");
    let body = format!(
        "search \"{escaped}\"; fields name,summary,first_release_date,total_rating,genres.name,cover.image_id,artworks.image_id,involved_companies.developer,involved_companies.publisher,involved_companies.company.name; limit 10;"
    );
    let r = a
        .post("https://api.igdb.com/v4/games")
        .header("Client-ID", id)
        .header("Authorization", &["Bearer", token.as_str()].join(" "))
        .send(body)
        .map_err(req_err)?;
    Ok(parse_igdb_games(&read_json(r)?, &game.title))
}

fn fetch_vndb(a: &ureq::Agent, game: &Game, keys: &ApiKeys) -> AppResult<GameMetadata> {
    let mut req = a.post("https://api.vndb.org/kana/vn");
    if let Some(k) = key(&keys.vndb) {
        req = req.header("Authorization", &format!("Token {k}"));
    }
    let body = json!({
        "filters": ["search", "=", game.title],
        "fields": "title, description, released, rating, image.url, developers.name, tags.name",
        "results": 10
    });
    let r = req.send_json(&body).map_err(req_err)?;
    Ok(parse_vndb(&read_json(r)?, &game.title))
}

/// Query the enabled providers in order and merge the results.
pub fn fetch(
    game: &Game,
    providers: &[MetadataProvider],
    keys: &ApiKeys,
    locale: &str,
    now: i64,
) -> AppResult<GameMetadata> {
    if providers.is_empty() {
        return Err(ErrorCode::MetadataNoProviders.into());
    }
    let a = agent();
    let mut merged = GameMetadata::default();
    let mut last_err = None;
    for &p in providers {
        let result = match p {
            MetadataProvider::SteamStore => fetch_steam_store(&a, game, locale),
            MetadataProvider::SteamGridDb => fetch_sgdb(&a, game, keys),
            MetadataProvider::Igdb => fetch_igdb(&a, game, keys),
            MetadataProvider::Vndb => fetch_vndb(&a, game, keys),
            MetadataProvider::Rawg => Err(AppError::with(
                ErrorCode::MetadataRequestFailed,
                "RAWG is discontinued; use IGDB or another enabled provider",
            )),
        };
        match result {
            Ok(m) => merged.fill_from(m, p),
            Err(e) => {
                log::warn!("metadata provider {p:?} failed for {}: {e}", game.id);
                last_err = Some(e);
            }
        }
    }
    if merged.is_empty() {
        return Err(last_err
            .unwrap_or_else(|| AppError::with(ErrorCode::MetadataNotFound, game.title.clone())));
    }
    merged.fetched_at = now;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_merge_and_status_never_leak_values() {
        let mut k = ApiKeys::default();
        k.merge(ApiKeys {
            rawg: Some("  abc ".into()),
            steamgriddb: Some("x".into()),
            ..Default::default()
        });
        assert_eq!(k.rawg.as_deref(), Some("abc"));
        k.merge(ApiKeys {
            steamgriddb: Some("".into()),
            ..Default::default()
        });
        assert_eq!(k.steamgriddb, None);
        assert_eq!(k.rawg.as_deref(), Some("abc"), "None keeps existing keys");
        let status = serde_json::to_string(&k.status()).unwrap();
        assert!(!status.contains("abc"));
        assert!(k.status().rawg && !k.status().steamgriddb);
    }

    #[test]
    fn keys_roundtrip_with_private_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SECRETS_FILE);
        let k = ApiKeys {
            igdb_client_id: Some("id".into()),
            ..Default::default()
        };
        k.save(&path).unwrap();
        assert_eq!(ApiKeys::load(&path), k);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn steam_parsers() {
        let search = json!({"total": 2, "items": [{"id": 1, "name": "Portal 2 Soundtrack"}, {"id": 620, "name": "Portal 2"}]});
        assert_eq!(parse_steam_search(&search, "portal 2"), Some(620));
        assert_eq!(parse_steam_search(&json!({"items": []}), "x"), None);

        let details = json!({"620": {"success": true, "data": {
            "short_description": "The <b>sequel</b> &amp; more",
            "developers": ["Valve"], "publishers": ["Valve"],
            "release_date": {"date": "18 Apr, 2011"},
            "genres": [{"description": "Action"}, {"description": "Adventure"}],
            "metacritic": {"score": 95}
        }}});
        let m = parse_steam_details(&details, 620);
        assert_eq!(m.description.as_deref(), Some("The sequel & more"));
        assert_eq!(m.developer.as_deref(), Some("Valve"));
        assert_eq!(m.genres, ["Action", "Adventure"]);
        assert_eq!(m.rating, Some(95.0));
        assert!(parse_steam_details(&json!({"620": {"success": false}}), 620).is_empty());
    }

    #[test]
    fn sgdb_parsers() {
        let search = json!({"success": true, "data": [{"id": 5, "name": "Hades"}, {"id": 6, "name": "Hades II"}]});
        assert_eq!(parse_sgdb_search(&search, "Hades II"), Some(6));
        let grids = json!({"data": [{"url": "http://insecure"}, {"url": "https://cdn2.steamgriddb.com/grid/a.png"}]});
        assert_eq!(
            parse_sgdb_first_image(&grids).as_deref(),
            Some("https://cdn2.steamgriddb.com/grid/a.png")
        );
    }

    #[test]
    fn igdb_parser() {
        let v = json!([{
            "name": "Celeste", "summary": "Climb.", "first_release_date": 1516838400, "total_rating": 91.5,
            "genres": [{"name": "Platform"}], "cover": {"image_id": "co1"}, "artworks": [{"image_id": "ar1"}],
            "involved_companies": [
                {"developer": false, "publisher": true, "company": {"name": "Matt Makes Games"}},
                {"developer": true, "publisher": false, "company": {"name": "Extremely OK Games"}}
            ]
        }]);
        let m = parse_igdb_games(&v, "Celeste");
        assert_eq!(m.release_date.as_deref(), Some("2018-01-25"));
        assert_eq!(m.developer.as_deref(), Some("Extremely OK Games"));
        assert_eq!(m.publisher.as_deref(), Some("Matt Makes Games"));
        assert_eq!(
            m.cover_url.as_deref(),
            Some("https://images.igdb.com/igdb/image/upload/t_cover_big/co1.jpg")
        );
        assert_eq!(
            m.hero_url.as_deref(),
            Some("https://images.igdb.com/igdb/image/upload/t_1080p/ar1.jpg")
        );
        assert!(parse_igdb_games(&json!([]), "x").is_empty());
    }

    #[test]
    fn vndb_parser() {
        let vn = json!({"results": [{"title": "Steins;Gate", "description": "[b]Time[/b] travel\n\nPart two", "released": "2009-10-15",
            "rating": 89.2, "image": {"url": "https://t.vndb.org/cv/1.jpg"}, "developers": [{"name": "5pb."}], "tags": [{"name": "Sci-fi"}]}]});
        let m = parse_vndb(&vn, "Steins;Gate");
        assert_eq!(m.description.as_deref(), Some("Time travel\n\nPart two"));
        assert_eq!(m.developer.as_deref(), Some("5pb."));
    }

    #[test]
    fn merge_prefers_earlier_providers() {
        let mut m = GameMetadata::default();
        m.fill_from(
            GameMetadata {
                cover_url: Some("https://a".into()),
                ..Default::default()
            },
            MetadataProvider::SteamGridDb,
        );
        m.fill_from(
            GameMetadata {
                cover_url: Some("https://b".into()),
                description: Some("d".into()),
                ..Default::default()
            },
            MetadataProvider::Igdb,
        );
        m.fill_from(GameMetadata::default(), MetadataProvider::Rawg);
        assert_eq!(m.cover_url.as_deref(), Some("https://a"));
        assert_eq!(m.description.as_deref(), Some("d"));
        assert_eq!(
            m.sources,
            [MetadataProvider::SteamGridDb, MetadataProvider::Igdb]
        );
    }

    #[test]
    fn fetch_requires_providers() {
        let g = Game::new(
            Platform::Local,
            "x",
            "X",
            crate::models::LaunchTarget::uri("https://x"),
        );
        assert_eq!(
            fetch(&g, &[], &ApiKeys::default(), "en", 0)
                .unwrap_err()
                .code,
            ErrorCode::MetadataNoProviders
        );
    }

    #[test]
    fn missing_keys_fail_before_network() {
        let g = Game::new(
            Platform::Local,
            "x",
            "X",
            crate::models::LaunchTarget::uri("https://x"),
        );
        for p in [MetadataProvider::SteamGridDb, MetadataProvider::Igdb] {
            assert_eq!(
                fetch(&g, &[p], &ApiKeys::default(), "en", 0)
                    .unwrap_err()
                    .code,
                ErrorCode::MetadataMissingKey
            );
        }
    }

    #[test]
    fn legacy_rawg_provider_never_makes_a_request() {
        let game = Game::new(
            Platform::Local,
            "x",
            "X",
            crate::models::LaunchTarget::uri("https://x"),
        );
        let error = fetch(
            &game,
            &[MetadataProvider::Rawg],
            &ApiKeys::default(),
            "en",
            0,
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::MetadataRequestFailed);
        assert_eq!(
            error.detail.as_deref(),
            Some("RAWG is discontinued; use IGDB or another enabled provider")
        );
    }

    #[test]
    fn helpers() {
        assert_eq!(plain_text("<p>a</p>\n\n<p>b  c</p>"), "a\n\nb c");
        assert_eq!(unix_to_ymd(0), "1970-01-01");
        assert_eq!(steam_language("pt-BR"), "brazilian");
        assert_eq!(steam_language("zh-Hant"), "tchinese");
        assert_eq!(steam_language("de-AT"), "german");
        assert_eq!(steam_language("xx"), "english");
    }
}
