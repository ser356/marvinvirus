import { invoke } from '@tauri-apps/api/core'
import type {
  CleanPlan,
  CleanResult,
  HistoryEntry,
  ScanReport,
} from './types'

export type ScanProgress = {
  category: string
  scanned: number
  found: number
  done: boolean
}

export const api = {
  scan(includePrefetch = false): Promise<ScanReport> {
    return invoke<ScanReport>('scan', { includePrefetch })
  },
  clean(plan: CleanPlan): Promise<CleanResult> {
    return invoke<CleanResult>('clean', { plan })
  },
  history(): Promise<HistoryEntry[]> {
    return invoke<HistoryEntry[]>('history')
  },
  restore(historyId: string): Promise<void> {
    return invoke<void>('restore', { historyId })
  },
  launchUninstaller(uninstallString: string): Promise<void> {
    return invoke<void>('launch_uninstaller', { uninstallString })
  },
  platform(): Promise<{ os: string; supported: boolean }> {
    return invoke('platform')
  },
}
