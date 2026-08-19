use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UninstallEntry {
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub install_date: Option<String>,
    pub install_location: Option<String>,
    pub estimated_size: Option<u64>,
    pub uninstall_string: String,
    pub heavy_startup: bool,
}

#[cfg(target_os = "windows")]
pub fn scan() -> Result<Vec<UninstallEntry>> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut out = Vec::new();
    let sources: [(HKEY, &str); 4] = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        (HKEY_CURRENT_USER, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
    ];

    for (hive, path) in sources {
        let root = match RegKey::predef(hive).open_subkey(path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub in root.enum_keys().flatten() {
            let key = match root.open_subkey(&sub) {
                Ok(k) => k,
                Err(_) => continue,
            };
            let name: String = match key.get_value("DisplayName") {
                Ok(v) => v,
                Err(_) => continue,
            };
            let uninstall_string: String = match key.get_value("UninstallString") {
                Ok(v) => v,
                Err(_) => continue,
            };
            let publisher: Option<String> = key.get_value("Publisher").ok();
            let version: Option<String> = key.get_value("DisplayVersion").ok();
            let install_date: Option<String> = key.get_value::<String, _>("InstallDate")
                .ok()
                .and_then(format_install_date);
            let install_location: Option<String> = key.get_value("InstallLocation").ok();
            let estimated_size: Option<u64> = key.get_value::<u32, _>("EstimatedSize").ok().map(|kb| (kb as u64) * 1024);
            let heavy_startup = heavy_startup_heuristic(&name);

            out.push(UninstallEntry {
                id: format!("u:{}:{}", hive as usize, sub),
                name,
                publisher,
                version,
                install_date,
                install_location,
                estimated_size,
                uninstall_string,
                heavy_startup,
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
pub fn scan() -> Result<Vec<UninstallEntry>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
fn format_install_date(raw: String) -> Option<String> {
    if raw.len() == 8 && raw.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8]))
    } else {
        Some(raw)
    }
}

fn heavy_startup_heuristic(name: &str) -> bool {
    let n = name.to_lowercase();
    const HEAVY: &[&str] = &[
        "spotify", "discord", "slack", "teams", "onedrive", "dropbox", "steam",
        "epic games", "adobe creative cloud", "adobe acrobat", "skype", "zoom",
        "razer", "logitech", "asus armoury", "nvidia geforce experience", "corsair icue",
        "microsoft edge webview", "cortana", "cyberlink", "wechat", "qq", "utorrent",
    ];
    HEAVY.iter().any(|k| n.contains(k))
}
