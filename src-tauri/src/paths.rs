use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub kind: RootKind,
    pub env: &'static str,
    pub suffix: &'static str,
    pub requires_elevation: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootKind {
    UserTemp,
    WindowsTemp,
    WindowsUpdateCache,
    Thumbnails,
    WerReports,
    Prefetch,
    ChromeCache,
    EdgeCache,
    FirefoxCache,
    UserDocs,
    UserDownloads,
    UserDesktop,
    UserPictures,
    UserVideos,
}

pub const ROOTS: &[Root] = &[
    Root { kind: RootKind::UserTemp, env: "TEMP", suffix: "", requires_elevation: false },
    Root { kind: RootKind::WindowsTemp, env: "SystemRoot", suffix: "Temp", requires_elevation: true },
    Root { kind: RootKind::WindowsUpdateCache, env: "SystemRoot", suffix: "SoftwareDistribution\\Download", requires_elevation: true },
    Root { kind: RootKind::Thumbnails, env: "LOCALAPPDATA", suffix: "Microsoft\\Windows\\Explorer", requires_elevation: false },
    Root { kind: RootKind::WerReports, env: "LOCALAPPDATA", suffix: "Microsoft\\Windows\\WER", requires_elevation: false },
    Root { kind: RootKind::Prefetch, env: "SystemRoot", suffix: "Prefetch", requires_elevation: true },
    Root { kind: RootKind::ChromeCache, env: "LOCALAPPDATA", suffix: "Google\\Chrome\\User Data", requires_elevation: false },
    Root { kind: RootKind::EdgeCache, env: "LOCALAPPDATA", suffix: "Microsoft\\Edge\\User Data", requires_elevation: false },
    Root { kind: RootKind::FirefoxCache, env: "LOCALAPPDATA", suffix: "Mozilla\\Firefox\\Profiles", requires_elevation: false },
    Root { kind: RootKind::UserDocs, env: "USERPROFILE", suffix: "Documents", requires_elevation: false },
    Root { kind: RootKind::UserDownloads, env: "USERPROFILE", suffix: "Downloads", requires_elevation: false },
    Root { kind: RootKind::UserDesktop, env: "USERPROFILE", suffix: "Desktop", requires_elevation: false },
    Root { kind: RootKind::UserPictures, env: "USERPROFILE", suffix: "Pictures", requires_elevation: false },
    Root { kind: RootKind::UserVideos, env: "USERPROFILE", suffix: "Videos", requires_elevation: false },
];

pub fn resolve(root: &Root) -> Option<PathBuf> {
    if !cfg!(target_os = "windows") {
        return None;
    }
    let base = std::env::var_os(root.env)?;
    let mut p = PathBuf::from(base);
    if !root.suffix.is_empty() {
        p.push(root.suffix);
    }
    Some(p)
}

pub fn resolved_roots() -> Vec<(Root, PathBuf)> {
    ROOTS
        .iter()
        .filter_map(|r| resolve(r).map(|p| (*r, p)))
        .filter(|(_, p)| p.exists())
        .collect()
}

pub fn is_within_any_root(p: &Path) -> bool {
    let target = match canonicalize(p) {
        Ok(c) => c,
        Err(_) => return false,
    };
    for r in ROOTS {
        if let Some(base) = resolve(r) {
            if let Ok(cbase) = canonicalize(&base) {
                if target.starts_with(&cbase) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn canonicalize(p: &Path) -> std::io::Result<PathBuf> {
    #[cfg(windows)]
    { dunce::canonicalize(p) }
    #[cfg(not(windows))]
    { std::fs::canonicalize(p) }
}
