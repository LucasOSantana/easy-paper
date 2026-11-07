use crate::models::UpdateInfo;
use crate::services::update_service::{self, UpdateState};

/// Result of the last check (the startup one, usually). `None` while it is
/// still running or if it failed.
#[tauri::command]
pub fn get_update_status(state: tauri::State<UpdateState>) -> Option<UpdateInfo> {
    state.0.lock().unwrap().clone()
}

/// Manual "Check for updates".
#[tauri::command]
pub async fn check_for_updates(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    tauri::async_runtime::spawn_blocking(move || update_service::check_for_updates(&app))
        .await
        .map_err(|e| format!("Update check was interrupted: {}", e))?
}
