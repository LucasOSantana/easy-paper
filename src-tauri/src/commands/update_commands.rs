use crate::models::UpdateInfo;
use crate::services::update_service::{self, UpdateState};

/// Result of the last check (the startup one, usually). `None` while it is
/// still running or if it failed.
#[tauri::command]
pub fn get_update_status(state: tauri::State<UpdateState>) -> Option<UpdateInfo> {
    state.info()
}

/// Manual "Check for updates".
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    update_service::check_for_updates(&app).await
}

/// Downloads and installs the available update. On success the app closes
/// and the installer opens the new version, so this only returns on failure.
#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    update_service::install_update(&app).await
}
