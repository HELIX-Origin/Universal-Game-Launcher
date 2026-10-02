//! Minimal parser for Valve's text KeyValues format (`.vdf` / `.acf`).
//!
//! Supports quoted/unquoted tokens, nested objects, `//` comments, escape
//! sequences and `[$PLATFORM]` conditionals (which are ignored). Keys are
//! matched case-insensitively by [`Vdf::get`] because Valve files are not
//! consistent about casing (`AppState` vs `appstate`).

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Value(String),
    Object(BTreeMap<String, Vdf>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Object(map) => map.get(key).or_else(|| {
                map.iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(key))
                    .map(|(_, v)| v)
            }),
            Vdf::Value(_) => None,
        }
    }

    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Vdf::Value(v) => Some(v.as_str()),
            Vdf::Object(_) => None,
        }
    }

    pub fn as_object(&self) -> Option<&BTreeMap<String, Vdf>> {
        match self {
            Vdf::Object(map) => Some(map),
            Vdf::Value(_) => None,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Open,
    Close,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '/' => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    for c in chars.by_ref() {
                        if c == '\n' {
                            break;
                        }
                    }
                } else {
                    return Err("unexpected '/'".into());
                }
            }
            '{' => {
                chars.next();
                tokens.push(Token::Open);
            }
            '}' => {
                chars.next();
                tokens.push(Token::Close);
            }
            '[' => {
                // Conditional such as [$WIN32] – skip it.
                for c in chars.by_ref() {
                    if c == ']' {
                        break;
                    }
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                let mut closed = false;
                while let Some(c) = chars.next() {
                    match c {
                        '"' => {
                            closed = true;
                            break;
                        }
                        '\\' => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some('\\') => s.push('\\'),
                            Some('"') => s.push('"'),
                            Some(other) => {
                                s.push('\\');
                                s.push(other);
                            }
                            None => break,
                        },
                        c => s.push(c),
                    }
                }
                if !closed {
                    return Err("unterminated string".into());
                }
                tokens.push(Token::Str(s));
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '{' || c == '}' || c == '"' {
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                tokens.push(Token::Str(s));
            }
        }
    }
    Ok(tokens)
}

fn parse_object<I: Iterator<Item = Token>>(
    tokens: &mut std::iter::Peekable<I>,
    nested: bool,
) -> Result<BTreeMap<String, Vdf>, String> {
    let mut map = BTreeMap::new();
    loop {
        match tokens.next() {
            None if nested => return Err("unexpected end of input".into()),
            None => return Ok(map),
            Some(Token::Close) if nested => return Ok(map),
            Some(Token::Close) => return Err("unexpected '}'".into()),
            Some(Token::Open) => return Err("unexpected '{'".into()),
            Some(Token::Str(key)) => {
                let value = match tokens.next() {
                    Some(Token::Str(v)) => Vdf::Value(v),
                    Some(Token::Open) => Vdf::Object(parse_object(tokens, true)?),
                    _ => return Err(format!("missing value for key '{key}'")),
                };
                map.insert(key, value);
            }
        }
    }
}

/// Parse a KeyValues document. The returned root is an object containing the
/// top-level key(s), e.g. `{"libraryfolders": {...}}`.
pub fn parse(input: &str) -> Result<Vdf, String> {
    let input = input.trim_start_matches('\u{feff}');
    let tokens = tokenize(input)?;
    let mut iter = tokens.into_iter().peekable();
    Ok(Vdf::Object(parse_object(&mut iter, false)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_objects_and_comments() {
        let doc = r#"
        // comment
        "AppState"
        {
            "appid"		"570"
            "name"		"Dota \"2\""
            "installdir"		"dota 2 beta"
            "UserConfig" { "language" "english" }
            unquoted value
        }"#;
        let v = parse(doc).unwrap();
        let app = v.get("appstate").unwrap();
        assert_eq!(app.str("appid"), Some("570"));
        assert_eq!(app.str("name"), Some("Dota \"2\""));
        assert_eq!(app.str("installdir"), Some("dota 2 beta"));
        assert_eq!(
            app.get("UserConfig").unwrap().str("language"),
            Some("english")
        );
        assert_eq!(app.str("unquoted"), Some("value"));
    }

    #[test]
    fn handles_windows_paths_and_conditionals() {
        let doc =
            r#""libraryfolders" { "0" { "path" "C:\\Program Files (x86)\\Steam" [$WIN32] } }"#;
        let v = parse(doc).unwrap();
        let path = v
            .get("libraryfolders")
            .unwrap()
            .get("0")
            .unwrap()
            .str("path");
        assert_eq!(path, Some(r"C:\Program Files (x86)\Steam"));
    }

    #[test]
    fn rejects_unbalanced_documents() {
        assert!(parse(r#""a" { "b" "c""#).is_err());
        assert!(parse(r#""a" "b" }"#).is_err());
        assert!(parse(r#""a" "unterminated"#).is_err());
    }
}
