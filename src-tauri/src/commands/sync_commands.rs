use std::path::PathBuf;

use crate::models::{AppConfig, BackupSummary, SyncFolderInfo, SyncStatus};
use crate::services::sync_service::{self, BACKUP_FILE_NAME};

async fn blocking<T: Send + 'static>(
    task: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| crate::tr!(
            en: format!("The operation was interrupted: {}", e),
            pt: format!("A operação foi interrompida: {}", e),
        ))?
}

/// Asks where to save and exports a backup. `None` when the user cancels.
#[tauri::command]
pub async fn export_backup(app: tauri::AppHandle, include_images: bool) -> Result<Option<BackupSummary>, String> {
    let Some(destination) = rfd::AsyncFileDialog::new()
        .set_title(crate::tr!(
            en: "Choose where to save the Easy Paper backup",
            pt: "Escolha onde salvar o backup do Easy Paper",
        ))
        .pick_folder()
        .await
    else {
        return Ok(None);
    };
    let destination = destination.path().to_path_buf();

    blocking(move || sync_service::export_backup(&app, &destination, include_images))
        .await
        .map(Some)
}

/// Asks for a backup file and restores it. `None` when the user cancels.
#[tauri::command]
pub async fn import_backup(app: tauri::AppHandle) -> Result<Option<BackupSummary>, String> {
    let Some(file) = rfd::AsyncFileDialog::new()
        .set_title(crate::tr!(en: "Choose an Easy Paper backup", pt: "Escolha um backup do Easy Paper"))
        .add_filter(crate::tr!(en: "Easy Paper backup", pt: "Backup do Easy Paper"), &["json"])
        .set_file_name(BACKUP_FILE_NAME)
        .pick_file()
        .await
    else {
        return Ok(None);
    };
    let file: PathBuf = file.path().to_path_buf();

    blocking(move || sync_service::import_backup(&app, &file)).await.map(Some)
}

#[tauri::command]
pub async fn pick_sync_folder() -> Option<String> {
    rfd::AsyncFileDialog::new()
        .set_title(crate::tr!(
            en: "Choose a sync folder (for example inside OneDrive)",
            pt: "Escolha uma pasta de sincronização (por exemplo, dentro do OneDrive)",
        ))
        .pick_folder()
        .await
        .map(|handle| handle.path().to_string_lossy().to_string())
}

#[tauri::command]
pub fn inspect_sync_folder(folder: String) -> Result<Option<SyncFolderInfo>, String> {
    sync_service::inspect_sync_folder(&folder)
}

#[tauri::command]
pub async fn connect_sync_folder(
    app: tauri::AppHandle,
    folder: String,
    use_folder_settings: bool,
) -> Result<SyncStatus, String> {
    blocking(move || sync_service::connect_sync_folder(&app, &folder, use_folder_settings)).await
}

#[tauri::command]
pub fn disconnect_sync(app: tauri::AppHandle) -> Result<AppConfig, String> {
    sync_service::disconnect_sync(&app)
}

#[tauri::command]
pub async fn sync_now(app: tauri::AppHandle) -> Result<SyncStatus, String> {
    blocking(move || sync_service::sync_now(&app)).await
}

#[tauri::command]
pub fn get_sync_status(app: tauri::AppHandle) -> SyncStatus {
    sync_service::sync_status(&app)
}
