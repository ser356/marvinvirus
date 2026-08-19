use crate::clean::{CleanFailure, StartupToggle};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct HelperPlan {
    pub files: Vec<String>,
    pub startup_toggle: Vec<HelperStartup>,
}

#[derive(Serialize, Debug, Clone)]
pub struct HelperStartup {
    pub id: String,
    pub enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind")]
pub enum HelperMsg {
    #[serde(rename = "ok")]
    Ok { path: String, freed: u64 },
    #[serde(rename = "err")]
    Err { path: String, reason: String },
    #[serde(rename = "done")]
    Done,
}

#[cfg(target_os = "windows")]
pub fn run_elevated(
    files: &[String],
    startup: &[StartupToggle],
) -> (Vec<String>, Vec<CleanFailure>, u64) {
    use std::io::{BufRead, BufReader, Write};

    let mut ok = Vec::new();
    let mut failed = Vec::new();
    let mut freed = 0u64;

    let helper_path = match locate_helper() {
        Some(p) => p,
        None => {
            for f in files {
                failed.push(CleanFailure { path: f.clone(), reason: "helper no encontrado".into() });
            }
            return (ok, failed, freed);
        }
    };

    let (pipe_name, pipe_handle) = match create_pipe() {
        Ok(v) => v,
        Err(e) => {
            for f in files {
                failed.push(CleanFailure { path: f.clone(), reason: format!("pipe: {e}") });
            }
            return (ok, failed, freed);
        }
    };

    let launched = launch_runas(&helper_path, &pipe_name);
    if let Err(e) = launched {
        for f in files {
            failed.push(CleanFailure { path: f.clone(), reason: format!("runas: {e}") });
        }
        return (ok, failed, freed);
    }

    let mut server = pipe_handle;
    let plan = HelperPlan {
        files: files.to_vec(),
        startup_toggle: startup
            .iter()
            .map(|s| HelperStartup { id: s.id.clone(), enabled: s.enabled })
            .collect(),
    };
    let serialized = serde_json::to_string(&plan).unwrap_or_default();
    let _ = server.write_all(serialized.as_bytes());
    let _ = server.write_all(b"\n");
    let _ = server.flush();

    let reader = BufReader::new(server);
    for line in reader.lines().flatten() {
        match serde_json::from_str::<HelperMsg>(&line) {
            Ok(HelperMsg::Ok { path, freed: f }) => { freed += f; ok.push(path); }
            Ok(HelperMsg::Err { path, reason }) => failed.push(CleanFailure { path, reason }),
            Ok(HelperMsg::Done) => break,
            Err(_) => continue,
        }
    }

    (ok, failed, freed)
}

#[cfg(not(target_os = "windows"))]
pub fn run_elevated(
    files: &[String],
    _startup: &[StartupToggle],
) -> (Vec<String>, Vec<CleanFailure>, u64) {
    let failed: Vec<CleanFailure> = files
        .iter()
        .map(|f| CleanFailure { path: f.clone(), reason: "helper elevado solo Windows".into() })
        .collect();
    (Vec::new(), failed, 0)
}

#[cfg(target_os = "windows")]
fn locate_helper() -> Option<std::path::PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let candidate = dir.join("marvinvirus_helper.exe");
    if candidate.exists() {
        return Some(candidate);
    }
    let sibling = dir.join("../../../../helper/target/release/marvinvirus_helper.exe");
    if sibling.exists() { Some(sibling) } else { None }
}

#[cfg(target_os = "windows")]
fn create_pipe() -> std::io::Result<(String, std::fs::File)> {
    use std::os::windows::io::FromRawHandle;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::{
        PIPE_ACCESS_DUPLEX, FILE_FLAG_FIRST_PIPE_INSTANCE,
    };
    use windows::Win32::System::Pipes::{
        CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT,
    };
    use std::os::windows::ffi::OsStrExt;

    let pipe_name = format!("\\\\.\\pipe\\marvinvirus-{}", uuid::Uuid::new_v4());
    let wide: Vec<u16> = std::ffi::OsStr::new(&pipe_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let handle = unsafe {
        CreateNamedPipeW(
            PCWSTR(wide.as_ptr()),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1,
            1 << 16,
            1 << 16,
            0,
            None,
        )
    };
    let handle = handle.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let file = unsafe { std::fs::File::from_raw_handle(handle.0 as _) };
    Ok((pipe_name, file))
}

#[cfg(target_os = "windows")]
fn launch_runas(exe: &std::path::Path, pipe: &str) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    let file: Vec<u16> = exe.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let params: Vec<u16> = std::ffi::OsStr::new(pipe).encode_wide().chain(std::iter::once(0)).collect();
    let verb = w!("runas");
    unsafe {
        let hinst = ShellExecuteW(
            None,
            verb,
            PCWSTR(file.as_ptr()),
            PCWSTR(params.as_ptr()),
            None,
            SW_HIDE,
        );
        if (hinst.0 as isize) <= 32 {
            return Err(std::io::Error::last_os_error());
        }
    }
    Ok(())
}
