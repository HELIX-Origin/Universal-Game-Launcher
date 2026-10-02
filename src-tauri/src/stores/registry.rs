//! Thin Windows-registry helpers (Windows only).

use winreg::enums::{
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY,
};
use winreg::{RegKey, HKEY};

fn open(hive: HKEY, path: &str, flags: u32) -> Option<RegKey> {
    RegKey::predef(hive)
        .open_subkey_with_flags(path, KEY_READ | flags)
        .ok()
}

/// Open a key in both the 32-bit and 64-bit views (deduplicated by caller).
pub fn open_both_views(hive: HKEY, path: &str) -> Vec<RegKey> {
    [KEY_WOW64_32KEY, KEY_WOW64_64KEY]
        .into_iter()
        .filter_map(|f| open(hive, path, f))
        .collect()
}

pub fn get_string(key: &RegKey, name: &str) -> Option<String> {
    key.get_value::<String, _>(name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn hkcu_string(path: &str, name: &str) -> Option<String> {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(path)
        .ok()
        .and_then(|k| get_string(&k, name))
}

pub fn hklm_string(path: &str, name: &str) -> Option<String> {
    open_both_views(HKEY_LOCAL_MACHINE, path)
        .iter()
        .find_map(|k| get_string(k, name))
}

/// `(subkey name, subkey)` pairs below `HKLM\<path>` in both registry views.
pub fn hklm_subkeys(path: &str) -> Vec<(String, RegKey)> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for key in open_both_views(HKEY_LOCAL_MACHINE, path) {
        for name in key.enum_keys().filter_map(Result::ok) {
            if seen.insert(name.to_ascii_lowercase()) {
                if let Ok(sub) = key.open_subkey(&name) {
                    out.push((name, sub));
                }
            }
        }
    }
    out
}

/// All "Add/Remove programs" entries (HKLM both views + HKCU).
pub fn uninstall_entries() -> Vec<(String, RegKey)> {
    const PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    let mut out = hklm_subkeys(PATH);
    if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey(PATH) {
        for name in key.enum_keys().filter_map(Result::ok) {
            if let Ok(sub) = key.open_subkey(&name) {
                out.push((name, sub));
            }
        }
    }
    out
}
