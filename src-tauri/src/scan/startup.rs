use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartupSource {
    HklmRun,
    HkcuRun,
    HklmRunonce,
    HkcuRunonce,
    StartupFolderUser,
    StartupFolderCommon,
    ScheduledTask,
    WindowsStartupApps,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StartupEntry {
    pub id: String,
    pub name: String,
    pub command: String,
    pub source: StartupSource,
    pub enabled: bool,
    pub requires_elevation: bool,
}

#[cfg(target_os = "windows")]
pub fn scan() -> Result<Vec<StartupEntry>> {
    let mut out = Vec::new();
    scan_run_keys(&mut out);
    scan_startup_folders(&mut out);
    scan_scheduled_tasks(&mut out);
    Ok(out)
}

#[cfg(not(target_os = "windows"))]
pub fn scan() -> Result<Vec<StartupEntry>> {
    Ok(Vec::new())
}

#[cfg(target_os = "windows")]
fn scan_run_keys(out: &mut Vec<StartupEntry>) {
    use winreg::enums::*;
    use winreg::RegKey;

    let sources: [(HKEY, &str, StartupSource, bool); 4] = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", StartupSource::HklmRun, true),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce", StartupSource::HklmRunonce, true),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run", StartupSource::HkcuRun, false),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce", StartupSource::HkcuRunonce, false),
    ];

    for (hive, path, source, elev) in sources {
        let key = match RegKey::predef(hive).open_subkey(path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for (name, value) in key.enum_values().flatten() {
            let command = match value.to_string().is_empty() {
                true => continue,
                false => value.to_string(),
            };
            out.push(StartupEntry {
                id: format!("s:reg:{:?}:{}", source, name),
                name,
                command,
                source,
                enabled: true,
                requires_elevation: elev,
            });
        }
    }

    let disabled: [(HKEY, &str, StartupSource); 2] = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run", StartupSource::HklmRun),
        (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run", StartupSource::HkcuRun),
    ];
    for (hive, path, source) in disabled {
        let key = match RegKey::predef(hive).open_subkey(path) {
            Ok(k) => k,
            Err(_) => continue,
        };
        for (name, value) in key.enum_values().flatten() {
            let raw: Vec<u8> = value.bytes;
            if raw.first().copied().unwrap_or(2) == 2 {
                for e in out.iter_mut() {
                    if e.source == source && e.name == name {
                        e.enabled = false;
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn scan_startup_folders(out: &mut Vec<StartupEntry>) {
    use std::path::PathBuf;

    let user = std::env::var_os("APPDATA")
        .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs\Startup"));
    let common = std::env::var_os("ProgramData")
        .map(|p| PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs\Startup"));

    for (dir_opt, source, elev) in [
        (user, StartupSource::StartupFolderUser, false),
        (common, StartupSource::StartupFolderCommon, true),
    ] {
        let dir = match dir_opt {
            Some(d) if d.exists() => d,
            _ => continue,
        };
        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = e.path();
            let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if name.is_empty() {
                continue;
            }
            out.push(StartupEntry {
                id: format!("s:folder:{:?}:{}", source, name),
                name,
                command: p.to_string_lossy().to_string(),
                source,
                enabled: true,
                requires_elevation: elev,
            });
        }
    }
}

#[cfg(target_os = "windows")]
fn scan_scheduled_tasks(out: &mut Vec<StartupEntry>) {
    use std::process::Command;
    use crate::util::NoWindow;

    let output = Command::new("schtasks.exe")
        .args(["/Query", "/FO", "CSV", "/V", "/NH"])
        .no_window()
        .output();

    let out_bytes = match output {
        Ok(o) if o.status.success() => o.stdout,
        _ => return,
    };
    let text = String::from_utf8_lossy(&out_bytes);

    for line in text.lines() {
        let cols: Vec<&str> = line.split("\",\"").collect();
        if cols.len() < 10 {
            continue;
        }
        let name = cols[1].trim_start_matches('"').to_string();
        let author = cols[6];
        let cmd = cols[8].to_string();
        let status = cols[3];
        let start_trigger = cols[7];

        if author.starts_with("Microsoft") || author.starts_with("N/A") {
            continue;
        }
        if !start_trigger.contains("logon") && !start_trigger.contains("boot") {
            continue;
        }
        out.push(StartupEntry {
            id: format!("s:task:{}", name),
            name: name.clone(),
            command: cmd.trim_end_matches('"').to_string(),
            source: StartupSource::ScheduledTask,
            enabled: !status.contains("Disabled"),
            requires_elevation: true,
        });
    }
}
