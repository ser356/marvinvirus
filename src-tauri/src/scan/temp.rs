use anyhow::Result;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::paths::{self, RootKind};
use super::{Category, FileItem, Risk};

pub fn scan(include_prefetch: bool) -> Result<Vec<FileItem>> {
    if !cfg!(target_os = "windows") {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for (root, base) in paths::resolved_roots() {
        if !is_temp_root(root.kind) {
            continue;
        }
        if root.kind == RootKind::Prefetch && !include_prefetch {
            continue;
        }
        let (risk, preselect) = classify(root.kind);
        walk_files(&base, root.kind, root.requires_elevation, risk, preselect, &mut out);
    }
    Ok(out)
}

fn is_temp_root(k: RootKind) -> bool {
    matches!(
        k,
        RootKind::UserTemp
            | RootKind::WindowsTemp
            | RootKind::WindowsUpdateCache
            | RootKind::Thumbnails
            | RootKind::WerReports
            | RootKind::Prefetch
    )
}

fn classify(k: RootKind) -> (Risk, bool) {
    match k {
        RootKind::UserTemp | RootKind::WindowsTemp | RootKind::WerReports => (Risk::Safe, true),
        RootKind::WindowsUpdateCache => (Risk::Review, false),
        RootKind::Thumbnails => (Risk::Safe, true),
        RootKind::Prefetch => (Risk::Sensitive, false),
        _ => (Risk::Review, false),
    }
}

fn walk_files(
    base: &std::path::Path,
    _kind: RootKind,
    requires_elevation: bool,
    risk: Risk,
    preselect: bool,
    out: &mut Vec<FileItem>,
) {
    for entry in WalkDir::new(base).follow_links(false).into_iter().flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        let md = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size = md.len();
        if size == 0 {
            continue;
        }
        let path = entry.path().to_string_lossy().to_string();
        out.push(FileItem {
            id: format!("f:{}", Uuid::new_v4()),
            path,
            size,
            category: Category::SystemTemp,
            risk,
            requires_elevation,
            preselect,
            label: None,
        });
    }
}
