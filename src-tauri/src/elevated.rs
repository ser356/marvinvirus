use crate::clean::{CleanFailure, StartupToggle};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ElevatedPlan {
    pub files: Vec<String>,
    pub startup_toggle: Vec<PlanStartup>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanStartup {
    pub id: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum ElevatedMsg {
    #[serde(rename = "ok")]
    Ok { path: String, freed: u64 },
    #[serde(rename = "err")]
    Err { path: String, reason: String },
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ElevatedResult {
    pub messages: Vec<ElevatedMsg>,
}

pub fn run_elevated(
    files: &[String],
    startup: &[StartupToggle],
) -> (Vec<String>, Vec<CleanFailure>, u64) {
    let mut ok = Vec::new();
    let mut failed = Vec::new();
    let mut freed = 0u64;

    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            fail_all(&mut failed, files, startup, format!("current_exe: {e}"));
            return (ok, failed, freed);
        }
    };

    let plan = ElevatedPlan {
        files: files.to_vec(),
        startup_toggle: startup
            .iter()
            .map(|s| PlanStartup { id: s.id.clone(), enabled: s.enabled })
            .collect(),
    };

    let tmp = std::env::temp_dir();
    let uid = uuid::Uuid::new_v4();
    let in_path = tmp.join(format!("marvinvirus-{uid}-in.json"));
    let out_path = tmp.join(format!("marvinvirus-{uid}-out.json"));

    let bytes = match serde_json::to_vec(&plan) {
        Ok(v) => v,
        Err(e) => {
            fail_all(&mut failed, files, startup, format!("serialize: {e}"));
            return (ok, failed, freed);
        }
    };
    if let Err(e) = std::fs::write(&in_path, &bytes) {
        fail_all(&mut failed, files, startup, format!("escribir plan: {e}"));
        return (ok, failed, freed);
    }

    if let Err(e) = spawn_elevated_and_wait(&exe, &in_path, &out_path) {
        fail_all(&mut failed, files, startup, format!("elevación: {e}"));
        let _ = std::fs::remove_file(&in_path);
        let _ = std::fs::remove_file(&out_path);
        return (ok, failed, freed);
    }

    let result: ElevatedResult = match std::fs::read(&out_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
    {
        Some(r) => r,
        None => {
            fail_all(&mut failed, files, startup, "salida del proceso elevado vacía o inválida".into());
            let _ = std::fs::remove_file(&in_path);
            let _ = std::fs::remove_file(&out_path);
            return (ok, failed, freed);
        }
    };
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);

    for m in result.messages {
        match m {
            ElevatedMsg::Ok { path, freed: f } => {
                freed += f;
                ok.push(path);
            }
            ElevatedMsg::Err { path, reason } => failed.push(CleanFailure { path, reason }),
        }
    }

    (ok, failed, freed)
}

fn fail_all(failed: &mut Vec<CleanFailure>, files: &[String], startup: &[StartupToggle], reason: String) {
    for f in files {
        failed.push(CleanFailure { path: f.clone(), reason: reason.clone() });
    }
    for s in startup {
        failed.push(CleanFailure { path: s.id.clone(), reason: reason.clone() });
    }
}

pub fn apply_from_cli(in_path: &str, out_path: &str) -> std::io::Result<()> {
    let bytes = std::fs::read(in_path)?;
    let plan: ElevatedPlan = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut messages = Vec::new();
    for f in &plan.files {
        match delete_to_recycle(f) {
            Ok(size) => messages.push(ElevatedMsg::Ok { path: f.clone(), freed: size }),
            Err(e) => messages.push(ElevatedMsg::Err { path: f.clone(), reason: e }),
        }
    }
    for s in &plan.startup_toggle {
        match apply_startup(&s.id, s.enabled) {
            Ok(()) => messages.push(ElevatedMsg::Ok { path: s.id.clone(), freed: 0 }),
            Err(e) => messages.push(ElevatedMsg::Err { path: s.id.clone(), reason: e }),
        }
    }

    let out = ElevatedResult { messages };
    let json = serde_json::to_vec(&out)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(out_path, json)
}

fn delete_to_recycle(path: &str) -> Result<u64, String> {
    let p = Path::new(path);
    if !p.exists() {
        return Err("no existe".into());
    }
    if !crate::paths::is_within_any_root(p) {
        return Err("fuera de whitelist".into());
    }
    let size = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    trash::delete(p).map_err(|e| e.to_string())?;
    Ok(size)
}

#[cfg(target_os = "windows")]
fn apply_startup(id: &str, enabled: bool) -> Result<(), String> {
    if let Some(name) = id.strip_prefix("s:reg:HklmRun:") {
        return toggle_startup_approved(winreg::enums::HKEY_LOCAL_MACHINE, name, enabled);
    }
    if let Some(name) = id.strip_prefix("s:reg:HkcuRun:") {
        return toggle_startup_approved(winreg::enums::HKEY_CURRENT_USER, name, enabled);
    }
    if let Some(name) = id.strip_prefix("s:task:") {
        return toggle_task(name, enabled);
    }
    Err("id de arranque no soportado".into())
}

#[cfg(not(target_os = "windows"))]
fn apply_startup(_id: &str, _enabled: bool) -> Result<(), String> {
    Err("solo Windows".into())
}

#[cfg(target_os = "windows")]
fn toggle_startup_approved(hive: winreg::HKEY, name: &str, enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let root = RegKey::predef(hive);
    let (key, _) = root
        .create_subkey_with_flags(
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
            KEY_READ | KEY_WRITE,
        )
        .map_err(|e| e.to_string())?;

    let payload: Vec<u8> = if enabled {
        vec![0x02, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    } else {
        vec![0x03, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    };
    key.set_raw_value(
        name,
        &winreg::RegValue { bytes: payload, vtype: REG_BINARY },
    )
    .map_err(|e| e.to_string())
}

#[cfg(target_os = "windows")]
fn toggle_task(name: &str, enabled: bool) -> Result<(), String> {
    use crate::util::NoWindow;
    use std::process::Command;
    let flag = if enabled { "/Enable" } else { "/Disable" };
    let status = Command::new("schtasks.exe")
        .args(["/Change", "/TN", name, flag])
        .no_window()
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("schtasks fallo: {status}"))
    }
}

#[cfg(target_os = "windows")]
fn spawn_elevated_and_wait(
    exe: &Path,
    in_path: &Path,
    out_path: &Path,
) -> std::io::Result<()> {
    use crate::util::NoWindow;
    use std::process::Command;

    let script = format!(
        "$ErrorActionPreference='Stop';$p=Start-Process -FilePath '{exe}' -ArgumentList @('--apply-elevated','{in_}','{out}') -Verb RunAs -WindowStyle Hidden -PassThru;$p.WaitForExit();exit $p.ExitCode",
        exe = ps_quote(&exe.to_string_lossy()),
        in_ = ps_quote(&in_path.to_string_lossy()),
        out = ps_quote(&out_path.to_string_lossy()),
    );
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .no_window()
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("relanzamiento elevado falló: {status}"),
        ))
    }
}

#[cfg(not(target_os = "windows"))]
fn spawn_elevated_and_wait(
    _exe: &Path,
    _in_path: &Path,
    _out_path: &Path,
) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "elevación solo Windows",
    ))
}

#[cfg(target_os = "windows")]
fn ps_quote(s: &str) -> String {
    s.replace('\'', "''")
}
