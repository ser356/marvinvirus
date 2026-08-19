use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RestoreItem {
    pub path: String,
    pub recycle_original: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HistoryEntry {
    pub id: String,
    pub at: String,
    pub freed_bytes: u64,
    pub restored: bool,
    pub items: Vec<RestoreItem>,
}

pub fn init(app: &tauri::AppHandle) -> Result<()> {
    let dir = history_dir(app)?;
    std::fs::create_dir_all(&dir).map_err(|e| anyhow!(e.to_string()))?;
    let file = dir.join("history.json");
    if !file.exists() {
        std::fs::write(&file, b"[]").map_err(|e| anyhow!(e.to_string()))?;
    }
    Ok(())
}

pub fn list(app: &tauri::AppHandle) -> Result<Vec<HistoryEntry>> {
    let file = history_file(app)?;
    if !file.exists() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&file).map_err(|e| anyhow!(e.to_string()))?;
    let mut v: Vec<HistoryEntry> = serde_json::from_str(&raw).unwrap_or_default();
    v.reverse();
    Ok(v)
}

pub fn append(app: &tauri::AppHandle, entry: &HistoryEntry) -> Result<()> {
    let file = history_file(app)?;
    let raw = std::fs::read_to_string(&file).unwrap_or_else(|_| "[]".to_string());
    let mut v: Vec<HistoryEntry> = serde_json::from_str(&raw).unwrap_or_default();
    v.push(entry.clone());
    let ser = serde_json::to_string_pretty(&v).map_err(|e| anyhow!(e.to_string()))?;
    std::fs::write(&file, ser).map_err(|e| anyhow!(e.to_string()))
}

pub fn restore(app: &tauri::AppHandle, id: &str) -> Result<()> {
    let file = history_file(app)?;
    let raw = std::fs::read_to_string(&file).map_err(|e| anyhow!(e.to_string()))?;
    let mut v: Vec<HistoryEntry> = serde_json::from_str(&raw).unwrap_or_default();
    let entry = v.iter_mut().find(|e| e.id == id).ok_or_else(|| anyhow!("no encontrado"))?;
    if entry.restored {
        return Err(anyhow!("ya restaurado"));
    }
    open_recycle_bin()?;
    entry.restored = true;
    let ser = serde_json::to_string_pretty(&v).map_err(|e| anyhow!(e.to_string()))?;
    std::fs::write(&file, ser).map_err(|e| anyhow!(e.to_string()))
}

#[cfg(target_os = "windows")]
fn open_recycle_bin() -> Result<()> {
    use std::process::Command;
    Command::new("explorer.exe")
        .arg("shell:RecycleBinFolder")
        .spawn()
        .map(|_| ())
        .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(not(target_os = "windows"))]
fn open_recycle_bin() -> Result<()> {
    Ok(())
}

fn history_dir(app: &tauri::AppHandle) -> Result<PathBuf> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map_err(|e| anyhow!(e.to_string()))
}

fn history_file(app: &tauri::AppHandle) -> Result<PathBuf> {
    Ok(history_dir(app)?.join("history.json"))
}
