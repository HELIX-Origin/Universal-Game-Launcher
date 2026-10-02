//! Xbox app / Microsoft Store PC games (Windows). Packages are enumerated
//! with `Get-AppxPackage`; a package is a game when its install folder has a
//! GDK `MicrosoftGame.config`. Games launch through the shell's AppsFolder
//! (`explorer.exe shell:AppsFolder\<PFN>!<AppId>`), exactly like the Start
//! menu / Xbox app.

use super::{ScanContext, ScanResult};
use crate::models::{Game, LaunchTarget, Platform};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// PowerShell that prints a JSON array of `{Name, PackageFamilyName, InstallLocation, AppId}`.
pub const PS_SCRIPT: &str = "$ErrorActionPreference='SilentlyContinue'; \
Get-AppxPackage | Where-Object { -not $_.IsFramework -and $_.SignatureKind -eq 'Store' -and $_.InstallLocation -and (Test-Path (Join-Path $_.InstallLocation 'MicrosoftGame.config')) } | \
ForEach-Object { $m = Get-AppxPackageManifest $_; [pscustomobject]@{ Name=$_.Name; PackageFamilyName=$_.PackageFamilyName; InstallLocation=$_.InstallLocation; AppId=@($m.Package.Applications.Application)[0].Id } } | \
ConvertTo-Json -Compress";

/// `DefaultDisplayName` from `MicrosoftGame.config`.
pub fn parse_game_config(xml: &str) -> Option<String> {
    let doc = roxmltree::Document::parse(xml).ok()?;
    doc.descendants()
        .find(|n| n.has_tag_name("ShellVisuals"))
        .and_then(|n| n.attribute("DefaultDisplayName"))
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.starts_with("ms-resource:"))
        .map(String::from)
}

pub fn app_target(pfn: &str, app_id: &str) -> LaunchTarget {
    LaunchTarget::Executable {
        path: PathBuf::from("explorer.exe"),
        args: vec![format!("shell:AppsFolder\\{pfn}!{app_id}")],
        working_dir: None,
    }
}

/// Turn the PowerShell JSON (object or array) into games. `read_config`
/// returns the contents of `MicrosoftGame.config` for an install folder.
pub fn parse_packages(json: &str, read_config: impl Fn(&Path) -> Option<String>) -> Vec<Game> {
    let v: Value = match serde_json::from_str(json.trim()) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let items = match v {
        Value::Array(a) => a,
        obj @ Value::Object(_) => vec![obj],
        _ => Vec::new(),
    };
    items
        .iter()
        .filter_map(|p| {
            let pfn = p["PackageFamilyName"].as_str()?;
            let app_id = p["AppId"].as_str()?;
            let dir = PathBuf::from(p["InstallLocation"].as_str()?);
            let title = read_config(&dir)
                .as_deref()
                .and_then(parse_game_config)
                .or_else(|| p["Name"].as_str().map(String::from))?;
            Some(
                Game::new(Platform::Xbox, pfn, title, app_target(pfn, app_id))
                    .with_install_dir(dir),
            )
        })
        .collect()
}

pub fn scan(ctx: &ScanContext) -> ScanResult {
    let _ = ctx;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let out = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                PS_SCRIPT,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("powershell: {e}"))?;
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Ok(parse_packages(&stdout, |dir| {
            super::read_text(&dir.join("MicrosoftGame.config"))
        }));
    }
    #[allow(unreachable_code)]
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<Game configVersion="1">
  <Identity Name="Microsoft.624F8B84B80" Publisher="CN=Microsoft" Version="1.0.0.0"/>
  <ExecutableList><Executable Name="ForzaHorizon5.exe" Id="Game"/></ExecutableList>
  <ShellVisuals DefaultDisplayName="Forza Horizon 5" PublisherDisplayName="Xbox Game Studios"/>
</Game>"#;

    #[test]
    fn parses_powershell_output() {
        let json = r#"{"Name":"Microsoft.624F8B84B80","PackageFamilyName":"Microsoft.624F8B84B80_8wekyb3d8bbwe","InstallLocation":"C:\\XboxGames\\Forza Horizon 5\\Content","AppId":"ForzaHorizon5"}"#;
        let games = parse_packages(json, |_| Some(CONFIG.to_string()));
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].title, "Forza Horizon 5");
        assert_eq!(games[0].id, "xbox:Microsoft.624F8B84B80_8wekyb3d8bbwe");
        assert_eq!(
            games[0].launch,
            app_target("Microsoft.624F8B84B80_8wekyb3d8bbwe", "ForzaHorizon5")
        );
        assert!(parse_packages("", |_| None).is_empty());
        assert_eq!(
            parse_packages(&format!("[{json}]"), |_| None)[0].title,
            "Microsoft.624F8B84B80"
        );
    }

    #[test]
    fn ignores_resource_display_names() {
        assert_eq!(
            parse_game_config(&CONFIG.replace("Forza Horizon 5", "ms-resource:Title")),
            None
        );
    }
}
