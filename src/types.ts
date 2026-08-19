export type Category =
  | 'system_temp'
  | 'browser_cache'
  | 'uninstall'
  | 'duplicates'
  | 'large_files'
  | 'startup'

export type Risk = 'safe' | 'review' | 'sensitive'

export interface FileItem {
  id: string
  path: string
  size: number
  category: Category
  risk: Risk
  requires_elevation: boolean
  preselect: boolean
  label?: string
}

export interface UninstallEntry {
  id: string
  name: string
  publisher?: string
  version?: string
  install_date?: string
  install_location?: string
  estimated_size?: number
  uninstall_string: string
  heavy_startup: boolean
}

export interface DuplicateGroup {
  hash: string
  size: number
  paths: string[]
}

export type StartupSource =
  | 'hklm_run'
  | 'hkcu_run'
  | 'hklm_runonce'
  | 'hkcu_runonce'
  | 'startup_folder_user'
  | 'startup_folder_common'
  | 'scheduled_task'
  | 'windows_startup_apps'

export interface StartupEntry {
  id: string
  name: string
  command: string
  source: StartupSource
  enabled: boolean
  requires_elevation: boolean
}

export interface ScanReport {
  files: FileItem[]
  uninstalls: UninstallEntry[]
  duplicates: DuplicateGroup[]
  large_files: FileItem[]
  startup: StartupEntry[]
  scanned_at: string
  reclaimable_bytes: number
}

export interface CleanPlan {
  files: string[]
  startup_toggle: { id: string; enabled: boolean }[]
  uninstall_ids: string[]
}

export interface CleanResult {
  freed_bytes: number
  ok: string[]
  failed: { path: string; reason: string }[]
  history_id: string
}

export interface HistoryEntry {
  id: string
  at: string
  freed_bytes: number
  restored: boolean
  items: { path: string; recycle_original: string }[]
}

export type Step = 'scan' | 'review' | 'confirm' | 'clean' | 'restore'
