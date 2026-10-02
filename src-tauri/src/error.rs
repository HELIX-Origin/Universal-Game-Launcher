//! Structured, localizable errors.
//!
//! The backend never sends human-readable English to the UI. Instead every
//! error carries a stable `code` that the frontend translates with the
//! `errors.<code>` key of the active locale (see `src/lib/i18n`), plus an
//! optional untranslated `detail` (a path, OS error, ...).

use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: ErrorCode,
    pub detail: Option<String>,
}

/// Keep in sync with the `errors` section of `src/lib/i18n/locales/en.ts`
/// (enforced by `tests/contracts/enums.test.ts`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    TitleRequired,
    TitleTooLong,
    InvalidCoverUrl,
    ExecutableRequired,
    ExecutableNotAbsolute,
    ExecutableNotFound,
    UrlRequired,
    UrlNotHttps,
    UrlHostNotAllowed,
    UnsupportedPlatform,
    GameNotFound,
    LaunchFailed,
    UriSchemeNotAllowed,
    NoInstallDir,
    Storage,
    MetadataNoProviders,
    MetadataMissingKey,
    MetadataRequestFailed,
    MetadataNotFound,
    Internal,
}

impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        AppError { code, detail: None }
    }

    pub fn with(code: ErrorCode, detail: impl Into<String>) -> Self {
        AppError {
            code,
            detail: Some(detail.into()),
        }
    }
}

impl From<ErrorCode> for AppError {
    fn from(code: ErrorCode) -> Self {
        AppError::new(code)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.detail {
            Some(d) => write!(f, "{:?}: {d}", self.code),
            None => write!(f, "{:?}", self.code),
        }
    }
}

impl std::error::Error for AppError {}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_camel_case_code() {
        let json =
            serde_json::to_value(AppError::with(ErrorCode::ExecutableNotFound, "/x")).unwrap();
        assert_eq!(json["code"], "executableNotFound");
        assert_eq!(json["detail"], "/x");
    }
}
