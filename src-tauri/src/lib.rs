mod paths;
mod scan;
mod clean;
mod history;
mod helper_ipc;
mod util;

use serde::Serialize;
use tauri::Manager;
use tracing_subscriber::EnvFilter;

use scan::ScanReport;
use clean::{CleanPlan, CleanResult};
use history::HistoryEntry;

#[derive(Serialize)]
struct Platform {
    os: String,
    supported: bool,
}

#[tauri::command]
fn platform() -> Platform {
    Platform {
        os: std::env::consts::OS.to_string(),
        supported: cfg!(target_os = "windows"),
    }
}

#[tauri::command]
async fn scan(include_prefetch: bool) -> Result<ScanReport, String> {
    tauri::async_runtime::spawn_blocking(move || scan::run(include_prefetch))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean(app: tauri::AppHandle, plan: CleanPlan) -> Result<CleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || clean::run(&app, plan))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn history(app: tauri::AppHandle) -> Result<Vec<HistoryEntry>, String> {
    history::list(&app).map_err(|e| e.to_string())
}

#[tauri::command]
fn restore(app: tauri::AppHandle, history_id: String) -> Result<(), String> {
    history::restore(&app, &history_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn launch_uninstaller(uninstall_string: String) -> Result<(), String> {
    clean::launch_uninstaller(&uninstall_string).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            history::init(&handle).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            platform,
            scan,
            clean,
            history,
            restore,
            launch_uninstaller,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
