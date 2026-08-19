use anyhow::Result;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::paths::{self, RootKind};
use super::{Category, FileItem, Risk};

pub fn scan() -> Result<Vec<FileItem>> {
    if !cfg!(target_os = "windows") {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for (root, base) in paths::resolved_roots() {
        match root.kind {
            RootKind::ChromeCache | RootKind::EdgeCache => scan_chromium(&base, &mut out),
            RootKind::FirefoxCache => scan_firefox(&base, &mut out),
            _ => {}
        }
    }
    Ok(out)
}

fn scan_chromium(user_data: &Path, out: &mut Vec<FileItem>) {
    let candidates = ["Cache", "Code Cache", "GPUCache", "Service Worker\\CacheStorage"];
    if let Ok(entries) = std::fs::read_dir(user_data) {
        for e in entries.flatten() {
            let profile = e.path();
            if !profile.is_dir() {
                continue;
            }
            let name = profile.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !(name == "Default" || name.starts_with("Profile ")) {
                continue;
            }
            for c in &candidates {
                let dir = profile.join(c);
                if dir.exists() {
                    push_dir(&dir, out);
                }
            }
        }
    }
}

fn scan_firefox(profiles_root: &Path, out: &mut Vec<FileItem>) {
    if let Ok(entries) = std::fs::read_dir(profiles_root) {
        for e in entries.flatten() {
            let p: PathBuf = e.path();
            if !p.is_dir() {
                continue;
            }
            let cache = p.join("cache2");
            if cache.exists() {
                push_dir(&cache, out);
            }
        }
    }
}

fn push_dir(dir: &Path, out: &mut Vec<FileItem>) {
    for entry in WalkDir::new(dir).follow_links(false).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
        if size == 0 {
            continue;
        }
        out.push(FileItem {
            id: format!("b:{}", Uuid::new_v4()),
            path: entry.path().to_string_lossy().to_string(),
            size,
            category: Category::BrowserCache,
            risk: Risk::Safe,
            requires_elevation: false,
            preselect: true,
            label: None,
        });
    }
}
