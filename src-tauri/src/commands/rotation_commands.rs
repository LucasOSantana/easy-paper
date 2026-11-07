use crate::models::{
    AppConfig, AutoRotateConfig, MAX_ROTATION_INTERVAL_MINUTES, MIN_ROTATION_INTERVAL_MINUTES,
};
use crate::services::config_service::save_config;
use crate::services::rotation_service::reset_rotation_timer;
use crate::state::ConfigState;

#[tauri::command]
pub fn set_auto_rotate(
    app: tauri::AppHandle,
    config_state: tauri::State<ConfigState>,
    settings: AutoRotateConfig,
) -> Result<AppConfig, String> {
    if !(MIN_ROTATION_INTERVAL_MINUTES..=MAX_ROTATION_INTERVAL_MINUTES)
        .contains(&settings.interval_minutes)
    {
        return Err(format!(
            "The interval must be between {} minutes and {} days.",
            MIN_ROTATION_INTERVAL_MINUTES,
            MAX_ROTATION_INTERVAL_MINUTES / (24 * 60)
        ));
    }

    let mut config = config_state.0.lock().unwrap();
    let updated_config = AppConfig {
        auto_rotate: settings,
        ..config.clone()
    };

    save_config(&app, &updated_config)
        .map_err(|e| format!("Failed to save configuration to disk: {}", e))?;

    *config = updated_config.clone();
    drop(config);

    reset_rotation_timer(&app);
    Ok(updated_config)
}
