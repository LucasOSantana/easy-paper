use tauri::Manager;
use crate::models::AppConfig;
use crate::services::config_service::save_config;
use crate::services::taskbar_service::set_taskbar_transparency;
use crate::state::ConfigState;

#[tauri::command]
pub async fn set_taskbar_transparent(app: tauri::AppHandle, transparent: bool) -> Result<AppConfig, String> {
    tauri::async_runtime::spawn_blocking(move || apply_and_save(&app, transparent))
        .await
        .map_err(|e| format!("Taskbar update was interrupted: {}", e))?
}

fn apply_and_save(app: &tauri::AppHandle, transparent: bool) -> Result<AppConfig, String> {
    set_taskbar_transparency(app, transparent)?;

    let config_state = app.state::<ConfigState>();
    let mut config = config_state.0.lock().unwrap();
    let updated_config = AppConfig {
        taskbar_transparent: transparent,
        ..config.clone()
    };

    save_config(app, &updated_config)
        .map_err(|e| format!("Failed to save configuration to disk: {}", e))?;

    *config = updated_config.clone();
    Ok(updated_config)
}
