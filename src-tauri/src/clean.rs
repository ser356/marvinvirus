use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::history::{self, HistoryEntry};
use crate::helper_ipc;
use crate::paths;

#[derive(Deserialize, Debug, Clone)]
pub struct StartupToggle {
    pub id: String,
    pub enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CleanPlan {
    pub files: Vec<String>,
    pub startup_toggle: Vec<StartupToggle>,
    pub uninstall_ids: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct CleanFailure {
    pub path: String,
    pub reason: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CleanResult {
    pub freed_bytes: u64,
    pub ok: Vec<String>,
    pub failed: Vec<CleanFailure>,
    pub history_id: String,
}

pub fn run(app: &tauri::AppHandle, plan: CleanPlan) -> Result<CleanResult> {
    let (local_paths, elevated_paths) = split_by_elevation(&plan.files);

    let mut ok = Vec::new();
    let mut failed = Vec::new();
    let mut freed: u64 = 0;
    let mut restore_items: Vec<history::RestoreItem> = Vec::new();

    for path in &local_paths {
        match delete_local(path) {
            Ok(size) => {
                freed += size;
                ok.push(path.clone());
                restore_items.push(history::RestoreItem {
                    path: path.clone(),
                    recycle_original: path.clone(),
                });
            }
            Err(e) => failed.push(CleanFailure { path: path.clone(), reason: e.to_string() }),
        }
    }

    if !elevated_paths.is_empty() || !plan.startup_toggle.is_empty() {
        let (helper_ok, helper_failed, helper_bytes) =
            helper_ipc::run_elevated(&elevated_paths, &plan.startup_toggle);
        freed += helper_bytes;
        for p in &helper_ok {
            restore_items.push(history::RestoreItem {
                path: p.clone(),
                recycle_original: p.clone(),
            });
        }
        ok.extend(helper_ok);
        failed.extend(helper_failed);
    }

    let history_id = format!("h:{}", Uuid::new_v4());
    let entry = HistoryEntry {
        id: history_id.clone(),
        at: time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        freed_bytes: freed,
        restored: false,
        items: restore_items,
    };
    history::append(app, &entry)?;

    Ok(CleanResult { freed_bytes: freed, ok, failed, history_id })
}

fn split_by_elevation(files: &[String]) -> (Vec<String>, Vec<String>) {
    let mut local = Vec::new();
    let mut elevated = Vec::new();
    for f in files {
        let p = Path::new(f);
        if !paths::is_within_any_root(p) {
            continue;
        }
        if needs_elevation(p) {
            elevated.push(f.clone());
        } else {
            local.push(f.clone());
        }
    }
    (local, elevated)
}

fn needs_elevation(p: &Path) -> bool {
    let s = p.to_string_lossy().to_lowercase();
    s.starts_with("c:\\windows") || s.contains("\\programdata\\")
}

fn delete_local(path: &str) -> Result<u64> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(anyhow!("no existe"));
    }
    if !paths::is_within_any_root(p) {
        return Err(anyhow!("fuera de whitelist"));
    }
    let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    trash::delete(p).map_err(|e| anyhow!(e.to_string()))?;
    Ok(size)
}

#[cfg(target_os = "windows")]
pub fn launch_uninstaller(uninstall_string: &str) -> Result<()> {
    use std::process::Command;
    let (exe, args) = split_command_line(uninstall_string);
    Command::new(&exe)
        .args(&args)
        .spawn()
        .map(|_| ())
        .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn launch_uninstaller(_uninstall_string: &str) -> Result<()> {
    Err(anyhow!("solo Windows"))
}

fn split_command_line(cmd: &str) -> (String, Vec<String>) {
    let trimmed = cmd.trim();
    if trimmed.starts_with('"') {
        if let Some(end) = trimmed[1..].find('"') {
            let exe = trimmed[1..1 + end].to_string();
            let rest = trimmed[2 + end..].trim().to_string();
            let args = if rest.is_empty() { Vec::new() } else { vec![rest] };
            return (exe, args);
        }
    }
    let mut parts = trimmed.splitn(2, ' ');
    let exe = parts.next().unwrap_or("").to_string();
    let args = parts.next().map(|s| vec![s.to_string()]).unwrap_or_default();
    (exe, args)
}
