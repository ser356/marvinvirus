use anyhow::{anyhow, Result};
use std::path::Path;

use crate::whitelist;

pub fn delete_to_recycle(path: &str) -> Result<u64> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(anyhow!("no existe"));
    }
    let canon = dunce::canonicalize(p).map_err(|e| anyhow!(e.to_string()))?;
    if !whitelist::is_within(&canon) {
        return Err(anyhow!("fuera de whitelist"));
    }
    let size = std::fs::metadata(&canon).map(|m| m.len()).unwrap_or(0);
    trash::delete(&canon).map_err(|e| anyhow!(e.to_string()))?;
    Ok(size)
}

#[cfg(target_os = "windows")]
pub fn apply_startup(id: &str, enabled: bool) -> Result<()> {
    if let Some(name) = id.strip_prefix("s:reg:HklmRun:") {
        return toggle_startup_approved(winreg::enums::HKEY_LOCAL_MACHINE, name, enabled);
    }
    if let Some(name) = id.strip_prefix("s:reg:HkcuRun:") {
        return toggle_startup_approved(winreg::enums::HKEY_CURRENT_USER, name, enabled);
    }
    if let Some(name) = id.strip_prefix("s:task:") {
        return toggle_task(name, enabled);
    }
    Err(anyhow!("id de arranque no soportado por el helper"))
}

#[cfg(not(target_os = "windows"))]
pub fn apply_startup(_id: &str, _enabled: bool) -> Result<()> {
    Err(anyhow!("solo Windows"))
}

#[cfg(target_os = "windows")]
fn toggle_startup_approved(hive: winreg::HKEY, name: &str, enabled: bool) -> Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let root = RegKey::predef(hive);
    let (key, _) = root
        .create_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
            KEY_READ | KEY_WRITE,
        )
        .map_err(|e| anyhow!(e.to_string()))?;

    let payload: Vec<u8> = if enabled {
        vec![0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    } else {
        vec![0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    };
    key.set_raw_value(
        name,
        &winreg::RegValue { bytes: payload, vtype: REG_BINARY },
    )
    .map_err(|e| anyhow!(e.to_string()))
}

#[cfg(target_os = "windows")]
fn toggle_task(name: &str, enabled: bool) -> Result<()> {
    use std::process::Command;
    use crate::util::NoWindow;
    let flag = if enabled { "/Enable" } else { "/Disable" };
    let status = Command::new("schtasks.exe")
        .args(["/Change", "/TN", name, flag])
        .no_window()
        .status()
        .map_err(|e| anyhow!(e.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow!("schtasks fallo: {}", status))
    }
}
