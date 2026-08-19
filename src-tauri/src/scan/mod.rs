pub mod temp;
pub mod browser;
pub mod uninstall;
pub mod duplicates;
pub mod startup;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use uninstall::UninstallEntry;
use duplicates::DuplicateGroup;
use startup::StartupEntry;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    SystemTemp,
    BrowserCache,
    Uninstall,
    Duplicates,
    LargeFiles,
    Startup,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Safe,
    Review,
    Sensitive,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileItem {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub category: Category,
    pub risk: Risk,
    pub requires_elevation: bool,
    pub preselect: bool,
    pub label: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanReport {
    pub files: Vec<FileItem>,
    pub uninstalls: Vec<UninstallEntry>,
    pub duplicates: Vec<DuplicateGroup>,
    pub large_files: Vec<FileItem>,
    pub startup: Vec<StartupEntry>,
    pub scanned_at: String,
    pub reclaimable_bytes: u64,
}

pub fn run(include_prefetch: bool) -> Result<ScanReport> {
    let files_temp = temp::scan(include_prefetch)?;
    let files_browser = browser::scan()?;
    let uninstalls = uninstall::scan()?;
    let (duplicates, large_files) = duplicates::scan()?;
    let startup = startup::scan()?;

    let mut files = files_temp;
    files.extend(files_browser);

    let reclaimable_bytes = files.iter().map(|f| f.size).sum::<u64>();

    Ok(ScanReport {
        files,
        uninstalls,
        duplicates,
        large_files,
        startup,
        scanned_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
        reclaimable_bytes,
    })
}
