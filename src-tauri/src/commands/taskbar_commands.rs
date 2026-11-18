use crate::models::AppConfig;
use crate::services::config_service::modify_config;
use crate::services::taskbar_service::set_taskbar_transparency;

#[tauri::command]
pub async fn set_taskbar_transparent(app: tauri::AppHandle, transparent: bool) -> Result<AppConfig, String> {
    tauri::async_runtime::spawn_blocking(move || apply_and_save(&app, transparent))
        .await
        .map_err(|e| crate::tr!(
            en: format!("Taskbar update was interrupted: {}", e),
            pt: format!("A atualização da barra de tarefas foi interrompida: {}", e),
        ))?
}

fn apply_and_save(app: &tauri::AppHandle, transparent: bool) -> Result<AppConfig, String> {
    set_taskbar_transparency(app, transparent)?;

    modify_config(app, |config| {
        config.taskbar_transparent = transparent;
        Ok(())
    })
}
