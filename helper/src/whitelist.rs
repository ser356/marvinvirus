use std::path::{Path, PathBuf};

const SEGMENTS: &[(&str, &str)] = &[
    ("TEMP", ""),
    ("SystemRoot", "Temp"),
    ("SystemRoot", "SoftwareDistribution\\Download"),
    ("LOCALAPPDATA", "Microsoft\\Windows\\Explorer"),
    ("LOCALAPPDATA", "Microsoft\\Windows\\WER"),
    ("SystemRoot", "Prefetch"),
    ("LOCALAPPDATA", "Google\\Chrome\\User Data"),
    ("LOCALAPPDATA", "Microsoft\\Edge\\User Data"),
    ("LOCALAPPDATA", "Mozilla\\Firefox\\Profiles"),
    ("USERPROFILE", "Documents"),
    ("USERPROFILE", "Downloads"),
    ("USERPROFILE", "Desktop"),
    ("USERPROFILE", "Pictures"),
    ("USERPROFILE", "Videos"),
];

pub fn roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for (env, suffix) in SEGMENTS {
        if let Some(base) = std::env::var_os(env) {
            let mut p = PathBuf::from(base);
            if !suffix.is_empty() {
                p.push(suffix);
            }
            if let Ok(c) = dunce::canonicalize(&p) {
                out.push(c);
            }
        }
    }
    out
}

pub fn is_within(target: &Path) -> bool {
    let canon = match dunce::canonicalize(target) {
        Ok(c) => c,
        Err(_) => return false,
    };
    roots().iter().any(|r| canon.starts_with(r))
}
