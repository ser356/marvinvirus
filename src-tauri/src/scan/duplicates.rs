use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::paths::{self, RootKind};
use super::{Category, FileItem, Risk};

const MIN_SIZE: u64 = 1024 * 1024;
const LARGE_FILE: u64 = 100 * 1024 * 1024;
const HASH_BUF: usize = 256 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub paths: Vec<String>,
}

pub fn scan() -> Result<(Vec<DuplicateGroup>, Vec<FileItem>)> {
    if !cfg!(target_os = "windows") {
        return Ok((Vec::new(), Vec::new()));
    }

    let user_roots: Vec<PathBuf> = paths::resolved_roots()
        .into_iter()
        .filter(|(r, _)| {
            matches!(
                r.kind,
                RootKind::UserDocs
                    | RootKind::UserDownloads
                    | RootKind::UserDesktop
                    | RootKind::UserPictures
                    | RootKind::UserVideos
            )
        })
        .map(|(_, p)| p)
        .collect();

    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut large_files: Vec<FileItem> = Vec::new();

    for root in &user_roots {
        for entry in WalkDir::new(root).follow_links(false).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if size < MIN_SIZE {
                continue;
            }
            let path = entry.path().to_path_buf();
            if size >= LARGE_FILE {
                large_files.push(FileItem {
                    id: format!("l:{}", Uuid::new_v4()),
                    path: path.to_string_lossy().to_string(),
                    size,
                    category: Category::LargeFiles,
                    risk: Risk::Sensitive,
                    requires_elevation: false,
                    preselect: false,
                    label: None,
                });
            }
            by_size.entry(size).or_default().push(path);
        }
    }

    let candidates: Vec<(u64, Vec<PathBuf>)> = by_size
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .collect();

    let groups: Vec<DuplicateGroup> = candidates
        .par_iter()
        .flat_map(|(size, paths)| {
            let mut per_hash: HashMap<String, Vec<String>> = HashMap::new();
            for p in paths {
                if let Some(h) = hash_file(p) {
                    per_hash.entry(h).or_default().push(p.to_string_lossy().to_string());
                }
            }
            per_hash
                .into_iter()
                .filter(|(_, ps)| ps.len() > 1)
                .map(|(hash, paths)| DuplicateGroup { hash, size: *size, paths })
                .collect::<Vec<_>>()
        })
        .collect();

    large_files.sort_by(|a, b| b.size.cmp(&a.size));
    Ok((groups, large_files))
}

fn hash_file(path: &std::path::Path) -> Option<String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).ok()?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = vec![0u8; HASH_BUF];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(hasher.finalize().to_hex().to_string())
}
