//! Deep links into each platform's **official app** ("Open app" / "Open
//! store"). The launcher never implements a storefront itself: browsing,
//! purchasing, downloading and signing in all happen in these apps.

use crate::models::{LaunchTarget, Platform};
#[allow(unused_imports)]
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientActions {
    pub open: Option<LaunchTarget>,
    pub store: Option<LaunchTarget>,
}

#[allow(dead_code)]
fn exe_if_exists(path: PathBuf) -> Option<LaunchTarget> {
    path.exists().then(|| LaunchTarget::exe(path))
}

#[allow(dead_code)]
fn in_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(name))
            .find(|p| p.is_file())
    })
}

#[cfg(target_os = "macos")]
fn mac_app(name: &str) -> Option<LaunchTarget> {
    exe_if_exists(PathBuf::from("/Applications").join(name))
}

/// Path of the official client, when it can be located on this machine.
fn client_exe(platform: Platform) -> Option<LaunchTarget> {
    #[cfg(windows)]
    {
        use crate::stores::registry::{get_string, hkcu_string, hklm_string, uninstall_entries};
        let local = dirs::data_local_dir().unwrap_or_default();
        return match platform {
            Platform::Gog => {
                hklm_string(r"SOFTWARE\WOW6432Node\GOG.com\GalaxyClient\paths", "client")
                    .and_then(|d| exe_if_exists(PathBuf::from(d).join("GalaxyClient.exe")))
            }
            Platform::Humble => hkcu_string(
                r"SOFTWARE\2f793df2-2969-529d-b0c0-7960ed40d70e",
                "InstallLocation",
            )
            .or_else(|| {
                hklm_string(
                    r"SOFTWARE\2f793df2-2969-529d-b0c0-7960ed40d70e",
                    "InstallLocation",
                )
            })
            .and_then(|d| exe_if_exists(PathBuf::from(d).join("Humble App.exe"))),
            Platform::Itch => exe_if_exists(local.join(r"itch\itch.exe")),
            Platform::Ubisoft => {
                hklm_string(r"SOFTWARE\WOW6432Node\Ubisoft\Launcher", "InstallDir").and_then(|d| {
                    let d = PathBuf::from(d);
                    exe_if_exists(d.join("UbisoftConnect.exe"))
                        .or_else(|| exe_if_exists(d.join("upc.exe")))
                })
            }
            Platform::Ea => exe_if_exists(PathBuf::from(
                r"C:\Program Files\Electronic Arts\EA Desktop\EA Desktop\EADesktop.exe",
            )),
            Platform::Origin => hklm_string(r"SOFTWARE\WOW6432Node\Origin", "ClientPath")
                .and_then(|p| exe_if_exists(PathBuf::from(p))),
            Platform::Amazon => exe_if_exists(local.join(r"Amazon Games\App\Amazon Games.exe")),
            Platform::BattleNet => uninstall_entries()
                .into_iter()
                .find(|(_, k)| {
                    get_string(k, "UninstallString").is_some_and(|u| u.contains("--uid=battle.net"))
                })
                .and_then(|(_, k)| get_string(&k, "InstallLocation"))
                .and_then(|d| exe_if_exists(PathBuf::from(d).join("Battle.net.exe"))),
            _ => None,
        };
    }
    #[cfg(target_os = "macos")]
    {
        return match platform {
            Platform::Gog => mac_app("GOG Galaxy.app"),
            Platform::Itch => mac_app("itch.app"),
            Platform::Ubisoft => mac_app("Ubisoft Connect.app"),
            Platform::Ea => mac_app("EA app.app"),
            Platform::Origin => mac_app("Origin.app"),
            Platform::BattleNet => mac_app("Battle.net.app"),
            _ => None,
        };
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return match platform {
            Platform::Lutris => in_path("lutris").map(LaunchTarget::exe),
            Platform::Itch => dirs::home_dir().and_then(|h| exe_if_exists(h.join(".itch/itch"))),
            _ => None,
        };
    }
    #[allow(unreachable_code)]
    {
        let _ = platform;
        None
    }
}

pub fn actions(platform: Platform) -> ClientActions {
    let uri = |u: &str| Some(LaunchTarget::uri(u));
    match platform {
        Platform::Steam => ClientActions {
            open: uri("steam://open/games"),
            store: uri("steam://store"),
        },
        Platform::Epic if cfg!(any(windows, target_os = "macos")) => ClientActions {
            open: uri("com.epicgames.launcher://apps"),
            store: uri("com.epicgames.launcher://store"),
        },
        Platform::Xbox if cfg!(windows) => ClientActions {
            open: Some(LaunchTarget::Executable {
                path: PathBuf::from("explorer.exe"),
                args: vec![
                    r"shell:AppsFolder\Microsoft.GamingApp_8wekyb3d8bbwe!Microsoft.Xbox.App".into(),
                ],
                working_dir: None,
            }),
            store: uri("ms-windows-store://home"),
        },
        Platform::GeforceNow => ClientActions {
            open: uri("https://play.geforcenow.com/"),
            store: None,
        },
        Platform::Xcloud => ClientActions {
            open: uri("https://www.xbox.com/play"),
            store: None,
        },
        p => ClientActions {
            open: client_exe(p),
            store: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_actions_pass_the_launcher_allowlist() {
        for p in Platform::ALL {
            let a = actions(p);
            for t in [a.open, a.store].into_iter().flatten() {
                if let LaunchTarget::Uri { uri } = t {
                    assert!(crate::launcher::check_uri(&uri).is_ok(), "{uri}");
                }
            }
        }
    }

    #[test]
    fn steam_and_cloud_always_have_actions() {
        assert!(actions(Platform::Steam).open.is_some());
        assert!(actions(Platform::Steam).store.is_some());
        assert!(actions(Platform::GeforceNow).open.is_some());
        assert!(actions(Platform::Xcloud).open.is_some());
        assert_eq!(actions(Platform::Local), ClientActions::default());
    }
}
