#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

mod ops;
mod util;
mod whitelist;

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};

#[derive(Deserialize, Debug)]
struct HelperPlan {
    files: Vec<String>,
    startup_toggle: Vec<HelperStartup>,
}

#[derive(Deserialize, Debug)]
struct HelperStartup {
    id: String,
    enabled: bool,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
enum HelperMsg {
    #[serde(rename = "ok")]
    Ok { path: String, freed: u64 },
    #[serde(rename = "err")]
    Err { path: String, reason: String },
    #[serde(rename = "done")]
    Done,
}

fn main() {
    let pipe = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("uso: marvinvirus_helper <pipe>");
            std::process::exit(2);
        }
    };
    if let Err(e) = run(&pipe) {
        eprintln!("helper error: {e}");
        std::process::exit(1);
    }
}

fn run(pipe: &str) -> anyhow::Result<()> {
    let stream = connect(pipe)?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    let mut header = String::new();
    reader.read_line(&mut header)?;
    let plan: HelperPlan = serde_json::from_str(header.trim())?;

    for f in &plan.files {
        match ops::delete_to_recycle(f) {
            Ok(size) => write_msg(&mut writer, HelperMsg::Ok { path: f.clone(), freed: size }),
            Err(e) => write_msg(&mut writer, HelperMsg::Err { path: f.clone(), reason: e.to_string() }),
        }
    }

    for s in &plan.startup_toggle {
        match ops::apply_startup(&s.id, s.enabled) {
            Ok(()) => write_msg(&mut writer, HelperMsg::Ok { path: s.id.clone(), freed: 0 }),
            Err(e) => write_msg(&mut writer, HelperMsg::Err { path: s.id.clone(), reason: e.to_string() }),
        }
    }

    write_msg(&mut writer, HelperMsg::Done);
    writer.flush()?;
    Ok(())
}

fn write_msg<W: Write>(w: &mut W, msg: HelperMsg) {
    if let Ok(json) = serde_json::to_string(&msg) {
        let _ = w.write_all(json.as_bytes());
        let _ = w.write_all(b"\n");
        let _ = w.flush();
    }
}

#[cfg(target_os = "windows")]
fn connect(pipe: &str) -> anyhow::Result<std::fs::File> {
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::FromRawHandle;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::GENERIC_READ;
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_EXISTING,
    };

    let wide: Vec<u16> = std::ffi::OsStr::new(pipe)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            (GENERIC_READ.0 | FILE_GENERIC_WRITE.0) as u32,
            FILE_SHARE_NONE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };
    Ok(unsafe { std::fs::File::from_raw_handle(handle.0 as _) })
}

#[cfg(not(target_os = "windows"))]
fn connect(_pipe: &str) -> anyhow::Result<std::fs::File> {
    anyhow::bail!("helper solo soportado en Windows")
}
