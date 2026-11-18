use serde::{Deserialize, Serialize};

use super::config::default_language;
use super::{AppConfig, AutoRotateConfig};

/// The part of the configuration that is backed up and synced between PCs.
/// Machine-specific values (wallpaper folder, sync folder, first-run flags)
/// are left out on purpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncedSettings {
    pub shortcut: String,
    pub taskbar_transparent: bool,
    pub auto_rotate: AutoRotateConfig,
    pub favorites: Vec<String>,
    /// Missing in files written before languages existed.
    #[serde(default = "default_language")]
    pub language: String,
}

impl From<&AppConfig> for SyncedSettings {
    fn from(config: &AppConfig) -> Self {
        Self {
            shortcut: config.shortcut.clone(),
            taskbar_transparent: config.taskbar_transparent,
            auto_rotate: config.auto_rotate.clone(),
            favorites: config.favorites.clone(),
            language: config.language.clone(),
        }
    }
}

/// File format shared by backups (`easy-paper-backup.json`) and the sync
/// file (`easy-paper-sync.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDocument {
    /// Always `"easy-paper"`, so unrelated JSON files are rejected.
    pub app: String,
    pub version: u32,
    /// Unix time (ms) of the change these settings come from.
    pub updated_at: u64,
    /// Name of the PC that wrote the file.
    pub device: String,
    pub settings: SyncedSettings,
}

pub const SYNC_DOCUMENT_APP: &str = "easy-paper";
pub const SYNC_DOCUMENT_VERSION: u32 = 1;

impl SyncDocument {
    pub fn new(settings: SyncedSettings, updated_at: u64, device: String) -> Self {
        Self {
            app: SYNC_DOCUMENT_APP.to_string(),
            version: SYNC_DOCUMENT_VERSION,
            updated_at,
            device,
            settings,
        }
    }

    pub fn parse(content: &str) -> Result<Self, String> {
        let not_a_backup = || {
            crate::tr!(
                en: "This file is not an Easy Paper backup.",
                pt: "Este arquivo não é um backup do Easy Paper.",
            )
        };
        let document: SyncDocument = serde_json::from_str(content).map_err(|_| not_a_backup())?;
        if document.app != SYNC_DOCUMENT_APP {
            return Err(not_a_backup());
        }
        if document.version > SYNC_DOCUMENT_VERSION {
            return Err(crate::tr!(
                en: "This backup was made by a newer version of Easy Paper. Update the app first.",
                pt: "Este backup foi feito por uma versão mais nova do Easy Paper. Atualize o app primeiro.",
            ));
        }
        Ok(document)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupSummary {
    /// Backup folder (export) or backup file (import).
    pub path: String,
    /// Wallpaper images copied.
    pub images: usize,
    /// Problems that did not stop the operation (e.g. a shortcut in use).
    pub warnings: Vec<String>,
}

/// In-memory sync status, shown in Settings.
#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncStatus {
    /// Unix time (ms) of the last successful check of the sync folder.
    pub last_checked_at: Option<u64>,
    /// What the last check did: "pushed", "pulled" or "up_to_date".
    pub last_action: Option<String>,
    /// PC that last wrote the sync file.
    pub remote_device: Option<String>,
    pub last_error: Option<String>,
}

/// What an existing sync file contains, shown before connecting to it.
#[derive(Debug, Clone, Serialize)]
pub struct SyncFolderInfo {
    pub device: String,
    pub updated_at: u64,
    pub favorites: usize,
}
