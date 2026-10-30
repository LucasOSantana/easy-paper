use tauri::Manager;
use crate::services::window_service::{hide_main_window, show_settings_window};

#[tauri::command]
pub fn hide_window(app: tauri::AppHandle) {
    hide_main_window(&app);
}

// Async: creating a window from a synchronous command deadlocks on Windows.
#[tauri::command]
pub async fn open_settings_window(app: tauri::AppHandle) {
    show_settings_window(&app);
}

#[tauri::command]
pub fn close_settings_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.close();
    }
}
