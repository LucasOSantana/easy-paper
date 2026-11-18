export interface BackupSummary {
  /** Backup folder (export) or backup file (import). */
  path: string;
  /** Wallpaper images copied. */
  images: number;
  /** Problems that did not stop the operation. */
  warnings: string[];
}

export type SyncAction = "pushed" | "pulled" | "up_to_date";

export interface SyncStatus {
  last_checked_at: number | null;
  last_action: SyncAction | null;
  remote_device: string | null;
  last_error: string | null;
}

/** What an existing sync file contains, shown before connecting to it. */
export interface SyncFolderInfo {
  device: string;
  updated_at: number;
  favorites: number;
}

export const CONFIG_CHANGED_EVENT = "config-changed";
export const SYNC_STATUS_EVENT = "sync-status";
