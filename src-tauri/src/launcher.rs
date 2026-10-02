//! Starts games. Only [`LaunchTarget`]s produced by the backend (scanners or
//! validated custom games) are ever launched – the UI passes a game *id*,
//! never a raw path or URI.

use crate::error::{AppError, AppResult, ErrorCode};
use crate::models::LaunchTarget;
use std::process::{Command, Stdio};

/// URI schemes the launcher is allowed to hand to the operating system.
pub const ALLOWED_SCHEMES: &[&str] = &[
    "steam",
    "com.epicgames.launcher",
    "goggalaxy",
    "humble",
    "uplay",
    "origin2",
    "link2ea",
    "amazon-games",
    "battlenet",
    "itch",
    "heroic",
    "lutris",
    "ms-windows-store",
    "https",
];

pub fn uri_scheme(uri: &str) -> Option<&str> {
    let (scheme, _) = uri.split_once(':')?;
    let valid = !scheme.is_empty()
        && scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    valid.then_some(scheme)
}

pub fn check_uri(uri: &str) -> AppResult<()> {
    match uri_scheme(uri) {
        Some(s) if ALLOWED_SCHEMES.iter().any(|a| a.eq_ignore_ascii_case(s)) => Ok(()),
        _ => Err(AppError::with(ErrorCode::UriSchemeNotAllowed, uri)),
    }
}

/// Build the OS command for an executable target.
pub fn build_command(
    path: &std::path::Path,
    args: &[String],
    working_dir: Option<&std::path::Path>,
) -> Command {
    let mut cmd = if cfg!(target_os = "macos") && path.extension().is_some_and(|e| e == "app") {
        let mut c = Command::new("open");
        c.arg("-a").arg(path);
        if !args.is_empty() {
            c.arg("--args").args(args);
        }
        c
    } else {
        let mut c = Command::new(path);
        c.args(args);
        c
    };
    if let Some(dir) = working_dir.filter(|d| d.is_dir()) {
        cmd.current_dir(dir);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd
}

/// Launch a target. `open_uri` is injected so the Tauri opener plugin can be
/// used in the app and a fake in tests.
pub fn launch(
    target: &LaunchTarget,
    open_uri: impl FnOnce(&str) -> Result<(), String>,
) -> AppResult<()> {
    match target {
        LaunchTarget::Uri { uri } => {
            check_uri(uri)?;
            open_uri(uri).map_err(|e| AppError::with(ErrorCode::LaunchFailed, e))
        }
        LaunchTarget::Executable {
            path,
            args,
            working_dir,
        } => {
            let mut child = build_command(path, args, working_dir.as_deref())
                .spawn()
                .map_err(|e| {
                    AppError::with(ErrorCode::LaunchFailed, format!("{}: {e}", path.display()))
                })?;
            // Reap the process when it exits so no zombie is left behind on Unix.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_store_schemes_only() {
        for ok in [
            "steam://rungameid/570",
            "uplay://launch/635/0",
            "https://play.geforcenow.com",
            "com.epicgames.launcher://apps/x",
        ] {
            assert!(check_uri(ok).is_ok(), "{ok}");
        }
        for bad in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "http://x",
            "/usr/bin/sh",
            "",
            "1abc://x",
            "smb://host/share",
        ] {
            assert_eq!(
                check_uri(bad).unwrap_err().code,
                ErrorCode::UriSchemeNotAllowed,
                "{bad}"
            );
        }
    }

    #[test]
    fn uri_launch_uses_opener() {
        let mut opened = String::new();
        launch(&LaunchTarget::uri("steam://rungameid/1"), |u| {
            opened = u.to_string();
            Ok(())
        })
        .unwrap();
        assert_eq!(opened, "steam://rungameid/1");

        let err = launch(&LaunchTarget::uri("steam://x"), |_| Err("boom".into())).unwrap_err();
        assert_eq!(err.code, ErrorCode::LaunchFailed);

        let err = launch(&LaunchTarget::uri("file:///x"), |_| panic!("must not open")).unwrap_err();
        assert_eq!(err.code, ErrorCode::UriSchemeNotAllowed);
    }

    #[test]
    fn missing_executable_reports_launch_failed() {
        let target = LaunchTarget::exe("/definitely/not/a/real/binary-ugl");
        assert_eq!(
            launch(&target, |_| Ok(())).unwrap_err().code,
            ErrorCode::LaunchFailed
        );
    }

    #[cfg(unix)]
    #[test]
    fn spawns_executables() {
        let target = LaunchTarget::Executable {
            path: "/bin/sh".into(),
            args: vec!["-c".into(), "exit 0".into()],
            working_dir: Some("/".into()),
        };
        assert!(launch(&target, |_| Ok(())).is_ok());
    }
}
